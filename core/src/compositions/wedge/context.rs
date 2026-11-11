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
}
