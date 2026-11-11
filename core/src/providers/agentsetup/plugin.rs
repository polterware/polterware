//! The embedded Claude Code plugin and the local marketplace that carries it.
//!
//! The canonical copy lives at `backend/providers/agentsetup/plugin`, because
//! the Go plugin is embedded from there and `plugins/polterware` is a symlink
//! to it for the repository marketplace. This module includes the same files,
//! so there is one copy.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{Error, Result};

pub const PLUGIN_NAME: &str = "polterware";
pub const MARKETPLACE_NAME: &str = "polterware-local";
pub const PLUGIN_ID: &str = "polterware@polterware-local";

const PLUGIN_MANIFEST: &str =
    include_str!("../../../../backend/providers/agentsetup/plugin/.claude-plugin/plugin.json");
const PLUGIN_HOOKS: &str =
    include_str!("../../../../backend/providers/agentsetup/plugin/hooks/hooks.json");
const PLUGIN_SKILL: &str =
    include_str!("../../../../backend/providers/agentsetup/plugin/skills/polter/SKILL.md");

/// Every embedded plugin file, keyed by its path under `plugin/`.
pub(crate) const PLUGIN_FILES: [(&str, &str); 3] = [
    (".claude-plugin/plugin.json", PLUGIN_MANIFEST),
    ("hooks/hooks.json", PLUGIN_HOOKS),
    ("skills/polter/SKILL.md", PLUGIN_SKILL),
];

/// The version of the plugin bundled in this binary.
pub fn embedded_version() -> String {
    let Ok(manifest) = serde_json::from_str::<Value>(PLUGIN_MANIFEST) else {
        return String::new();
    };
    manifest
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// The local marketplace under the database directory.
pub fn marketplace_dir(db_dir: &str) -> PathBuf {
    Path::new(db_dir).join("agent-plugins").join("claude")
}

/// Returns the embedded polter skill.
pub fn skill_file() -> &'static [u8] {
    PLUGIN_SKILL.as_bytes()
}

/// Writes the local marketplace and the embedded plugin into `dir`, replacing
/// the previous copy of the plugin so stale files never linger.
pub fn write_marketplace(dir: &Path) -> Result<()> {
    let target = dir.join("plugins").join(PLUGIN_NAME);
    if target.exists() {
        fs::remove_dir_all(&target).map_err(io_error)?;
    }
    fs::create_dir_all(&target).map_err(io_error)?;
    for (rel, body) in PLUGIN_FILES {
        let dest = target.join(rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        fs::write(&dest, body).map_err(io_error)?;
    }
    let manifest = serde_json::json!({
        "name": MARKETPLACE_NAME,
        "owner": {"name": "Polterware"},
        "plugins": [{
            "name": PLUGIN_NAME,
            "source": format!("./plugins/{PLUGIN_NAME}"),
            "description": "Write Polterware Work cards in Polter format, lint .pw.md files, and audit each session as one AgentRun.",
        }],
    });
    let mut body = serde_json::to_string_pretty(&manifest).map_err(json_error)?;
    body.push('\n');
    let metadata_dir = dir.join(".claude-plugin");
    fs::create_dir_all(&metadata_dir).map_err(io_error)?;
    fs::write(metadata_dir.join("marketplace.json"), body).map_err(io_error)
}

/// Names the installed agent integrations older than this binary: the Claude
/// Code plugin in the local marketplace and the shared skill file. It reads
/// files only, so it is cheap enough for every status; an integration that was
/// never installed says nothing.
pub fn stale_notices(db_dir: &str) -> Vec<String> {
    let mut out = Vec::new();
    let manifest = marketplace_dir(db_dir)
        .join("plugins")
        .join(PLUGIN_NAME)
        .join(".claude-plugin")
        .join("plugin.json");
    if let Ok(body) = fs::read(&manifest)
        && let Ok(installed) = serde_json::from_slice::<Value>(&body)
    {
        let version = installed
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if version != embedded_version() {
            out.push(format!(
                "setup: the Claude Code plugin is {version} and this binary has {}; run polterware setup claude",
                embedded_version()
            ));
        }
    }
    if let Ok(dir) = super::skill_dir()
        && let Ok(have) = fs::read(dir.join("SKILL.md"))
        && have != skill_file()
    {
        out.push(format!(
            "setup: the agent skill in {} differs from this binary's; run polterware setup codex",
            dir.display()
        ));
    }
    out
}

fn io_error(err: std::io::Error) -> Error {
    Error(err.to_string())
}

fn json_error(err: serde_json::Error) -> Error {
    Error(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::agentsetup::install_agents;
    use crate::providers::agentsetup::test_support::{EnvGuard, lock_env};

    #[test]
    fn write_marketplace_matches_the_embedded_plugin() {
        let dir = tempfile::tempdir().expect("temp dir");
        write_marketplace(dir.path()).expect("write marketplace");
        for rel in [
            ".claude-plugin/marketplace.json",
            "plugins/polterware/.claude-plugin/plugin.json",
            "plugins/polterware/hooks/hooks.json",
            "plugins/polterware/skills/polter/SKILL.md",
        ] {
            assert!(dir.path().join(rel).exists(), "{rel}");
        }
        assert!(!embedded_version().is_empty(), "no embedded version");
        // A stale file from an older plugin must not survive a rewrite.
        let stale = dir
            .path()
            .join("plugins")
            .join("polterware")
            .join("old.txt");
        fs::write(&stale, "x").expect("write stale");
        write_marketplace(dir.path()).expect("rewrite marketplace");
        assert!(!stale.exists(), "stale file kept");
    }

    #[test]
    fn stale_notices_name_old_installs_only() {
        let _lock = lock_env();
        let home = tempfile::tempdir().expect("home");
        let _home = EnvGuard::set("HOME", home.path());
        let db_dir = tempfile::tempdir().expect("db dir");
        let db = db_dir.path().display().to_string();
        assert!(
            stale_notices(&db).is_empty(),
            "nothing installed, yet {:?}",
            stale_notices(&db)
        );
        write_marketplace(&marketplace_dir(&db)).expect("write marketplace");
        install_agents("").expect("install skill");
        assert!(
            stale_notices(&db).is_empty(),
            "current installs, yet {:?}",
            stale_notices(&db)
        );
        let manifest = marketplace_dir(&db)
            .join("plugins")
            .join(PLUGIN_NAME)
            .join(".claude-plugin")
            .join("plugin.json");
        fs::write(&manifest, r#"{"name":"polterware","version":"0.0.1"}"#).expect("write manifest");
        let dir = crate::providers::agentsetup::skill_dir().expect("skill dir");
        fs::write(dir.join("SKILL.md"), "old skill").expect("write skill");
        let got = stale_notices(&db);
        assert_eq!(got.len(), 2, "notices = {got:?}");
        assert!(
            got[0].contains(&format!(
                "plugin is 0.0.1 and this binary has {}; run polterware setup claude",
                embedded_version()
            )),
            "{}",
            got[0]
        );
        assert!(got[1].contains("run polterware setup codex"), "{}", got[1]);
    }

    #[test]
    fn embedded_files_are_the_repository_copy() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let on_disk =
            fs::read(root.join("backend/providers/agentsetup/plugin/skills/polter/SKILL.md"))
                .expect("repository skill");
        assert_eq!(on_disk, skill_file(), "the skill has one copy");
    }
}
