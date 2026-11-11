//! Port of `backend/compositions/wedge/context.go`.

#[allow(unused_imports)]
use super::*;

use time::OffsetDateTime;

use crate::domains::agentrun;
use crate::domains::context as contextpkg;
use crate::providers::sqlite::{self, Store};

/// Compiles a bounded context package (B3/B4).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextInput {
    pub project_id: String,
    pub query: String,
    pub entity_id: String,
    pub token_budget: i32,
    pub agent_run_id: String,
    /// `scope_node_id` and `scope_node_ids` narrow the package to those
    /// Structure nodes (union). `scope_exact` leaves ancestors out of every
    /// node.
    pub scope_node_id: String,
    pub scope_node_ids: Vec<String>,
    pub scope_exact: bool,
}

/// Compiles context and optionally appends an AgentRun context step.
/// Environment entities are kept in sync by the environment writes.
pub fn compile_context(store: &Store, in_: ContextInput) -> sqlite::Result<contextpkg::Package> {
    if in_.project_id.is_empty() {
        return Err(sqlite::Error::message("project_id is required"));
    }
    let budget = if in_.token_budget == 0 {
        contextpkg::DEFAULT_TOKEN_BUDGET
    } else {
        in_.token_budget
    };
    let pkg = store.compile_context(&contextpkg::Request {
        project_id: in_.project_id.clone(),
        query: in_.query.clone(),
        entity_id: in_.entity_id.clone(),
        token_budget: budget,
        scope_node_id: in_.scope_node_id.clone(),
        scope_node_ids: in_.scope_node_ids.clone(),
        scope_exact: in_.scope_exact,
    })?;
    if !in_.agent_run_id.is_empty() {
        let now = OffsetDateTime::now_utc();
        store.append_agent_run_step_if_open(&in_.agent_run_id, |run: &agentrun::Run| {
            agentrun::new_context_step(&run.id, &pkg, Some(now))
        })?;
    }
    Ok(pkg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    #[test]
    fn compiles_a_package_for_the_project() {
        let TempStore { store, .. } = TempStore::new();
        save_project(&store, "p", OffsetDateTime::UNIX_EPOCH);
        let pkg = compile_context(
            &store,
            ContextInput {
                project_id: "p".to_string(),
                ..ContextInput::default()
            },
        )
        .expect("compile");
        assert_eq!(pkg.project_id, "p");
    }

    #[test]
    fn a_package_needs_a_project() {
        let TempStore { store, .. } = TempStore::new();
        let err = compile_context(&store, ContextInput::default()).unwrap_err();
        assert_eq!(err.to_string(), "project_id is required");
    }

    /// A project started from a folder links it as its only component and
    /// compiles a context package for it.
    #[test]
    fn init_scan_context() {
        use crate::compositions::wedge::init::{InitInput, init_project};

        let root_dir = tempfile::tempdir().expect("root");
        let root = root_dir.path().to_string_lossy().into_owned();
        for (name, body) in [
            ("go.mod", "module github.com/acme/demo\n\ngo 1.23\n"),
            ("README.md", "# Demo\n"),
            (
                "package.json",
                r#"{"name":"demo","dependencies":{"left-pad":"1.0.0"}}"#,
            ),
        ] {
            std::fs::write(root_dir.path().join(name), body).expect("write");
        }

        let TempStore { store, .. } = TempStore::new();
        let project = init_project(
            &store,
            InitInput {
                name: "Demo".to_string(),
                dir: root.clone(),
                ..InitInput::default()
            },
        )
        .expect("init project");

        let repos = store.list_repositories(&project.id).expect("repositories");
        assert_eq!(repos.len(), 1, "repos = {repos:?}");
        assert_eq!(
            repos[0].local_path, root,
            "init links the folder as the only repository"
        );
        let snapshot = store.load_work_snapshot(&project.id).expect("snapshot");
        assert_eq!(snapshot.nodes.len(), 1, "nodes = {:?}", snapshot.nodes);
        assert_eq!(snapshot.nodes[0].level, "component");
        assert_eq!(
            snapshot.nodes[0].repo, repos[0].name,
            "init creates one component on the repository"
        );
        let envs = store.list_environments(&project.id).expect("environments");
        assert!(
            envs.len() == 1 && envs[0].is_default,
            "expected a default local env: {envs:?}"
        );

        let pkg = compile_context(
            &store,
            ContextInput {
                project_id: project.id.clone(),
                ..ContextInput::default()
            },
        )
        .expect("compile");
        assert_eq!(pkg.project_id, project.id, "context project mismatch");
    }
}
