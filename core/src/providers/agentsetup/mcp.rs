//! MCP clients: registering `polterware -db <db> mcp` with Claude Code,
//! Codex, and Cursor.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Map, Value};

use super::command::{Failure, home_dir, look_path, run};
use super::{Error, Result};

/// The entry each client keeps for Polterware.
pub const MCP_SERVER_NAME: &str = "polterware";

/// How a client stands with the polterware entry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum McpStatus {
    #[default]
    Unknown,
    NotInstalled,
    NotConfigured,
    Configured,
    ConfiguredOther,
}

impl McpStatus {
    /// Names the status in the CLI's JSON and the Desktop.
    pub fn as_str(&self) -> &'static str {
        match self {
            McpStatus::NotInstalled => "not_installed",
            McpStatus::NotConfigured => "not_configured",
            McpStatus::Configured => "configured",
            McpStatus::ConfiguredOther => "configured_other",
            McpStatus::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for McpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

type StatusHook = Box<dyn Fn(&str, &str) -> Result<McpStatus> + Send + Sync>;
type AddHook = Box<dyn Fn(&str, &str) -> Result<()> + Send + Sync>;
type RemoveHook = Box<dyn Fn() -> Result<()> + Send + Sync>;

/// Configures one agent client to launch `polterware -db <db> mcp`.
pub struct McpClient {
    pub id: &'static str,
    pub name: &'static str,
    pub(crate) status: StatusHook,
    pub(crate) add: AddHook,
    pub(crate) remove: RemoveHook,
}

impl McpClient {
    /// Reports whether the client launches this binary on this database.
    pub fn status(&self, core: &str, db_path: &str) -> Result<McpStatus> {
        (self.status)(core, db_path)
    }

    /// Drops the polterware entry from the client.
    pub fn remove(&self) -> Result<()> {
        (self.remove)()
    }

    pub(crate) fn add(&self, core: &str, db_path: &str) -> Result<()> {
        (self.add)(core, db_path)
    }
}

/// The arguments a client passes to the binary.
pub fn mcp_args(db_path: &str) -> Vec<String> {
    vec!["-db".to_string(), db_path.to_string(), "mcp".to_string()]
}

/// Lists the clients Polterware can set up.
pub fn mcp_clients() -> Vec<McpClient> {
    vec![
        McpClient {
            id: "claude",
            name: "Claude Code",
            status: Box::new(|core, db_path| {
                cli_status("claude", &["mcp", "get", MCP_SERVER_NAME], core, db_path)
            }),
            add: Box::new(|core, db_path| {
                let mut args = vec![
                    "mcp".to_string(),
                    "add".to_string(),
                    "--scope".to_string(),
                    "user".to_string(),
                    MCP_SERVER_NAME.to_string(),
                    "--".to_string(),
                    core.to_string(),
                ];
                args.extend(mcp_args(db_path));
                run_client_cli("claude", &args)
            }),
            remove: Box::new(|| {
                run_client_cli(
                    "claude",
                    &[
                        "mcp".to_string(),
                        "remove".to_string(),
                        "--scope".to_string(),
                        "user".to_string(),
                        MCP_SERVER_NAME.to_string(),
                    ],
                )
            }),
        },
        McpClient {
            id: "codex",
            name: "Codex",
            status: Box::new(|core, db_path| {
                cli_status("codex", &["mcp", "get", MCP_SERVER_NAME], core, db_path)
            }),
            add: Box::new(|core, db_path| {
                let mut args = vec![
                    "mcp".to_string(),
                    "add".to_string(),
                    MCP_SERVER_NAME.to_string(),
                    "--".to_string(),
                    core.to_string(),
                ];
                args.extend(mcp_args(db_path));
                run_client_cli("codex", &args)
            }),
            remove: Box::new(|| {
                run_client_cli(
                    "codex",
                    &[
                        "mcp".to_string(),
                        "remove".to_string(),
                        MCP_SERVER_NAME.to_string(),
                    ],
                )
            }),
        },
        McpClient {
            id: "cursor",
            name: "Cursor",
            status: Box::new(cursor_status),
            add: Box::new(cursor_add),
            remove: Box::new(cursor_remove),
        },
    ]
}

/// Returns the client with `id` (claude, codex, or cursor).
pub fn find_mcp_client(id: &str) -> Option<McpClient> {
    mcp_clients().into_iter().find(|client| client.id == id)
}

/// Installs or refreshes the polterware entry.
pub fn configure_mcp(client: &McpClient, core: &str, db_path: &str) -> Result<()> {
    let status = client.status(core, db_path)?;
    match status {
        McpStatus::NotInstalled => Err(Error(format!("{} is not installed", client.name))),
        McpStatus::Configured => Ok(()),
        McpStatus::ConfiguredOther => {
            client.remove()?;
            client.add(core, db_path)
        }
        McpStatus::NotConfigured | McpStatus::Unknown => client.add(core, db_path),
    }
}

fn cli_status<S: AsRef<str>>(
    binary: &str,
    args: &[S],
    core: &str,
    db_path: &str,
) -> Result<McpStatus> {
    if look_path(binary).is_none() {
        return Ok(McpStatus::NotInstalled);
    }
    let (output, failure) = run(binary, args, Duration::from_secs(20));
    match failure {
        Some(Failure::TimedOut) => Err(Error(format!("{binary} timed out"))),
        Some(Failure::Exit { .. }) => Ok(McpStatus::NotConfigured),
        Some(Failure::Spawn(message)) => Err(Error(message)),
        None => {
            let text = String::from_utf8_lossy(&output);
            if text.contains(core) && text.contains(db_path) {
                Ok(McpStatus::Configured)
            } else {
                Ok(McpStatus::ConfiguredOther)
            }
        }
    }
}

fn run_client_cli<S: AsRef<str>>(binary: &str, args: &[S]) -> Result<()> {
    let (output, failure) = run(binary, args, Duration::from_secs(30));
    match failure {
        None => Ok(()),
        Some(failure) => {
            let mut message = String::from_utf8_lossy(&output).trim().to_string();
            if message.is_empty() {
                message = failure.message();
            }
            Err(Error(format!("{binary}: {}", first_line(&message))))
        }
    }
}

fn first_line(text: &str) -> &str {
    match text.find('\n') {
        Some(index) => &text[..index],
        None => text,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct CursorServer {
    #[serde(default)]
    command: String,
    #[serde(default)]
    args: Vec<String>,
}

fn cursor_config_path() -> Option<PathBuf> {
    let home = home_dir().ok()?;
    let dir = Path::new(&home).join(".cursor");
    if fs::metadata(&dir).is_err() && look_path("cursor").is_none() {
        return None;
    }
    Some(dir.join("mcp.json"))
}

/// Keeps unknown keys intact; a missing file is an empty config.
fn read_cursor_config(path: &Path) -> Result<(Map<String, Value>, Map<String, Value>)> {
    let mut root = Map::new();
    let mut servers = Map::new();
    let body = match fs::read(path) {
        Ok(body) => body,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok((root, servers));
        }
        Err(err) => return Err(Error(err.to_string())),
    };
    if !String::from_utf8_lossy(&body).trim().is_empty() {
        let parsed: Value = serde_json::from_slice(&body).map_err(|_| not_valid_json(path))?;
        root = match parsed {
            Value::Object(root) => root,
            _ => return Err(not_valid_json(path)),
        };
    }
    if let Some(raw) = root.get("mcpServers")
        && !raw.is_null()
    {
        servers = match raw {
            Value::Object(servers) => servers.clone(),
            _ => {
                return Err(Error(format!(
                    "{}: mcpServers is not an object",
                    path.display()
                )));
            }
        };
    }
    Ok((root, servers))
}

fn not_valid_json(path: &Path) -> Error {
    Error(format!(
        "{} is not valid JSON; fix it by hand",
        path.display()
    ))
}

fn write_cursor_config(
    path: &Path,
    mut root: Map<String, Value>,
    servers: Map<String, Value>,
) -> Result<()> {
    root.insert("mcpServers".to_string(), Value::Object(servers));
    let mut body =
        serde_json::to_string_pretty(&Value::Object(root)).map_err(|err| Error(err.to_string()))?;
    body.push('\n');
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| Error(err.to_string()))?;
    }
    let mode = match fs::metadata(path) {
        Ok(metadata) => mode_of(&metadata),
        Err(_) => 0o644,
    };
    let tmp = PathBuf::from(format!("{}.polterware-tmp", path.display()));
    write_file(&tmp, body.as_bytes(), mode)?;
    fs::rename(&tmp, path).map_err(|err| Error(err.to_string()))
}

#[cfg(unix)]
fn mode_of(metadata: &fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o777
}

#[cfg(not(unix))]
fn mode_of(_metadata: &fs::Metadata) -> u32 {
    0o644
}

#[cfg(unix)]
fn write_file(path: &Path, body: &[u8], mode: u32) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(mode)
        .open(path)
        .map_err(|err| Error(err.to_string()))?;
    file.write_all(body).map_err(|err| Error(err.to_string()))
}

#[cfg(not(unix))]
fn write_file(path: &Path, body: &[u8], _mode: u32) -> Result<()> {
    fs::write(path, body).map_err(|err| Error(err.to_string()))
}

fn cursor_status(core: &str, db_path: &str) -> Result<McpStatus> {
    let Some(path) = cursor_config_path() else {
        return Ok(McpStatus::NotInstalled);
    };
    let (_, servers) = read_cursor_config(&path)?;
    let Some(raw) = servers.get(MCP_SERVER_NAME) else {
        return Ok(McpStatus::NotConfigured);
    };
    match serde_json::from_value::<CursorServer>(raw.clone()) {
        Ok(entry) if entry.command == core && entry.args == mcp_args(db_path) => {
            Ok(McpStatus::Configured)
        }
        _ => Ok(McpStatus::ConfiguredOther),
    }
}

fn cursor_add(core: &str, db_path: &str) -> Result<()> {
    let Some(path) = cursor_config_path() else {
        return Err(Error("Cursor is not installed".to_string()));
    };
    let (root, mut servers) = read_cursor_config(&path)?;
    let entry = serde_json::to_value(CursorServer {
        command: core.to_string(),
        args: mcp_args(db_path),
    })
    .map_err(|err| Error(err.to_string()))?;
    servers.insert(MCP_SERVER_NAME.to_string(), entry);
    write_cursor_config(&path, root, servers)
}

fn cursor_remove() -> Result<()> {
    let Some(path) = cursor_config_path() else {
        return Ok(());
    };
    let (root, mut servers) = read_cursor_config(&path)?;
    if servers.remove(MCP_SERVER_NAME).is_none() {
        return Ok(());
    }
    write_cursor_config(&path, root, servers)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::providers::agentsetup::test_support::{EnvGuard, lock_env};

    struct CursorHome {
        config: PathBuf,
        _home: tempfile::TempDir,
        _empty_path: tempfile::TempDir,
        _home_guard: EnvGuard,
        _path_guard: EnvGuard,
    }

    /// Points HOME at a temp dir; with_dir controls whether `~/.cursor` exists.
    fn cursor_home(with_dir: bool) -> CursorHome {
        let home = tempfile::tempdir().expect("home");
        let empty_path = tempfile::tempdir().expect("empty path");
        let home_guard = EnvGuard::set("HOME", home.path());
        // Hide any real `cursor` binary so installation is decided by ~/.cursor alone.
        let path_guard = EnvGuard::set("PATH", empty_path.path());
        if with_dir {
            fs::create_dir_all(home.path().join(".cursor")).expect("cursor dir");
        }
        let config = home.path().join(".cursor").join("mcp.json");
        CursorHome {
            config,
            _home: home,
            _empty_path: empty_path,
            _home_guard: home_guard,
            _path_guard: path_guard,
        }
    }

    fn read_json(path: &Path) -> Map<String, Value> {
        let body = fs::read(path).expect("read config");
        match serde_json::from_slice(&body).expect("valid JSON") {
            Value::Object(root) => root,
            other => panic!("config is not an object: {other}"),
        }
    }

    fn servers_of(path: &Path) -> Map<String, Value> {
        let root = read_json(path);
        match root.get("mcpServers").expect("mcpServers") {
            Value::Object(servers) => servers.clone(),
            other => panic!("mcpServers is not an object: {other}"),
        }
    }

    #[test]
    fn cursor_not_installed() {
        let _lock = lock_env();
        let _home = cursor_home(false);
        let status = cursor_status("/bin/polterware", "/tmp/db").expect("status");
        assert_eq!(status, McpStatus::NotInstalled);
        assert!(
            cursor_add("/bin/polterware", "/tmp/db").is_err(),
            "cursorAdd must fail when Cursor is not installed"
        );
        cursor_remove().expect("cursorRemove without Cursor must be a no-op");
    }

    #[test]
    fn cursor_missing_file_is_not_configured() {
        let _lock = lock_env();
        let _home = cursor_home(true);
        let status = cursor_status("/bin/polterware", "/tmp/db").expect("status");
        assert_eq!(status, McpStatus::NotConfigured);
    }

    #[test]
    fn cursor_add_writes_the_entry_and_is_configured() {
        let _lock = lock_env();
        let home = cursor_home(true);
        cursor_add("/bin/polterware", "/tmp/db").expect("add");
        let servers = servers_of(&home.config);
        let entry: CursorServer =
            serde_json::from_value(servers.get(MCP_SERVER_NAME).expect("entry").clone())
                .expect("entry shape");
        assert_eq!(entry.command, "/bin/polterware");
        assert_eq!(entry.args.join(" "), "-db /tmp/db mcp");
        let status = cursor_status("/bin/polterware", "/tmp/db").expect("status");
        assert_eq!(status, McpStatus::Configured);
    }

    #[test]
    fn cursor_status_detects_a_different_config() {
        let _lock = lock_env();
        let _home = cursor_home(true);
        cursor_add("/bin/polterware", "/tmp/db").expect("add");
        for (name, core, db_path) in [
            ("other binary", "/other/polterware", "/tmp/db"),
            ("other db", "/bin/polterware", "/tmp/other.db"),
        ] {
            let status = cursor_status(core, db_path).expect("status");
            assert_eq!(status, McpStatus::ConfiguredOther, "{name}");
        }
    }

    #[test]
    fn cursor_add_preserves_unknown_keys_and_other_servers() {
        let _lock = lock_env();
        let home = cursor_home(true);
        let seed = r#"{"theme":"dark","mcpServers":{"other":{"command":"x","args":["y"]}}}"#;
        fs::write(&home.config, seed).expect("seed config");
        set_mode(&home.config, 0o600);
        cursor_add("/bin/polterware", "/tmp/db").expect("add");
        let root = read_json(&home.config);
        assert_eq!(
            root.get("theme"),
            Some(&Value::String("dark".to_string())),
            "unknown root key lost"
        );
        let servers = servers_of(&home.config);
        assert!(
            servers.contains_key("other"),
            "existing server entry dropped"
        );
        assert!(
            servers.contains_key(MCP_SERVER_NAME),
            "polterware entry missing"
        );
        assert_eq!(mode(&home.config), 0o600, "file mode changed");
    }

    #[test]
    fn cursor_add_is_idempotent() {
        let _lock = lock_env();
        let home = cursor_home(true);
        for _ in 0..2 {
            cursor_add("/bin/polterware", "/tmp/db").expect("add");
        }
        assert_eq!(servers_of(&home.config).len(), 1, "want exactly one server");
    }

    #[test]
    fn cursor_remove_only_deletes_polterware() {
        let _lock = lock_env();
        let home = cursor_home(true);
        fs::write(&home.config, r#"{"mcpServers":{"other":{"command":"x"}}}"#)
            .expect("seed config");
        cursor_add("/bin/polterware", "/tmp/db").expect("add");
        cursor_remove().expect("remove");
        let servers = servers_of(&home.config);
        assert!(
            !servers.contains_key(MCP_SERVER_NAME),
            "polterware entry still present"
        );
        assert!(
            servers.contains_key("other"),
            "unrelated server was removed"
        );
        cursor_remove().expect("a second remove is a no-op");
    }

    #[test]
    fn cursor_invalid_json_is_not_overwritten() {
        let _lock = lock_env();
        let home = cursor_home(true);
        const BROKEN: &str = r#"{"mcpServers": "#;
        fs::write(&home.config, BROKEN).expect("seed config");
        assert!(
            cursor_add("/bin/polterware", "/tmp/db").is_err(),
            "cursorAdd must refuse invalid JSON"
        );
        assert_eq!(
            fs::read_to_string(&home.config).expect("read config"),
            BROKEN,
            "invalid file was modified"
        );
        assert!(
            cursor_status("/bin/polterware", "/tmp/db").is_err(),
            "cursorStatus must report invalid JSON"
        );
    }

    #[test]
    fn read_cursor_config_rejects_non_object_servers() {
        let _lock = lock_env();
        let dir = tempfile::tempdir().expect("dir");
        let path = dir.path().join("mcp.json");
        fs::write(&path, r#"{"mcpServers":[1]}"#).expect("seed config");
        assert!(
            read_cursor_config(&path).is_err(),
            "want error for non-object mcpServers"
        );
    }

    #[test]
    fn read_cursor_config_treats_empty_and_null_as_empty() {
        let _lock = lock_env();
        let dir = tempfile::tempdir().expect("dir");
        for (name, body) in [("empty", "  \n"), ("null", r#"{"mcpServers":null}"#)] {
            let path = dir.path().join(format!("{name}.json"));
            fs::write(&path, body).expect("seed config");
            let (_, servers) = read_cursor_config(&path).expect("config");
            assert!(servers.is_empty(), "{name}: servers = {servers:?}");
        }
    }

    #[test]
    fn configure_client_refreshes_a_stale_entry() {
        let _lock = lock_env();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let added = Arc::clone(&calls);
        let removed = Arc::clone(&calls);
        let mut client = McpClient {
            id: "fake",
            name: "Fake",
            status: Box::new(|_, _| Ok(McpStatus::ConfiguredOther)),
            add: Box::new(move |_, _| {
                lock(&added).push("add".to_string());
                Ok(())
            }),
            remove: Box::new(move || {
                lock(&removed).push("remove".to_string());
                Ok(())
            }),
        };
        configure_mcp(&client, "core", "db").expect("configure");
        assert_eq!(lock(&calls).join(","), "remove,add");

        lock(&calls).clear();
        client.status = Box::new(|_, _| Ok(McpStatus::Configured));
        configure_mcp(&client, "core", "db").expect("configure");
        assert!(
            lock(&calls).is_empty(),
            "configured client must be left alone, calls = {:?}",
            lock(&calls)
        );

        client.status = Box::new(|_, _| Ok(McpStatus::NotInstalled));
        assert!(
            configure_mcp(&client, "core", "db").is_err(),
            "want error for a client that is not installed"
        );
    }

    #[test]
    fn mcp_args_point_at_the_binary_and_database() {
        assert_eq!(mcp_args("/x.db").join(" "), "-db /x.db mcp");
    }

    #[test]
    fn mcp_clients_cover_claude_codex_and_cursor() {
        let clients = mcp_clients();
        let ids: Vec<&str> = clients.iter().map(|client| client.id).collect();
        assert_eq!(ids.join(","), "claude,codex,cursor");
        for client in &clients {
            assert!(!client.name.is_empty(), "client {} has no name", client.id);
        }
    }

    #[test]
    fn find_mcp_client_reads_the_list() {
        let client = find_mcp_client("codex").expect("codex client");
        assert_eq!(client.name, "Codex");
        assert!(find_mcp_client("nope").is_none());
    }

    #[test]
    fn statuses_read_like_the_cli_json() {
        assert_eq!(McpStatus::default(), McpStatus::Unknown);
        assert_eq!(McpStatus::Unknown.as_str(), "unknown");
        assert_eq!(McpStatus::NotInstalled.to_string(), "not_installed");
        assert_eq!(McpStatus::ConfiguredOther.as_str(), "configured_other");
    }

    fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
        mutex
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[cfg(unix)]
    fn set_mode(path: &Path, mode: u32) {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("chmod");
    }

    #[cfg(unix)]
    fn mode(path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path).expect("stat").permissions().mode() & 0o777
    }

    #[cfg(not(unix))]
    fn set_mode(_path: &Path, _mode: u32) {}

    #[cfg(not(unix))]
    fn mode(_path: &Path) -> u32 {
        0o600
    }
}
