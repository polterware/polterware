//! Port of `backend/compositions/local-core/work_handlers.go`.
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
            "/polterware.local.v1.WorkService/ListWorkCards",
            list_work_cards,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/ListStructure",
            list_structure,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/ListStructureHistory",
            list_structure_history,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/ListAgentSessions",
            list_agent_sessions,
        ),
        Route::unary("/polterware.local.v1.WorkService/ApplyPolter", apply_polter),
        Route::unary("/polterware.local.v1.WorkService/LintPolter", lint_polter),
        Route::unary("/polterware.local.v1.WorkService/QueryPolter", query_polter),
        Route::unary(
            "/polterware.local.v1.WorkService/WriteHandoff",
            write_handoff,
        ),
        Route::unary("/polterware.local.v1.WorkService/EndSession", end_session),
        Route::unary(
            "/polterware.local.v1.WorkService/ExportProjections",
            export_projections,
        ),
        Route::unary("/polterware.local.v1.WorkService/FileContext", file_context),
        Route::unary(
            "/polterware.local.v1.WorkService/ListMilestones",
            list_milestones,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/CreateMilestone",
            create_milestone,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/UpdateMilestone",
            update_milestone,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/ArchiveMilestone",
            archive_milestone,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/ReorderMilestones",
            reorder_milestones,
        ),
    ]
}

/// `polterware.local.v1.WorkService.ListWorkCards`, not ported yet.
pub fn list_work_cards(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListWorkCardsRequest>,
) -> Result<local_v1::ListWorkCardsResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.ListWorkCards is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.ListStructure`, not ported yet.
pub fn list_structure(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListStructureRequest>,
) -> Result<local_v1::ListStructureResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.ListStructure is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.ListStructureHistory`, not ported yet.
pub fn list_structure_history(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListStructureHistoryRequest>,
) -> Result<local_v1::ListStructureHistoryResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.ListStructureHistory is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.ListAgentSessions`, not ported yet.
pub fn list_agent_sessions(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListAgentSessionsRequest>,
) -> Result<local_v1::ListAgentSessionsResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.ListAgentSessions is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.ApplyPolter`, not ported yet.
pub fn apply_polter(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ApplyPolterRequest>,
) -> Result<local_v1::ApplyPolterResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.ApplyPolter is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.LintPolter`, not ported yet.
pub fn lint_polter(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::LintPolterRequest>,
) -> Result<local_v1::LintPolterResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.LintPolter is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.QueryPolter`, not ported yet.
pub fn query_polter(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::QueryPolterRequest>,
) -> Result<local_v1::QueryPolterResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.QueryPolter is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.WriteHandoff`, not ported yet.
pub fn write_handoff(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::WriteHandoffRequest>,
) -> Result<local_v1::WriteHandoffResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.WriteHandoff is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.EndSession`, not ported yet.
pub fn end_session(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::EndSessionRequest>,
) -> Result<local_v1::EndSessionResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.EndSession is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.ExportProjections`, not ported yet.
pub fn export_projections(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ExportProjectionsRequest>,
) -> Result<local_v1::ExportProjectionsResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.ExportProjections is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.FileContext`, not ported yet.
pub fn file_context(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::FileContextRequest>,
) -> Result<local_v1::FileContextResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.FileContext is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.ListMilestones`, not ported yet.
pub fn list_milestones(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListMilestonesRequest>,
) -> Result<local_v1::ListMilestonesResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.ListMilestones is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.CreateMilestone`, not ported yet.
pub fn create_milestone(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::CreateMilestoneRequest>,
) -> Result<local_v1::CreateMilestoneResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.CreateMilestone is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.UpdateMilestone`, not ported yet.
pub fn update_milestone(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::UpdateMilestoneRequest>,
) -> Result<local_v1::UpdateMilestoneResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.UpdateMilestone is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.ArchiveMilestone`, not ported yet.
pub fn archive_milestone(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ArchiveMilestoneRequest>,
) -> Result<local_v1::ArchiveMilestoneResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.ArchiveMilestone is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.ReorderMilestones`, not ported yet.
pub fn reorder_milestones(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ReorderMilestonesRequest>,
) -> Result<local_v1::ReorderMilestonesResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.ReorderMilestones is not implemented",
    ))
}
