//! The seed project's defaults: environments, settings, and platforms.
//!
//! Gives an empty database the Polterware project itself, so Polterware is
//! managed with its own build, plus one empty project that shows every empty
//! state.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::compositions::wedge;
use crate::domains::polter;
use crate::domains::projects;
use crate::platform::ids;
use crate::providers::sqlite;

use super::knowledge::seed_knowledge;
use super::structure::structure_text;

pub const PROJECT_ID: &str = "proj-polterware";
pub const PROJECT_NAME: &str = "Polterware";

/// A project with no content, to show every empty state.
pub const EMPTY_PROJECT_ID: &str = "proj-empty";

const SEED_LABEL: &str = "seed";

/// The sibling repositories of the Polterware project, by the name they get in
/// `project_repos`.
const CHECKOUTS: [(&str, &str); 2] = [("web", "polterware-web"), ("cloud", "polterware-cloud")];

/// The seed's failures: the store, project init, its Polter writes, and the
/// plain messages its validation adds.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Store(#[from] sqlite::Error),
    #[error(transparent)]
    Init(#[from] wedge::InitError),
    #[error(transparent)]
    Project(#[from] projects::ProjectError),
    #[error(transparent)]
    Apply(#[from] wedge::ApplyFailure),
    /// The seed text and the Polter grammar drifted apart.
    #[error("seed text line {line}: {message}: {error}")]
    Text {
        line: i32,
        message: String,
        error: wedge::ApplyFailure,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Seeds an empty database. The Polterware project is added only when the app
/// can find its own checkout (from the working directory or the binary's
/// location); otherwise only the empty project is created.
pub fn defaults(store: &sqlite::Store) -> Result<()> {
    let count: i64 = store
        .db
        .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
        .map_err(sqlite::Error::from)?;
    if count != 0 {
        return Ok(());
    }
    let root = detect_root();
    if !root.is_empty() {
        polterware(store, &root)?;
    }
    seed_empty_project(store)
}

/// Registers the project at `root`, its sibling checkouts, and the real
/// Polterware content: Decisions, Structure nodes, Rules, Work cards, docs, and
/// facts.
pub fn polterware(store: &sqlite::Store, root: &str) -> Result<()> {
    wedge::init_project(
        store,
        wedge::InitInput {
            id: PROJECT_ID.to_string(),
            name: PROJECT_NAME.to_string(),
            ..wedge::InitInput::default()
        },
    )?;
    seed_settings(store)?;
    save_repository(store, "main", root)?;
    let mut registered: HashMap<&str, bool> = HashMap::new();
    for (name, dir) in CHECKOUTS {
        let checkout = parent_dir(root).join(dir);
        if std::fs::metadata(checkout.join(".git")).is_err() {
            continue;
        }
        let checkout = checkout.to_string_lossy().into_owned();
        save_repository(store, name, &checkout)?;
        registered.insert(name, true);
    }
    let structure = structure_text(
        *registered.get("web").unwrap_or(&false),
        *registered.get("cloud").unwrap_or(&false),
    );
    let texts = [structure.as_str()];
    for text in texts {
        apply(store, text)?;
    }
    seed_knowledge(store)?;
    store.sync_project_model(PROJECT_ID)?;
    Ok(())
}

/// Registers one clone under a fixed name, so the seed text can link components
/// to it.
fn save_repository(store: &sqlite::Store, name: &str, dir: &str) -> Result<()> {
    let repo = projects::new_repository(
        &ids::new_ulid(),
        PROJECT_ID,
        name,
        &wedge::repository_remote(dir),
        dir,
    )?;
    store.save_repository(repo)?;
    Ok(())
}

/// Writes one Polter text and reports the first error diagnostic, which only
/// happens when the seed text and the Polter grammar drift apart.
fn apply(store: &sqlite::Store, text: &str) -> Result<()> {
    let failure = match wedge::apply_polter(
        store,
        wedge::ApplyInput {
            project_id: PROJECT_ID.to_string(),
            text: text.to_string(),
            auto_start_label: SEED_LABEL.to_string(),
            agent: SEED_LABEL.to_string(),
            ..wedge::ApplyInput::default()
        },
    ) {
        Ok(_) => return Ok(()),
        Err(failure) => failure,
    };
    if let Some(diag) = failure
        .result
        .diagnostics
        .iter()
        .find(|diag| diag.severity == polter::SEVERITY_ERROR)
    {
        return Err(Error::Text {
            line: diag.line,
            message: diag.message.clone(),
            error: failure,
        });
    }
    Err(Error::Apply(failure))
}

/// Finds the Polterware checkout the app runs from, or "".
pub fn detect_root() -> String {
    let mut starts: Vec<String> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd.to_string_lossy().into_owned());
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        starts.push(dir.to_string_lossy().into_owned());
    }
    for start in starts {
        let root = find_root(&start);
        if !root.is_empty() {
            return root;
        }
    }
    String::new()
}

/// Walks up from `dir` to the directory that holds the Polterware Rust
/// workspace (a `Cargo.toml` workspace with a `core` crate) and is a Git work
/// tree.
fn find_root(dir: &str) -> String {
    let mut dir = PathBuf::from(dir);
    loop {
        let manifest = dir.join("Cargo.toml");
        if let Ok(body) = std::fs::read_to_string(&manifest)
            && body.contains("[workspace]")
            && dir.join("core").join("Cargo.toml").is_file()
        {
            let found = std::fs::canonicalize(&dir)
                .unwrap_or_else(|_| dir.clone())
                .to_string_lossy()
                .into_owned();
            if git_top_level(&found) == found {
                return found;
            }
            return String::new();
        }
        let Some(parent) = dir.parent().map(Path::to_path_buf) else {
            return String::new();
        };
        if parent == dir {
            return String::new();
        }
        dir = parent;
    }
}

fn git_top_level(dir: &str) -> String {
    let Ok(out) = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
    else {
        return String::new();
    };
    if !out.status.success() {
        return String::new();
    }
    let top = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let Ok(top) = std::fs::canonicalize(&top) else {
        return String::new();
    };
    let Ok(resolved) = std::fs::canonicalize(dir) else {
        return String::new();
    };
    if resolved != top {
        return String::new();
    }
    dir.to_string()
}

/// Creates a project the way a user-registered one starts: empty layout,
/// default settings, and one local environment.
fn seed_empty_project(store: &sqlite::Store) -> Result<()> {
    wedge::init_project(
        store,
        wedge::InitInput {
            id: EMPTY_PROJECT_ID.to_string(),
            name: "Empty project".to_string(),
            ..wedge::InitInput::default()
        },
    )?;
    Ok(())
}

fn seed_settings(store: &sqlite::Store) -> Result<()> {
    let envs = store.list_environments(PROJECT_ID)?;
    let mut settings = projects::Settings {
        project_id: PROJECT_ID.to_string(),
        description: "Project Context and Change Intelligence for AI Coding".to_string(),
        repository: "polterware/polterware".to_string(),
        ..projects::Settings::default()
    };
    if let Some(first) = envs.first() {
        settings.default_environment_id = first.id.clone();
    }
    store.save_project_settings(settings)?;
    Ok(())
}

/// The directory that holds `path`, like Go's `filepath.Dir`.
fn parent_dir(path: &str) -> PathBuf {
    match Path::new(path).parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        Some(_) => PathBuf::from("."),
        None if Path::new(path).is_absolute() => PathBuf::from("/"),
        None => PathBuf::from("."),
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::params;

    use super::*;

    fn new_seeded() -> (sqlite::Store, tempfile::TempDir) {
        let root = detect_root();
        assert!(
            !root.is_empty(),
            "tests must run inside the Polterware checkout"
        );
        let dir = tempfile::tempdir().expect("a temp dir");
        let store = sqlite::Store::new(dir.path().join("seed.db")).expect("open the store");
        defaults(&store).expect("Defaults");
        (store, dir)
    }

    fn count(store: &sqlite::Store, query: &str, args: impl rusqlite::Params) -> i64 {
        store
            .db
            .query_row(query, args, |row| row.get(0))
            .unwrap_or_else(|err| panic!("{query}: {err}"))
    }

    #[test]
    fn defaults_creates_polterware_and_empty_projects() {
        let (store, _dir) = new_seeded();
        let project = store
            .get_project(PROJECT_ID)
            .expect("the Polterware project exists");
        assert_eq!(project.name, "Polterware");
        let repos = store
            .list_repositories(PROJECT_ID)
            .expect("the repositories read");
        let main = repos
            .iter()
            .any(|repo| repo.name == "main" && repo.local_path == detect_root());
        assert!(main, "the checkout is the main repository: {repos:?}");
        store
            .get_project(EMPTY_PROJECT_ID)
            .expect("the empty project exists");
        let cards = count(
            &store,
            "SELECT COUNT(*) FROM work_cards WHERE project_id=?",
            params![EMPTY_PROJECT_ID],
        );
        assert_eq!(cards, 0, "the empty project has {cards} cards");
    }

    #[test]
    fn polterware_content_is_real() {
        let (store, _dir) = new_seeded();
        for stage in ["shape", "plan", "build", "verify", "delivered"] {
            let n = count(
                &store,
                "SELECT COUNT(*) FROM work_cards WHERE project_id=? AND stage=? AND status='active'",
                params![PROJECT_ID, stage],
            );
            assert!(n > 0, "no card in stage {stage:?}");
        }
        // Two project Rules and the five old principles are seeded as active.
        let rules = count(
            &store,
            "SELECT COUNT(*) FROM rules WHERE project_id=?",
            params![PROJECT_ID],
        );
        assert!(rules >= 7, "want at least 7 Rules, got {rules}");
        let active = count(
            &store,
            "SELECT COUNT(*) FROM rules WHERE project_id=? AND status='active'",
            params![PROJECT_ID],
        );
        assert!(active >= 7, "want at least 7 active Rules, got {active}");
        // A Rule with no contexts applies to the whole project.
        let project_wide = count(
            &store,
            "SELECT COUNT(*) FROM rules r WHERE r.project_id=?
		AND NOT EXISTS (SELECT 1 FROM entity_contexts c WHERE c.entity_id=r.id)",
            params![PROJECT_ID],
        );
        assert_eq!(
            project_wide, 7,
            "want 7 project-wide Rules, got {project_wide}"
        );
        let nodes = count(
            &store,
            "SELECT COUNT(*) FROM structure_nodes WHERE project_id=?",
            params![PROJECT_ID],
        );
        assert!(
            nodes >= 10,
            "want a Structure of Polterware, got {nodes} nodes"
        );
        let invariants = count(
            &store,
            "SELECT COUNT(*) FROM work_invariants wi JOIN work_cards wc ON wc.id=wi.card_id WHERE wc.project_id=?",
            params![PROJECT_ID],
        );
        assert!(invariants > 0, "want card invariants");
    }

    #[test]
    fn no_mock_content_remains() {
        let (store, _dir) = new_seeded();
        let connections = count(&store, "SELECT COUNT(*) FROM integration_connections", []);
        assert_eq!(
            connections, 0,
            "integration_connections has {connections} mock rows"
        );
        for word in [
            "example-api",
            "stripe",
            "redis",
            "prod-postgres",
            "payment service",
            "demo",
        ] {
            let pattern = format!("%{}%", word.to_lowercase());
            let knowledge = count(
                &store,
                "SELECT COUNT(*) FROM knowledge_items WHERE lower(title||body||summary) LIKE ?",
                params![pattern],
            );
            assert_eq!(knowledge, 0, "knowledge mentions mock word {word:?}");
            let cards = count(
                &store,
                "SELECT COUNT(*) FROM work_cards WHERE lower(title||intent) LIKE ?",
                params![pattern],
            );
            assert_eq!(cards, 0, "cards mention mock word {word:?}");
        }
    }

    #[test]
    fn defaults_is_idempotent() {
        let (store, _dir) = new_seeded();
        let tables = [
            "projects",
            "environments",
            "knowledge_items",
            "work_cards",
            "work_tasks",
            "structure_nodes",
            "rules",
        ];
        let mut before = HashMap::new();
        for table in tables {
            before.insert(
                table,
                count(&store, &format!("SELECT COUNT(*) FROM {table}"), []),
            );
        }
        defaults(&store).expect("a second Defaults");
        for table in tables {
            let after = count(&store, &format!("SELECT COUNT(*) FROM {table}"), []);
            assert_eq!(
                after, before[table],
                "{table} changed from {} to {after} on a second seed",
                before[table]
            );
        }
    }

    #[test]
    fn polterware_structure_is_seeded() {
        let (store, _dir) = new_seeded();
        let nodes = count(
            &store,
            "SELECT COUNT(*) FROM structure_nodes WHERE project_id=?",
            params![PROJECT_ID],
        );
        assert!(nodes >= 30, "want a real tree, got {nodes} nodes");
    }
}
