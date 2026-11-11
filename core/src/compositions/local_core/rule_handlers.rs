//! Port of `backend/compositions/local-core/rule_handlers.go`.
//!
//! Rules are the law agents read: statements the code must keep, each holding
//! for the Structure nodes it names and everything under them, or for the whole
//! project.

use std::sync::Arc;

use crate::compositions::wedge;
use crate::local_v1;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid};

pub fn routes() -> Vec<Route> {
    vec![Route::unary(
        "/polterware.local.v1.WorkService/ListRules",
        list_rules,
    )]
}

/// `polterware.local.v1.WorkService.ListRules`: returns the project's active
/// Rules; a scope keeps the ones with a context inside it plus the
/// project-wide ones.
pub fn list_rules(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListRulesRequest>,
) -> Result<local_v1::ListRulesResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let store = core.store();
    let rules = store.list_rules(&msg.project_id).map_err(internal)?;
    let scope = wedge::load_node_scopes(&store, &msg.project_id, &msg.scope_nodes, msg.scope_exact)
        .map_err(|err| invalid(err.to_string()))?;
    let ids = store.scope_map(&msg.project_id, "rule").map_err(internal)?;
    let mut out = local_v1::ListRulesResponse::default();
    for rule in &rules {
        let node_ids = ids.get(&rule.id).cloned().unwrap_or_default();
        if !scope.matches_any(&node_ids) {
            continue;
        }
        out.rules.push(local_v1::Rule {
            id: rule.id.clone(),
            slug: rule.slug.clone(),
            statement: rule.statement.clone(),
            scope_node_ids: node_ids,
            scope_paths: rule.scopes.clone(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::HeaderMap;

    use super::super::connect::Code;
    use super::super::scope_handlers::set_scope;
    use super::*;

    fn request<T>(message: T) -> ConnectRequest<T> {
        ConnectRequest {
            headers: HeaderMap::new(),
            message,
        }
    }

    fn core_with_rule() -> (Arc<Core>, String) {
        let harness = crate::compositions::local_core::test_support::Harness::start();
        let core = Arc::clone(&harness.core);
        let project = {
            let store = core.store();
            let project = wedge::init_project(
                &store,
                wedge::InitInput {
                    name: "Demo".to_string(),
                    ..wedge::InitInput::default()
                },
            )
            .expect("init project");
            wedge::apply_polter(
                &store,
                wedge::ApplyInput {
                    project_id: project.id.clone(),
                    text: "---\npolter: 1\nsummary: Add auth\nrationale: Rules for auth\n---\n\n# auth: Auth\nlevel: domain\n\n#### rule no-secrets: Never commit secrets.\nscope: auth\n".to_string(),
                    auto_start_label: "cli".to_string(),
                    ..wedge::ApplyInput::default()
                },
            )
            .expect("apply rule");
            project
        };
        (core, project.id)
    }

    #[test]
    fn rules_list_with_their_scope() {
        let (core, project_id) = core_with_rule();
        assert_eq!(
            list_rules(
                Arc::clone(&core),
                request(local_v1::ListRulesRequest::default()),
            )
            .unwrap_err()
            .code,
            Code::InvalidArgument
        );
        let listed = list_rules(
            Arc::clone(&core),
            request(local_v1::ListRulesRequest {
                project_id: project_id.clone(),
                ..local_v1::ListRulesRequest::default()
            }),
        )
        .expect("list rules");
        assert_eq!(listed.rules.len(), 1, "rules = {listed:?}");
        let rule = &listed.rules[0];
        assert_eq!(rule.slug, "no-secrets");
        assert_eq!(rule.statement, "Never commit secrets.");
        assert_eq!(rule.scope_paths, vec!["auth"], "rule = {rule:?}");
        assert_eq!(rule.scope_node_ids.len(), 1, "rule = {rule:?}");

        let node_id = rule.scope_node_ids[0].clone();
        let unchanged = list_rules(
            Arc::clone(&core),
            request(local_v1::ListRulesRequest {
                project_id: project_id.clone(),
                scope_nodes: vec!["auth".to_string()],
                ..local_v1::ListRulesRequest::default()
            }),
        )
        .expect("list scoped rules");
        assert_eq!(unchanged.rules.len(), 1, "scoped rules = {unchanged:?}");
        let outside = list_rules(
            Arc::clone(&core),
            request(local_v1::ListRulesRequest {
                project_id: project_id.clone(),
                scope_nodes: vec!["nope".to_string()],
                ..local_v1::ListRulesRequest::default()
            }),
        )
        .unwrap_err();
        assert_eq!(outside.code, Code::InvalidArgument, "{outside:?}");

        // A Rule is named by its slug as often as by its id, and none clears
        // its contexts.
        set_scope(
            Arc::clone(&core),
            request(local_v1::SetScopeRequest {
                project_id: project_id.clone(),
                kind: "rule".to_string(),
                id: "no-secrets".to_string(),
                nodes: vec![node_id.clone()],
            }),
        )
        .expect("set scope by slug");
        set_scope(
            Arc::clone(&core),
            request(local_v1::SetScopeRequest {
                project_id: project_id.clone(),
                kind: "rule".to_string(),
                id: "no-secrets".to_string(),
                nodes: Vec::new(),
            }),
        )
        .expect("clear scope");
        let cleared = list_rules(
            Arc::clone(&core),
            request(local_v1::ListRulesRequest {
                project_id: project_id.clone(),
                ..local_v1::ListRulesRequest::default()
            }),
        )
        .expect("list rules");
        assert!(
            cleared.rules[0].scope_node_ids.is_empty(),
            "cleared rule = {:?}",
            cleared.rules[0]
        );
        let refused = set_scope(
            Arc::clone(&core),
            request(local_v1::SetScopeRequest {
                project_id: project_id.clone(),
                kind: "bogus".to_string(),
                id: "x".to_string(),
                nodes: Vec::new(),
            }),
        )
        .unwrap_err();
        assert_eq!(refused.code, Code::InvalidArgument);
        assert_eq!(
            refused.message,
            "unknown scope kind \"bogus\": use decision, knowledge, fact, rule, or item"
        );
        let missing = set_scope(
            Arc::clone(&core),
            request(local_v1::SetScopeRequest {
                project_id,
                kind: "rule".to_string(),
                id: " ".to_string(),
                nodes: Vec::new(),
            }),
        )
        .unwrap_err();
        assert_eq!(missing.message, "id is required");
    }
}
