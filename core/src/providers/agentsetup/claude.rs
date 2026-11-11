//! Installs the plugin through the `claude` CLI, never by editing Claude
//! Code's files, and reports what that CLI says about it.

use std::time::Duration;

use super::command::{self, Failure, look_path};
use super::plugin::{
    MARKETPLACE_NAME, PLUGIN_ID, embedded_version, marketplace_dir, write_marketplace,
};
use super::{Error, Result};

/// What `claude plugin list` reports for the Polterware plugin.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PluginState {
    pub installed: bool,
    pub enabled: bool,
    pub version: String,
}

/// The setup doctor's view of this machine.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    /// Absolute path of `polterware` on PATH, "" when missing.
    pub polterware_path: String,
    /// Absolute path of `claude`, "" when missing.
    pub claude_path: String,
    pub plugin: PluginState,
    pub embedded_ver: String,
    pub marketplace: String,
    /// Why the plugin state could not be read, if it could not.
    pub problem: String,
}

impl Report {
    /// Reports whether the installed plugin matches the binary.
    pub fn current(&self) -> bool {
        self.plugin.installed && self.plugin.version == self.embedded_ver
    }
}

/// Inspects the machine. It never changes anything.
pub fn doctor(db_dir: &str) -> Report {
    let mut report = Report {
        embedded_ver: embedded_version(),
        marketplace: marketplace_dir(db_dir).display().to_string(),
        ..Report::default()
    };
    report.polterware_path = look_path("polterware").unwrap_or_default();
    report.claude_path = look_path("claude").unwrap_or_default();
    if report.claude_path.is_empty() {
        return report;
    }
    match plugin_state() {
        Ok(state) => report.plugin = state,
        Err(err) => report.problem = err.to_string(),
    }
    report
}

#[derive(serde::Deserialize)]
struct PluginEntry {
    #[serde(default)]
    id: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    enabled: bool,
}

fn plugin_state() -> Result<PluginState> {
    let out = run_claude(Duration::from_secs(20), &["plugin", "list", "--json"])?;
    let plugins: Option<Vec<PluginEntry>> = serde_json::from_str(&out)
        .map_err(|_| Error("claude plugin list: unreadable output".to_string()))?;
    for plugin in plugins.unwrap_or_default() {
        if plugin.id == PLUGIN_ID {
            return Ok(PluginState {
                installed: true,
                enabled: plugin.enabled,
                version: plugin.version,
            });
        }
    }
    Ok(PluginState::default())
}

/// Writes the marketplace and installs, or updates, the plugin through the
/// `claude` CLI.
pub fn install_claude(db_dir: &str) -> Result<()> {
    if look_path("claude").is_none() {
        return Err(Error("claude is not installed or not on PATH".to_string()));
    }
    let dir = marketplace_dir(db_dir);
    write_marketplace(&dir)?;
    let known = marketplace_known()?;
    if known {
        run_claude(
            Duration::from_secs(60),
            &["plugin", "marketplace", "update", MARKETPLACE_NAME],
        )?;
    } else {
        let dir = dir.display().to_string();
        run_claude(
            Duration::from_secs(60),
            &["plugin", "marketplace", "add", dir.as_str()],
        )?;
    }
    let state = plugin_state()?;
    let failure = if !state.installed {
        run_claude(Duration::from_secs(60), &["plugin", "install", PLUGIN_ID]).err()
    } else if state.version != embedded_version() {
        run_claude(Duration::from_secs(60), &["plugin", "update", PLUGIN_ID]).err()
    } else if !state.enabled {
        run_claude(Duration::from_secs(30), &["plugin", "enable", PLUGIN_ID]).err()
    } else {
        None
    };
    match failure {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

/// Removes the plugin and the local marketplace entry.
pub fn uninstall_claude() -> Result<()> {
    if look_path("claude").is_none() {
        return Err(Error("claude is not installed or not on PATH".to_string()));
    }
    let state = plugin_state()?;
    if state.installed {
        run_claude(Duration::from_secs(60), &["plugin", "uninstall", PLUGIN_ID])?;
    }
    let known = marketplace_known()?;
    if !known {
        return Ok(());
    }
    run_claude(
        Duration::from_secs(30),
        &["plugin", "marketplace", "remove", MARKETPLACE_NAME],
    )
    .map(|_| ())
}

/// Enables or disables the installed plugin.
pub fn set_claude_enabled(enabled: bool) -> Result<()> {
    let verb = if enabled { "enable" } else { "disable" };
    run_claude(Duration::from_secs(30), &["plugin", verb, PLUGIN_ID]).map(|_| ())
}

#[derive(serde::Deserialize)]
struct MarketplaceEntry {
    #[serde(default)]
    name: String,
}

fn marketplace_known() -> Result<bool> {
    let out = run_claude(
        Duration::from_secs(20),
        &["plugin", "marketplace", "list", "--json"],
    )?;
    let items: Option<Vec<MarketplaceEntry>> = serde_json::from_str(&out)
        .map_err(|_| Error("claude plugin marketplace list: unreadable output".to_string()))?;
    Ok(items
        .unwrap_or_default()
        .iter()
        .any(|item| item.name == MARKETPLACE_NAME))
}

fn run_claude<S: AsRef<str>>(timeout: Duration, args: &[S]) -> Result<String> {
    let (output, failure) = command::run("claude", args, timeout);
    match failure {
        None => Ok(String::from_utf8_lossy(&output).into_owned()),
        Some(Failure::TimedOut) => {
            let head: Vec<&str> = args.iter().take(2).map(AsRef::as_ref).collect();
            Err(Error(format!("claude {} timed out", head.join(" "))))
        }
        Some(failure) => {
            let mut message = String::from_utf8_lossy(&output).trim().to_string();
            if message.is_empty() {
                message = failure.message();
            }
            if let Some(index) = message.find('\n') {
                message.truncate(index);
            }
            Err(Error(format!("claude: {message}")))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::providers::agentsetup::test_support::{EnvGuard, lock_env};

    /// Installs a `claude` script on PATH that logs its arguments and answers
    /// the plugin and marketplace list queries from JSON files.
    struct FakeClaude {
        dir: PathBuf,
        _home: tempfile::TempDir,
        _path: EnvGuard,
    }

    impl FakeClaude {
        fn new(plugins: &str, marketplaces: &str) -> FakeClaude {
            let home = tempfile::tempdir().expect("fake claude dir");
            let dir = home.path().to_path_buf();
            let calls_log = dir.join("calls.log");
            let plugins_json = dir.join("plugins.json");
            let marketplaces_json = dir.join("marketplaces.json");
            let script = format!(
                "#!/bin/sh\necho \"$@\" >> \"{}\"\ncase \"$1 $2 $3\" in\n  \"plugin list --json\") cat \"{}\" ;;\n  \"plugin marketplace list\") cat \"{}\" ;;\nesac\nexit 0\n",
                calls_log.display(),
                plugins_json.display(),
                marketplaces_json.display()
            );
            let claude = dir.join("claude");
            fs::write(&claude, script).expect("write fake claude");
            set_executable(&claude);
            let path = format!("{}:/usr/bin:/bin", dir.display());
            let fake = FakeClaude {
                dir,
                _home: home,
                _path: EnvGuard::set("PATH", path),
            };
            fake.set(plugins, marketplaces);
            fake
        }

        fn set(&self, plugins: &str, marketplaces: &str) {
            fs::write(self.dir.join("plugins.json"), plugins).expect("write plugins");
            fs::write(self.dir.join("marketplaces.json"), marketplaces)
                .expect("write marketplaces");
        }

        fn calls(&self) -> Vec<String> {
            match fs::read_to_string(self.dir.join("calls.log")) {
                Ok(body) => body.trim().lines().map(str::to_string).collect(),
                Err(_) => Vec::new(),
            }
        }
    }

    #[cfg(unix)]
    fn set_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod fake claude");
    }

    #[cfg(not(unix))]
    fn set_executable(_path: &Path) {}

    fn plugin_json(version: &str, enabled: bool) -> String {
        format!(r#"[{{"id":"{PLUGIN_ID}","version":"{version}","enabled":{enabled}}}]"#)
    }

    fn marketplace_json(known: bool) -> String {
        if known {
            format!(r#"[{{"name":"{MARKETPLACE_NAME}"}}]"#)
        } else {
            "[]".to_string()
        }
    }

    fn has_call(calls: &[String], prefix: &str) -> bool {
        calls.iter().any(|call| call.starts_with(prefix))
    }

    #[test]
    fn install_claude_fails_without_the_binary() {
        let _lock = lock_env();
        let empty = tempfile::tempdir().expect("empty path");
        let _path = EnvGuard::set("PATH", empty.path());
        let db = tempfile::tempdir().expect("db");
        let err = install_claude(&db.path().display().to_string()).expect_err("no claude");
        assert!(err.to_string().contains("not installed"), "err = {err}");
        assert!(
            uninstall_claude().is_err(),
            "uninstall must fail without claude"
        );
    }

    #[test]
    fn install_claude_adds_the_marketplace_and_installs_on_a_fresh_machine() {
        let _lock = lock_env();
        let fake = FakeClaude::new("[]", &marketplace_json(false));
        let db = tempfile::tempdir().expect("db");
        let db_dir = db.path().display().to_string();
        install_claude(&db_dir).expect("install");
        let calls = fake.calls();
        assert!(
            has_call(
                &calls,
                &format!(
                    "plugin marketplace add {}",
                    marketplace_dir(&db_dir).display()
                )
            ),
            "marketplace was not added: {calls:?}"
        );
        assert!(
            has_call(&calls, &format!("plugin install {PLUGIN_ID}")),
            "plugin was not installed: {calls:?}"
        );
        assert!(
            marketplace_dir(&db_dir).exists(),
            "marketplace directory was not written"
        );
    }

    #[test]
    fn install_claude_updates_an_outdated_plugin() {
        let _lock = lock_env();
        let fake = FakeClaude::new(&plugin_json("0.0.0-old", true), &marketplace_json(true));
        install_claude(
            &tempfile::tempdir()
                .expect("db")
                .path()
                .display()
                .to_string(),
        )
        .expect("install");
        let calls = fake.calls();
        assert!(
            has_call(
                &calls,
                &format!("plugin marketplace update {MARKETPLACE_NAME}")
            ),
            "known marketplace must be updated: {calls:?}"
        );
        assert!(
            has_call(&calls, &format!("plugin update {PLUGIN_ID}")),
            "outdated plugin must be updated: {calls:?}"
        );
        assert!(
            !has_call(&calls, "plugin install") && !has_call(&calls, "plugin marketplace add"),
            "unexpected fresh-install calls: {calls:?}"
        );
    }

    #[test]
    fn install_claude_enables_a_disabled_current_plugin() {
        let _lock = lock_env();
        let fake = FakeClaude::new(
            &plugin_json(&embedded_version(), false),
            &marketplace_json(true),
        );
        install_claude(
            &tempfile::tempdir()
                .expect("db")
                .path()
                .display()
                .to_string(),
        )
        .expect("install");
        assert!(
            has_call(&fake.calls(), &format!("plugin enable {PLUGIN_ID}")),
            "disabled plugin must be enabled: {:?}",
            fake.calls()
        );
    }

    #[test]
    fn install_claude_leaves_a_current_plugin_alone() {
        let _lock = lock_env();
        let fake = FakeClaude::new(
            &plugin_json(&embedded_version(), true),
            &marketplace_json(true),
        );
        install_claude(
            &tempfile::tempdir()
                .expect("db")
                .path()
                .display()
                .to_string(),
        )
        .expect("install");
        for verb in ["plugin install", "plugin update", "plugin enable"] {
            assert!(
                !has_call(&fake.calls(), verb),
                "{verb:?} must not run for a current plugin: {:?}",
                fake.calls()
            );
        }
    }

    #[test]
    fn uninstall_claude_removes_the_plugin_and_the_marketplace() {
        let _lock = lock_env();
        let fake = FakeClaude::new(
            &plugin_json(&embedded_version(), true),
            &marketplace_json(true),
        );
        uninstall_claude().expect("uninstall");
        let calls = fake.calls();
        assert!(
            has_call(&calls, &format!("plugin uninstall {PLUGIN_ID}"))
                && has_call(
                    &calls,
                    &format!("plugin marketplace remove {MARKETPLACE_NAME}")
                ),
            "calls = {calls:?}"
        );
    }

    #[test]
    fn uninstall_claude_when_nothing_is_installed() {
        let _lock = lock_env();
        let fake = FakeClaude::new("[]", &marketplace_json(false));
        uninstall_claude().expect("uninstall");
        let calls = fake.calls();
        assert!(
            !has_call(&calls, "plugin uninstall") && !has_call(&calls, "plugin marketplace remove"),
            "nothing to remove, got {calls:?}"
        );
    }

    #[test]
    fn set_claude_enabled_uses_the_matching_verb() {
        let _lock = lock_env();
        let fake = FakeClaude::new("[]", "[]");
        set_claude_enabled(true).expect("enable");
        set_claude_enabled(false).expect("disable");
        let calls = fake.calls();
        assert_eq!(
            calls,
            vec![
                format!("plugin enable {PLUGIN_ID}"),
                format!("plugin disable {PLUGIN_ID}")
            ]
        );
    }

    #[test]
    fn doctor_reports_the_plugin_state() {
        let _lock = lock_env();
        let fake = FakeClaude::new(
            &plugin_json(&embedded_version(), true),
            &marketplace_json(true),
        );
        let report = doctor(
            &tempfile::tempdir()
                .expect("db")
                .path()
                .display()
                .to_string(),
        );
        assert!(
            !report.claude_path.is_empty() && report.plugin.installed && report.plugin.enabled,
            "report = {report:?}"
        );
        assert!(
            report.current(),
            "plugin at the embedded version must be current: {report:?}"
        );

        fake.set(&plugin_json("0.0.0-old", true), &marketplace_json(true));
        assert!(
            !doctor(
                &tempfile::tempdir()
                    .expect("db")
                    .path()
                    .display()
                    .to_string()
            )
            .current(),
            "an outdated plugin must not be current"
        );
    }

    #[test]
    fn doctor_without_claude_reports_nothing_installed() {
        let _lock = lock_env();
        let empty = tempfile::tempdir().expect("empty path");
        let _path = EnvGuard::set("PATH", empty.path());
        let report = doctor(
            &tempfile::tempdir()
                .expect("db")
                .path()
                .display()
                .to_string(),
        );
        assert!(
            report.claude_path.is_empty() && !report.plugin.installed && !report.current(),
            "report = {report:?}"
        );
    }

    #[test]
    fn doctor_reports_an_unreadable_plugin_list() {
        let _lock = lock_env();
        let _fake = FakeClaude::new("not json", &marketplace_json(true));
        let report = doctor(
            &tempfile::tempdir()
                .expect("db")
                .path()
                .display()
                .to_string(),
        );
        assert!(!report.problem.is_empty(), "problem = {report:?}");
    }

    #[test]
    fn run_claude_reports_the_first_line_of_a_failure() {
        let _lock = lock_env();
        let home = tempfile::tempdir().expect("fake claude dir");
        let claude = home.path().join("claude");
        fs::write(
            &claude,
            "#!/bin/sh\necho 'boom on line one'\necho 'second line'\nexit 3\n",
        )
        .expect("write fake claude");
        set_executable(&claude);
        let _path = EnvGuard::set("PATH", format!("{}:/usr/bin:/bin", home.path().display()));
        let err = run_claude(Duration::from_secs(5), &["plugin", "list"]).expect_err("failure");
        assert_eq!(err.to_string(), "claude: boom on line one");
    }

    #[test]
    fn run_claude_times_out() {
        let _lock = lock_env();
        let home = tempfile::tempdir().expect("fake claude dir");
        let claude = home.path().join("claude");
        fs::write(&claude, "#!/bin/sh\nexec sleep 5\n").expect("write fake claude");
        set_executable(&claude);
        let _path = EnvGuard::set("PATH", format!("{}:/usr/bin:/bin", home.path().display()));
        let err = run_claude(Duration::from_millis(200), &["plugin", "list"]).expect_err("timeout");
        assert!(err.to_string().contains("timed out"), "err = {err}");
    }
}
