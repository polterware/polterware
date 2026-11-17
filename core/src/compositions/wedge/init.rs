//! A new local project and its first component.
//!
//! A new local project: its name and, when it starts from a
//! folder, its first component.

use rand::RngCore;

use crate::domains::agentrun;
use crate::domains::projects;
use crate::providers::sqlite;

use super::path::path_base;
use super::repos::{absolute_dir, repo_name_for};
use super::work::{ApplyInput, apply_polter};
#[allow(unused_imports)]
use super::*;

/// The errors `init.go` returns, one per Go `fmt.Errorf` site plus the store and
/// the Polter write.
#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error(transparent)]
    Store(#[from] sqlite::Error),
    #[error(transparent)]
    Repos(#[from] super::repos::ReposError),
    #[error("{0}")]
    Message(String),
}

impl InitError {
    fn message(text: impl Into<String>) -> InitError {
        InitError::Message(text.into())
    }
}

pub type InitResult<T> = std::result::Result<T, InitError>;

/// Registers a local project in the SQLite store. `dir` is optional: given a
/// folder, the project starts with one component linked to the repository
/// cloned there.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InitInput {
    pub id: String,
    pub name: String,
    pub dir: String,
}

pub fn init_project(store: &sqlite::Store, input: InitInput) -> InitResult<projects::Project> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(InitError::message("name is required"));
    }
    let mut dir = input.dir.trim().to_string();
    if !dir.is_empty() {
        dir = absolute_dir(&dir)?;
    }
    let mut id = input.id.trim().to_string();
    if id.is_empty() {
        id = new_project_id()?;
    }
    let project =
        projects::new_project(&id, name).map_err(|err| InitError::message(err.to_string()))?;
    store.save_project(project.clone())?;
    if store.get_project_settings(&project.id).is_err() {
        store.save_project_settings(projects::Settings {
            project_id: project.id.clone(),
            ..projects::Settings::default()
        })?;
    }
    if !dir.is_empty() {
        link_initial_component(store, &project.id, &dir)?;
    }
    Ok(project)
}

/// Gives a new project its first component, named after the folder and linked
/// to the repository cloned there. A project that already has nodes keeps them.
fn link_initial_component(store: &sqlite::Store, project_id: &str, dir: &str) -> InitResult<()> {
    let snapshot = match store.load_work_snapshot(project_id) {
        Ok(snapshot) => snapshot,
        Err(err) => return Err(InitError::Store(err)),
    };
    if !snapshot.nodes.is_empty() {
        return Ok(());
    }
    let slug = repo_name_for("", dir);
    let text = format!(
        "---\npolter: 1\n---\n\n# {}: {}\nlevel: component\ndir: {dir}\n",
        slug,
        path_base(dir)
    );
    apply_polter(
        store,
        ApplyInput {
            project_id: project_id.to_string(),
            text,
            actor_kind: agentrun::ACTOR_KIND_HUMAN.to_string(),
            tool: "init".to_string(),
            summary: "Link the project folder".to_string(),
            rationale: "The project was created from this folder".to_string(),
            ..ApplyInput::default()
        },
    )
    .map_err(|err| InitError::message(err.to_string()))?;
    Ok(())
}

fn new_project_id() -> InitResult<String> {
    let mut buf = [0u8; 4];
    rand::rng().fill_bytes(&mut buf);
    Ok(format!("proj-{}", hex::encode(buf)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::sqlite::test_support::TempStore;

    #[test]
    fn a_project_needs_a_name() {
        let holder = TempStore::new();
        let err = init_project(
            &holder.store,
            InitInput {
                name: "  ".to_string(),
                ..InitInput::default()
            },
        )
        .expect_err("a project needs a name");
        assert_eq!(err.to_string(), "name is required");
    }

    // The Go test (`TestInitScanContext`) also compiles the project's context;
    // that read belongs to `context.rs`, so this half covers InitProject alone.
    #[test]
    fn init_links_the_folder_as_the_first_component() {
        let holder = TempStore::new();
        let base = tempfile::tempdir().expect("root");
        let root = base.path().join("demo");
        std::fs::create_dir_all(&root).expect("folder");
        let root = root.to_string_lossy().into_owned();

        let project = init_project(
            &holder.store,
            InitInput {
                name: "Demo".to_string(),
                dir: root.clone(),
                ..InitInput::default()
            },
        )
        .expect("project");
        let repos = holder.store.list_repositories(&project.id).expect("repos");
        assert_eq!(
            repos.len(),
            1,
            "init links the folder as the only repository: {repos:?}"
        );
        assert_eq!(repos[0].local_path, root);
        let snapshot = holder
            .store
            .load_work_snapshot(&project.id)
            .expect("snapshot");
        assert_eq!(
            snapshot.nodes.len(),
            1,
            "init creates one component on the repository: {:?}",
            snapshot.nodes
        );
        assert_eq!(snapshot.nodes[0].level, "component");
        assert_eq!(snapshot.nodes[0].repo, repos[0].name);
    }

    #[test]
    fn a_given_id_is_kept_and_a_missing_folder_is_refused() {
        let holder = TempStore::new();
        let project = init_project(
            &holder.store,
            InitInput {
                id: "proj-fixed".to_string(),
                name: "Demo".to_string(),
                ..InitInput::default()
            },
        )
        .expect("project");
        assert_eq!(project.id, "proj-fixed");

        let err = init_project(
            &holder.store,
            InitInput {
                name: "Other".to_string(),
                dir: "/no/such/folder-anywhere".to_string(),
                ..InitInput::default()
            },
        )
        .expect_err("a missing folder is refused");
        assert!(
            err.to_string()
                .starts_with("repository folder /no/such/folder-anywhere does not exist"),
            "{err}"
        );
    }
}
