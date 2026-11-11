//! Port of `backend/compositions/local-core/canon_handlers.go`.
//!
//! The handlers are not ported yet: each answers Connect `unimplemented`, worded as
//! the generated `Unimplemented…Handler` of `localv1connect` words it.

use std::sync::Arc;

use crate::local_v1;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::Core;

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
            "/polterware.local.v1.ProjectModelService/ListFacts",
            list_facts,
        ),
        Route::unary("/polterware.local.v1.ProjectModelService/GetFact", get_fact),
        Route::unary(
            "/polterware.local.v1.ProjectModelService/UpsertFact",
            upsert_fact,
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

/// `polterware.local.v1.ProjectModelService.ListEntities`, not ported yet.
pub fn list_entities(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListEntitiesRequest>,
) -> Result<local_v1::ListEntitiesResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.ProjectModelService.ListEntities is not implemented",
    ))
}

/// `polterware.local.v1.ProjectModelService.ListRelationships`, not ported yet.
pub fn list_relationships(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListRelationshipsRequest>,
) -> Result<local_v1::ListRelationshipsResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.ProjectModelService.ListRelationships is not implemented",
    ))
}

/// `polterware.local.v1.ProjectModelService.GetEntity`, not ported yet.
pub fn get_entity(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::GetEntityRequest>,
) -> Result<local_v1::GetEntityResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.ProjectModelService.GetEntity is not implemented",
    ))
}

/// `polterware.local.v1.ProjectModelService.SyncProjectModel`, not ported yet.
pub fn sync_project_model(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::SyncProjectModelRequest>,
) -> Result<local_v1::SyncProjectModelResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.ProjectModelService.SyncProjectModel is not implemented",
    ))
}

/// `polterware.local.v1.ProjectModelService.ListFacts`, not ported yet.
pub fn list_facts(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListFactsRequest>,
) -> Result<local_v1::ListFactsResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.ProjectModelService.ListFacts is not implemented",
    ))
}

/// `polterware.local.v1.ProjectModelService.GetFact`, not ported yet.
pub fn get_fact(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::GetFactRequest>,
) -> Result<local_v1::GetFactResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.ProjectModelService.GetFact is not implemented",
    ))
}

/// `polterware.local.v1.ProjectModelService.UpsertFact`, not ported yet.
pub fn upsert_fact(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::UpsertFactRequest>,
) -> Result<local_v1::UpsertFactResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.ProjectModelService.UpsertFact is not implemented",
    ))
}

/// `polterware.local.v1.ContextService.CompileContext`, not ported yet.
pub fn compile_context(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::CompileContextRequest>,
) -> Result<local_v1::CompileContextResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.ContextService.CompileContext is not implemented",
    ))
}

/// `polterware.local.v1.ContextService.ListAgentToolCalls`, not ported yet.
pub fn list_agent_tool_calls(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListAgentToolCallsRequest>,
) -> Result<local_v1::ListAgentToolCallsResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.ContextService.ListAgentToolCalls is not implemented",
    ))
}

/// `polterware.local.v1.KnowledgeService.SearchKnowledge`, not ported yet.
pub fn search_knowledge(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::SearchKnowledgeRequest>,
) -> Result<local_v1::SearchKnowledgeResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.KnowledgeService.SearchKnowledge is not implemented",
    ))
}

/// `polterware.local.v1.KnowledgeService.ListKnowledgeLinks`, not ported yet.
pub fn list_knowledge_links(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListKnowledgeLinksRequest>,
) -> Result<local_v1::ListKnowledgeLinksResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.KnowledgeService.ListKnowledgeLinks is not implemented",
    ))
}

/// `polterware.local.v1.KnowledgeService.SaveKnowledgeLink`, not ported yet.
pub fn save_knowledge_link(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::SaveKnowledgeLinkRequest>,
) -> Result<local_v1::SaveKnowledgeLinkResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.KnowledgeService.SaveKnowledgeLink is not implemented",
    ))
}

/// `polterware.local.v1.KnowledgeService.DeleteKnowledgeLink`, not ported yet.
pub fn delete_knowledge_link(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::DeleteKnowledgeLinkRequest>,
) -> Result<local_v1::DeleteKnowledgeLinkResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.KnowledgeService.DeleteKnowledgeLink is not implemented",
    ))
}
