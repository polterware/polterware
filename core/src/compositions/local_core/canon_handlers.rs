//! Port of `backend/compositions/local-core/canon_handlers.go`.
//!
//! The ProjectModelService, ContextService, and KnowledgeService handlers: the
//! canon entities, relationships, and facts, the compiled context package, the
//! agent tool calls, and the knowledge search and entity links.

use std::sync::Arc;

use crate::compositions::wedge;
use crate::domains::agentaccess;
use crate::domains::agentrun;
use crate::domains::canon;
use crate::domains::context as contextpkg;
use crate::domains::knowledge;
use crate::domains::polter::model::go_quote;
use crate::local_v1;
use crate::providers::sqlite;

use super::audited_write::{change_envelope_from_proto, map_audited_err};
use super::connect::{Code, ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid, knowledge_proto};

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.ProjectModelService/ListEntities",
            list_entities,
        ),
        Route::unary(
            "/polterware.local.v1.ProjectModelService/ListRelationships",
            list_relationships,
        ),
        Route::unary(
            "/polterware.local.v1.ProjectModelService/GetEntity",
            get_entity,
        ),
        Route::unary(
            "/polterware.local.v1.ProjectModelService/SyncProjectModel",
            sync_project_model,
        ),
        Route::unary(
            "/polterware.local.v1.ContextService/CompileContext",
            compile_context,
        ),
        Route::unary(
            "/polterware.local.v1.ContextService/ListAgentToolCalls",
            list_agent_tool_calls,
        ),
        Route::unary(
            "/polterware.local.v1.KnowledgeService/SearchKnowledge",
            search_knowledge,
        ),
        Route::unary(
            "/polterware.local.v1.KnowledgeService/ListKnowledgeLinks",
            list_knowledge_links,
        ),
        Route::unary(
            "/polterware.local.v1.KnowledgeService/SaveKnowledgeLink",
            save_knowledge_link,
        ),
        Route::unary(
            "/polterware.local.v1.KnowledgeService/DeleteKnowledgeLink",
            delete_knowledge_link,
        ),
    ]
}

pub fn list_entities(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListEntitiesRequest>,
) -> Result<local_v1::ListEntitiesResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let items = core
        .store()
        .list_canon_entities(&msg.project_id, &msg.kind)
        .map_err(internal)?;
    Ok(local_v1::ListEntitiesResponse {
        entities: items.iter().map(canon_entity_proto).collect(),
    })
}

pub fn list_relationships(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListRelationshipsRequest>,
) -> Result<local_v1::ListRelationshipsResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let items = core
        .store()
        .list_canon_relationships(&msg.project_id, &msg.entity_id)
        .map_err(internal)?;
    Ok(local_v1::ListRelationshipsResponse {
        relationships: items.iter().map(canon_relationship_proto).collect(),
    })
}

pub fn get_entity(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::GetEntityRequest>,
) -> Result<local_v1::GetEntityResponse, ConnectError> {
    let msg = request.message;
    if msg.id.is_empty() {
        return Err(invalid("entity id is required"));
    }
    let item = match core.store().get_canon_entity(&msg.id) {
        Ok(item) => item,
        Err(err) if err.is_no_rows() => {
            return Err(ConnectError::new(
                Code::NotFound,
                canon::Error::UnknownEntity.to_string(),
            ));
        }
        Err(err) => return Err(internal(err)),
    };
    Ok(local_v1::GetEntityResponse {
        entity: Some(canon_entity_proto(&item)),
    })
}

pub fn sync_project_model(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SyncProjectModelRequest>,
) -> Result<local_v1::SyncProjectModelResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let count = core
        .store()
        .sync_project_model(&msg.project_id)
        .map_err(internal)?;
    Ok(local_v1::SyncProjectModelResponse {
        entity_count: count as i32,
    })
}

pub fn search_knowledge(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SearchKnowledgeRequest>,
) -> Result<local_v1::SearchKnowledgeResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let hits = core
        .store()
        .search_knowledge(&msg.project_id, &msg.query, &msg.kind, msg.limit)
        .map_err(internal)?;
    let mut out = Vec::with_capacity(hits.len());
    for hit in &hits {
        out.push(local_v1::KnowledgeSearchHit {
            item: Some(knowledge_proto(&hit.item)),
            score: hit.score,
            snippet: hit.snippet.clone(),
            truth_class: hit.truth_class.clone(),
            source: hit.source.clone(),
        });
    }
    Ok(local_v1::SearchKnowledgeResponse { hits: out })
}

pub fn list_knowledge_links(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListKnowledgeLinksRequest>,
) -> Result<local_v1::ListKnowledgeLinksResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let items = core
        .store()
        .list_knowledge_links(&msg.project_id, &msg.knowledge_id, &msg.entity_id)
        .map_err(internal)?;
    Ok(local_v1::ListKnowledgeLinksResponse {
        links: items.iter().map(knowledge_link_proto).collect(),
    })
}

pub fn save_knowledge_link(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SaveKnowledgeLinkRequest>,
) -> Result<local_v1::SaveKnowledgeLinkResponse, ConnectError> {
    let msg = request.message;
    let Some(link_msg) = msg.link.as_ref() else {
        return Err(invalid("link is required"));
    };
    let mut env = change_envelope_from_proto(msg.change.as_ref()).map_err(map_audited_err)?;
    if !canon::valid_predicate(&link_msg.predicate) {
        return Err(invalid("invalid predicate"));
    }
    let actor = if link_msg.actor.is_empty() {
        agentrun::ACTOR_HUMAN
    } else {
        link_msg.actor.as_str()
    };
    let source = if link_msg.source.is_empty() {
        agentrun::SOURCE_DESKTOP
    } else {
        link_msg.source.as_str()
    };
    let mut link = knowledge::new_entity_link(
        &link_msg.project_id,
        &link_msg.knowledge_id,
        &link_msg.entity_id,
        &link_msg.predicate,
        &link_msg.truth_class,
        source,
        actor,
    )
    .map_err(|err| invalid(err.to_string()))?;
    if !link_msg.id.is_empty() {
        link.id = link_msg.id.clone();
    }
    env.project_id = link.project_id.clone();
    if core.store().get_canon_entity(&link.entity_id).is_err() {
        return Err(invalid("entity not found"));
    }
    {
        let store = core.store();
        let item = link.clone();
        wedge::execute_audited_write(
            &store,
            env,
            agentrun::ACTOR_KIND_HUMAN,
            "desktop.knowledge.link",
            |_run| {
                store.save_knowledge_link(&item)?;
                Ok((vec![item.id.clone()], item.clone()))
            },
        )
    }
    .map_err(map_audited_err)?;
    Ok(local_v1::SaveKnowledgeLinkResponse {
        link: Some(knowledge_link_proto(&link)),
    })
}

pub fn delete_knowledge_link(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::DeleteKnowledgeLinkRequest>,
) -> Result<local_v1::DeleteKnowledgeLinkResponse, ConnectError> {
    let msg = request.message;
    if msg.id.is_empty() {
        return Err(invalid("link id is required"));
    }
    let mut env = change_envelope_from_proto(msg.change.as_ref()).map_err(map_audited_err)?;
    let existing = match core.store().get_knowledge_link(&msg.id) {
        Ok(link) => link,
        Err(err) if err.is_no_rows() => {
            return Err(ConnectError::new(Code::NotFound, "unknown knowledge link"));
        }
        Err(err) => return Err(internal(err)),
    };
    env.project_id = existing.project_id.clone();
    let id = msg.id.clone();
    {
        let store = core.store();
        wedge::execute_audited_write(
            &store,
            env,
            agentrun::ACTOR_KIND_HUMAN,
            "desktop.knowledge.unlink",
            |_run| {
                store.delete_knowledge_link(&id)?;
                Ok((vec![id.clone()], ()))
            },
        )
    }
    .map_err(map_audited_err)?;
    Ok(local_v1::DeleteKnowledgeLinkResponse {})
}

pub fn compile_context(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::CompileContextRequest>,
) -> Result<local_v1::CompileContextResponse, ConnectError> {
    let ConnectRequest {
        headers,
        message: msg,
    } = request;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let mut pkg = contextpkg::Package::default();
    let mut failure: Option<sqlite::Error> = None;
    {
        let store = core.store();
        let refs = wedge::scope_refs(&msg.scope_node, &msg.scope_nodes);
        let scope = wedge::resolve_scope_refs(&store, &msg.project_id, &refs, msg.scope_exact)
            .map_err(|err| invalid(err.to_string()))?;
        if !msg.agent_run_id.is_empty() {
            // Same path as the CLI and MCP: appends a context step to the run.
            pkg = wedge::compile_context(
                &store,
                wedge::ContextInput {
                    project_id: msg.project_id.clone(),
                    query: msg.query.clone(),
                    entity_id: msg.entity_id.clone(),
                    token_budget: msg.token_budget,
                    agent_run_id: msg.agent_run_id.clone(),
                    scope_node_ids: scope.node_ids.clone(),
                    scope_exact: scope.exact,
                    ..wedge::ContextInput::default()
                },
            )
            .map_err(map_audited_err)?;
        } else {
            match store.compile_context(&contextpkg::Request {
                project_id: msg.project_id.clone(),
                query: msg.query.clone(),
                entity_id: msg.entity_id.clone(),
                token_budget: msg.token_budget,
                scope_node_ids: scope.node_ids.clone(),
                scope_exact: scope.exact,
                ..contextpkg::Request::default()
            }) {
                Ok(compiled) => pkg = compiled,
                Err(err) => failure = Some(err),
            }
        }
        let summary = match &failure {
            None => agentaccess::summarize_result(
                &format!(
                    "items={} tokens={} truncated={}",
                    pkg.items.len(),
                    pkg.tokens_used,
                    pkg.truncated
                ),
                240,
            ),
            Some(err) => agentaccess::summarize_result(&err.to_string(), 240),
        };
        let _ = store.append_agent_tool_call(agentaccess::ToolCall {
            project_id: msg.project_id.clone(),
            agent_run_id: msg.agent_run_id.clone(),
            tool_name: "context.get".to_string(),
            arguments_json: format!(
                r#"{{"query":{},"entity_id":{},"token_budget":{}}}"#,
                go_quote(&msg.query),
                go_quote(&msg.entity_id),
                msg.token_budget
            ),
            result_summary: summary,
            ok: failure.is_none(),
            source: super::lifecycle::client_of(&headers),
            ..agentaccess::ToolCall::default()
        });
    }
    if let Some(err) = failure {
        return Err(internal(err));
    }
    let mut items = Vec::with_capacity(pkg.items.len());
    for item in &pkg.items {
        items.push(local_v1::ContextItem {
            kind: item.kind.clone(),
            id: item.id.clone(),
            title: item.title.clone(),
            excerpt: item.excerpt.clone(),
            truth_class: item.truth_class.clone(),
            source: item.source.clone(),
            path: item.path.clone(),
            reasons: item.reasons.clone(),
        });
    }
    Ok(local_v1::CompileContextResponse {
        project_id: pkg.project_id,
        query: pkg.query,
        entity_id: pkg.entity_id,
        token_budget: pkg.token_budget,
        tokens_used: pkg.tokens_used,
        truncated: pkg.truncated,
        items,
    })
}

pub fn list_agent_tool_calls(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListAgentToolCallsRequest>,
) -> Result<local_v1::ListAgentToolCallsResponse, ConnectError> {
    let calls = core
        .store()
        .list_agent_tool_calls(i64::from(request.message.limit))
        .map_err(internal)?;
    let mut out = Vec::with_capacity(calls.len());
    for call in &calls {
        out.push(local_v1::AgentToolCall {
            id: call.id.clone(),
            project_id: call.project_id.clone(),
            tool_name: call.tool_name.clone(),
            arguments_json: call.arguments_json.clone(),
            result_summary: call.result_summary.clone(),
            ok: call.ok,
            source: call.source.clone(),
            created_at: sqlite::format_rfc3339_zone(call.created_at),
            agent_run_id: call.agent_run_id.clone(),
        });
    }
    Ok(local_v1::ListAgentToolCallsResponse { calls: out })
}

pub fn canon_entity_proto(item: &canon::Entity) -> local_v1::CanonEntity {
    local_v1::CanonEntity {
        id: item.id.clone(),
        project_id: item.project_id.clone(),
        kind: item.kind.clone(),
        ref_kind: item.ref_kind.clone(),
        ref_id: item.ref_id.clone(),
        title: item.title.clone(),
        truth_class: item.truth_class.clone(),
        source: item.source.clone(),
        actor: item.actor.clone(),
        observed_at: sqlite::format_rfc3339_zone(item.observed_at),
        created_at: sqlite::format_rfc3339_zone(item.created_at),
        updated_at: sqlite::format_rfc3339_zone(item.updated_at),
    }
}

pub fn canon_relationship_proto(item: &canon::Relationship) -> local_v1::CanonRelationship {
    local_v1::CanonRelationship {
        id: item.id.clone(),
        project_id: item.project_id.clone(),
        subject_entity_id: item.subject_entity_id.clone(),
        predicate: item.predicate.clone(),
        object_entity_id: item.object_entity_id.clone(),
        truth_class: item.truth_class.clone(),
        source: item.source.clone(),
        actor: item.actor.clone(),
        observed_at: sqlite::format_rfc3339_zone(item.observed_at),
        created_at: sqlite::format_rfc3339_zone(item.created_at),
    }
}

pub fn knowledge_link_proto(item: &knowledge::EntityLink) -> local_v1::KnowledgeEntityLink {
    local_v1::KnowledgeEntityLink {
        id: item.id.clone(),
        project_id: item.project_id.clone(),
        knowledge_id: item.knowledge_id.clone(),
        entity_id: item.entity_id.clone(),
        predicate: item.predicate.clone(),
        truth_class: item.truth_class.clone(),
        source: item.source.clone(),
        actor: item.actor.clone(),
        observed_at: sqlite::format_rfc3339_zone(item.observed_at),
        created_at: sqlite::format_rfc3339_zone(item.created_at),
    }
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderMap;

    use crate::providers::seed;

    use super::*;

    fn request<T>(message: T) -> ConnectRequest<T> {
        ConnectRequest {
            headers: HeaderMap::new(),
            message,
        }
    }

    #[test]
    fn entities_and_context_are_read_from_the_seeded_project() {
        let harness = crate::compositions::local_core::test_support::Harness::start();
        let core = Arc::clone(&harness.core);
        let project_id = seed::PROJECT_ID.to_string();

        let entities = list_entities(
            Arc::clone(&core),
            request(local_v1::ListEntitiesRequest {
                project_id: project_id.clone(),
                ..local_v1::ListEntitiesRequest::default()
            }),
        )
        .expect("entities");
        assert!(
            !entities.entities.is_empty(),
            "the seeded project has canon"
        );

        let sync = sync_project_model(
            Arc::clone(&core),
            request(local_v1::SyncProjectModelRequest {
                project_id: project_id.clone(),
            }),
        )
        .expect("sync");
        assert!(sync.entity_count > 0);

        let compiled = compile_context(
            Arc::clone(&core),
            request(local_v1::CompileContextRequest {
                project_id: project_id.clone(),
                ..local_v1::CompileContextRequest::default()
            }),
        )
        .expect("compile");
        assert_eq!(compiled.project_id, project_id);
        assert!(compiled.token_budget > 0);

        let calls = list_agent_tool_calls(
            Arc::clone(&core),
            request(local_v1::ListAgentToolCallsRequest { limit: 10 }),
        )
        .expect("calls");
        assert!(!calls.calls.is_empty(), "context.get appends a tool call");
        assert_eq!(calls.calls[0].tool_name, "context.get");
        assert!(calls.calls[0].ok);
        assert_eq!(calls.calls[0].source, "desktop");
    }

    #[test]
    fn knowledge_links_need_a_known_entity() {
        let harness = crate::compositions::local_core::test_support::Harness::start();
        let core = Arc::clone(&harness.core);
        let project_id = seed::PROJECT_ID.to_string();
        let entity = list_entities(
            Arc::clone(&core),
            request(local_v1::ListEntitiesRequest {
                project_id: project_id.clone(),
                ..local_v1::ListEntitiesRequest::default()
            }),
        )
        .expect("entities")
        .entities
        .into_iter()
        .next()
        .expect("an entity");
        {
            let store = core.store();
            let doc = knowledge::new_item(
                "doc-1",
                &project_id,
                "A doc",
                "body",
                knowledge::KIND_DOC,
                Vec::new(),
            )
            .expect("doc");
            store.save_knowledge(doc).expect("save doc");
        }

        let err = save_knowledge_link(
            Arc::clone(&core),
            request(local_v1::SaveKnowledgeLinkRequest {
                link: Some(local_v1::KnowledgeEntityLink {
                    project_id: project_id.clone(),
                    knowledge_id: "doc-1".to_string(),
                    entity_id: entity.id.clone(),
                    predicate: "cites".to_string(),
                    ..local_v1::KnowledgeEntityLink::default()
                }),
                change: Some(local_v1::ChangeEnvelope {
                    summary: "Link a doc".to_string(),
                    ..local_v1::ChangeEnvelope::default()
                }),
            }),
        )
        .expect_err("an unknown predicate");
        assert_eq!(err.code, Code::InvalidArgument);
        assert_eq!(err.message, "invalid predicate");

        let saved = save_knowledge_link(
            Arc::clone(&core),
            request(local_v1::SaveKnowledgeLinkRequest {
                link: Some(local_v1::KnowledgeEntityLink {
                    project_id: project_id.clone(),
                    knowledge_id: "doc-1".to_string(),
                    entity_id: entity.id.clone(),
                    predicate: canon::PRED_DOCUMENTS.to_string(),
                    ..local_v1::KnowledgeEntityLink::default()
                }),
                change: Some(local_v1::ChangeEnvelope {
                    summary: "Link a doc".to_string(),
                    ..local_v1::ChangeEnvelope::default()
                }),
            }),
        )
        .expect("save link");
        let link = saved.link.expect("link");
        assert_eq!(link.actor, agentrun::ACTOR_HUMAN);
        assert_eq!(link.source, agentrun::SOURCE_DESKTOP);

        let listed = list_knowledge_links(
            Arc::clone(&core),
            request(local_v1::ListKnowledgeLinksRequest {
                project_id: project_id.clone(),
                knowledge_id: "doc-1".to_string(),
                ..local_v1::ListKnowledgeLinksRequest::default()
            }),
        )
        .expect("links");
        assert_eq!(listed.links.len(), 1);
        assert_eq!(listed.links[0].id, link.id);

        delete_knowledge_link(
            Arc::clone(&core),
            request(local_v1::DeleteKnowledgeLinkRequest {
                id: link.id.clone(),
                change: Some(local_v1::ChangeEnvelope {
                    summary: "Unlink a doc".to_string(),
                    ..local_v1::ChangeEnvelope::default()
                }),
            }),
        )
        .expect("unlink");
        let err = delete_knowledge_link(
            Arc::clone(&core),
            request(local_v1::DeleteKnowledgeLinkRequest {
                id: link.id,
                change: Some(local_v1::ChangeEnvelope {
                    summary: "Unlink a doc".to_string(),
                    ..local_v1::ChangeEnvelope::default()
                }),
            }),
        )
        .expect_err("second unlink");
        assert_eq!(err.code, Code::NotFound);
    }
}
