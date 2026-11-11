//! Port of `backend/compositions/local-core/archive_handlers.go`.
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
            "/polterware.local.v1.CanvasService/SetCanvasArchived",
            set_canvas_archived,
        ),
        Route::unary(
            "/polterware.local.v1.KnowledgeService/SetKnowledgeArchived",
            set_knowledge_archived,
        ),
    ]
}

/// `polterware.local.v1.CanvasService.SetCanvasArchived`, not ported yet.
pub fn set_canvas_archived(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::SetCanvasArchivedRequest>,
) -> Result<local_v1::SetCanvasArchivedResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.CanvasService.SetCanvasArchived is not implemented",
    ))
}

/// `polterware.local.v1.KnowledgeService.SetKnowledgeArchived`, not ported yet.
pub fn set_knowledge_archived(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::SetKnowledgeArchivedRequest>,
) -> Result<local_v1::SetKnowledgeArchivedResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.KnowledgeService.SetKnowledgeArchived is not implemented",
    ))
}
