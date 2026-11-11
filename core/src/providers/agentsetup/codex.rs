//! The skill file Codex and other agents read, and the AGENTS.md line that
//! points at the CLI.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::command::home_dir;
use super::plugin::skill_file;
use super::{Error, Result};

const AGENTS_LINE_BEGIN: &str = "<!-- polterware:agent-setup:begin -->";
const AGENTS_LINE_END: &str = "<!-- polterware:agent-setup:end -->";
const AGENTS_LINE: &str = "Polterware: run `polterware format` once for the Polter grammar, read with `polterware query cards`, write Work cards with `polterware apply <file|->`; start with `polterware status` and end with `polterware handoff`.";

/// Where Codex and other agents read the polter skill.
pub fn skill_dir() -> Result<PathBuf> {
    let home = home_dir()?;
    Ok(Path::new(&home)
        .join(".agents")
        .join("skills")
        .join("polter"))
}

/// Describes the skill file and the AGENTS.md line for one project.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentsState {
    pub skill_path: String,
    pub skill_ready: bool,
    pub line_ready: bool,
}

/// Reads the state without changing it. `root` may be empty.
pub fn inspect_agents(root: &str) -> AgentsState {
    let mut state = AgentsState::default();
    if let Ok(dir) = skill_dir() {
        let path = dir.join("SKILL.md");
        let want = skill_file();
        let have = fs::read(&path);
        state.skill_path = path.display().to_string();
        state.skill_ready = matches!(have, Ok(have) if have == want);
    }
    if !root.is_empty() {
        let body = fs::read(Path::new(root).join("AGENTS.md")).unwrap_or_default();
        state.line_ready = String::from_utf8_lossy(&body).contains(AGENTS_LINE_BEGIN);
    }
    state
}

/// Writes the skill file and, when `root` is set, the AGENTS.md line.
pub fn install_agents(root: &str) -> Result<()> {
    let dir = skill_dir()?;
    fs::create_dir_all(&dir).map_err(io_error)?;
    fs::write(dir.join("SKILL.md"), skill_file()).map_err(io_error)?;
    if root.is_empty() {
        return Ok(());
    }
    let path = Path::new(root).join("AGENTS.md");
    let body = match fs::read(&path) {
        Ok(body) => body,
        Err(err) if err.kind() == ErrorKind::NotFound => Vec::new(),
        Err(err) => return Err(io_error(err)),
    };
    let mut text = String::from_utf8_lossy(&body).into_owned();
    if text.contains(AGENTS_LINE_BEGIN) {
        return Ok(());
    }
    let block = format!("{AGENTS_LINE_BEGIN}\n{AGENTS_LINE}\n{AGENTS_LINE_END}\n");
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    if !text.is_empty() {
        text.push('\n');
    }
    text.push_str(&block);
    fs::write(&path, text).map_err(io_error)
}

/// Removes the skill file and the AGENTS.md line.
pub fn uninstall_agents(root: &str) -> Result<()> {
    if let Ok(dir) = skill_dir()
        && dir.exists()
    {
        fs::remove_dir_all(&dir).map_err(io_error)?;
    }
    if root.is_empty() {
        return Ok(());
    }
    let path = Path::new(root).join("AGENTS.md");
    let Ok(body) = fs::read(&path) else {
        return Ok(());
    };
    let text = String::from_utf8_lossy(&body).into_owned();
    let Some(start) = text.find(AGENTS_LINE_BEGIN) else {
        return Ok(());
    };
    let Some(end_match) = text.find(AGENTS_LINE_END) else {
        return Ok(());
    };
    if end_match < start {
        return Ok(());
    }
    let end = end_match + AGENTS_LINE_END.len();
    let text = format!(
        "{}\n{}",
        text[..start].trim_end_matches('\n'),
        text[end..].trim_start_matches('\n')
    );
    if text.trim().is_empty() {
        return fs::remove_file(&path).map_err(io_error);
    }
    fs::write(&path, text).map_err(io_error)
}

fn io_error(err: std::io::Error) -> Error {
    Error(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::agentsetup::test_support::{EnvGuard, lock_env};

    /// Points HOME at a fresh directory and returns a project root beside it.
    fn codex_home() -> (EnvGuard, tempfile::TempDir, tempfile::TempDir) {
        let home = tempfile::tempdir().expect("home");
        let guard = EnvGuard::set("HOME", home.path());
        let root = tempfile::tempdir().expect("root");
        (guard, home, root)
    }

    fn skill_file_exists() -> bool {
        skill_dir()
            .map(|dir| dir.join("SKILL.md").exists())
            .unwrap_or(false)
    }

    #[test]
    fn install_agents_writes_skill_and_agents_line() {
        let _lock = lock_env();
        let (_guard, _home, root) = codex_home();
        let root = root.path().display().to_string();
        install_agents(&root).expect("install");
        let state = inspect_agents(&root);
        assert!(state.skill_ready && state.line_ready, "state = {state:?}");
        let body = fs::read_to_string(Path::new(&root).join("AGENTS.md")).expect("agents file");
        assert!(
            body.contains("polterware status"),
            "AGENTS.md missing the Polterware line:\n{body}"
        );
    }

    #[test]
    fn install_agents_without_root_only_writes_the_skill() {
        let _lock = lock_env();
        let (_guard, _home, _root) = codex_home();
        install_agents("").expect("install");
        let state = inspect_agents("");
        assert!(state.skill_ready && !state.line_ready, "state = {state:?}");
        assert!(skill_file_exists(), "skill file written");
    }

    #[test]
    fn install_agents_is_idempotent() {
        let _lock = lock_env();
        let (_guard, _home, root) = codex_home();
        let root = root.path().display().to_string();
        for _ in 0..3 {
            install_agents(&root).expect("install");
        }
        let body = fs::read_to_string(Path::new(&root).join("AGENTS.md")).expect("agents file");
        assert_eq!(
            body.matches(AGENTS_LINE_BEGIN).count(),
            1,
            "block written more than once:\n{body}"
        );
    }

    #[test]
    fn install_agents_keeps_existing_content() {
        let _lock = lock_env();
        let (_guard, _home, root) = codex_home();
        let path = root.path().join("AGENTS.md");
        const ORIGINAL: &str = "# Rules\n\nBe kind.\n";
        fs::write(&path, ORIGINAL).expect("write agents");
        install_agents(&root.path().display().to_string()).expect("install");
        let body = fs::read_to_string(&path).expect("agents file");
        assert!(
            body.starts_with(ORIGINAL),
            "existing content was changed:\n{body}"
        );
    }

    #[test]
    fn install_agents_adds_a_newline_before_the_block() {
        let _lock = lock_env();
        let (_guard, _home, root) = codex_home();
        let path = root.path().join("AGENTS.md");
        fs::write(&path, "no trailing newline").expect("write agents");
        install_agents(&root.path().display().to_string()).expect("install");
        let body = fs::read_to_string(&path).expect("agents file");
        assert!(
            body.contains(&format!("no trailing newline\n\n{AGENTS_LINE_BEGIN}")),
            "block must start on its own paragraph:\n{body:?}"
        );
    }

    #[test]
    fn uninstall_agents_keeps_other_content() {
        let _lock = lock_env();
        let (_guard, _home, root) = codex_home();
        let root_path = root.path().display().to_string();
        let path = root.path().join("AGENTS.md");
        const ORIGINAL: &str = "# Rules\n\nBe kind.\n";
        fs::write(&path, ORIGINAL).expect("write agents");
        install_agents(&root_path).expect("install");
        uninstall_agents(&root_path).expect("uninstall");
        let body = fs::read_to_string(&path).expect("agents file");
        assert!(
            !body.contains("polterware") && body.contains("Be kind."),
            "uninstall left the wrong content:\n{body}"
        );
        let state = inspect_agents(&root_path);
        assert!(!state.skill_ready && !state.line_ready, "state = {state:?}");
    }

    #[test]
    fn uninstall_agents_deletes_a_file_that_only_held_the_block() {
        let _lock = lock_env();
        let (_guard, _home, root) = codex_home();
        let root_path = root.path().display().to_string();
        install_agents(&root_path).expect("install");
        uninstall_agents(&root_path).expect("uninstall");
        assert!(
            !root.path().join("AGENTS.md").exists(),
            "AGENTS.md should be removed"
        );
    }

    #[test]
    fn uninstall_agents_without_an_install_is_a_noop() {
        let _lock = lock_env();
        let (_guard, _home, root) = codex_home();
        let path = root.path().join("AGENTS.md");
        fs::write(&path, "keep me\n").expect("write agents");
        uninstall_agents(&root.path().display().to_string()).expect("uninstall");
        assert_eq!(fs::read_to_string(&path).expect("agents file"), "keep me\n");
        uninstall_agents(&root.path().join("missing").display().to_string())
            .expect("a missing root must not fail");
    }

    #[test]
    fn inspect_agents_detects_a_stale_skill() {
        let _lock = lock_env();
        let (_guard, _home, _root) = codex_home();
        install_agents("").expect("install");
        let dir = skill_dir().expect("skill dir");
        fs::write(dir.join("SKILL.md"), "stale").expect("write skill");
        assert!(
            !inspect_agents("").skill_ready,
            "an edited skill file must not count as ready"
        );
    }

    #[test]
    fn agents_line_round_trip() {
        let _lock = lock_env();
        let (_guard, _home, root) = codex_home();
        let root_path = root.path().display().to_string();
        let path = root.path().join("AGENTS.md");
        fs::write(&path, "# Rules\n\nBe kind.\n").expect("write agents");
        install_agents(&root_path).expect("install");
        install_agents(&root_path).expect("install again");
        let body = fs::read_to_string(&path).expect("agents file");
        assert_eq!(
            body.matches(AGENTS_LINE_BEGIN).count(),
            1,
            "line not idempotent:\n{body}"
        );
        let state = inspect_agents(&root_path);
        assert!(state.skill_ready && state.line_ready, "state = {state:?}");
        uninstall_agents(&root_path).expect("uninstall");
        let body = fs::read_to_string(&path).expect("agents file");
        assert_eq!(body.trim(), "# Rules\n\nBe kind.", "user text changed");
        assert!(!inspect_agents(&root_path).skill_ready, "skill file kept");
    }
}
