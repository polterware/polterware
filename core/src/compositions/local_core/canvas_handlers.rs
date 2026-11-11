//! Port of `backend/compositions/local-core/canvas_handlers.go`.
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
            "/polterware.local.v1.CanvasService/ListCanvases",
            list_canvases,
        ),
        Route::unary("/polterware.local.v1.CanvasService/GetCanvas", get_canvas),
        Route::unary(
            "/polterware.local.v1.CanvasService/CreateCanvas",
            create_canvas,
        ),
        Route::unary(
            "/polterware.local.v1.CanvasService/UpdateCanvas",
            update_canvas,
        ),
        Route::unary(
            "/polterware.local.v1.CanvasService/DeleteCanvas",
            delete_canvas,
        ),
        Route::unary(
            "/polterware.local.v1.CanvasService/ListCanvasTypes",
            list_canvas_types,
        ),
    ]
}

/// `polterware.local.v1.CanvasService.ListCanvases`, not ported yet.
pub fn list_canvases(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListCanvasesRequest>,
) -> Result<local_v1::ListCanvasesResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.CanvasService.ListCanvases is not implemented",
    ))
}

/// `polterware.local.v1.CanvasService.GetCanvas`, not ported yet.
pub fn get_canvas(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::GetCanvasRequest>,
) -> Result<local_v1::GetCanvasResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.CanvasService.GetCanvas is not implemented",
    ))
}

/// `polterware.local.v1.CanvasService.CreateCanvas`, not ported yet.
pub fn create_canvas(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::CreateCanvasRequest>,
) -> Result<local_v1::CreateCanvasResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.CanvasService.CreateCanvas is not implemented",
    ))
}

/// `polterware.local.v1.CanvasService.UpdateCanvas`, not ported yet.
pub fn update_canvas(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::UpdateCanvasRequest>,
) -> Result<local_v1::UpdateCanvasResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.CanvasService.UpdateCanvas is not implemented",
    ))
}

/// `polterware.local.v1.CanvasService.DeleteCanvas`, not ported yet.
pub fn delete_canvas(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::DeleteCanvasRequest>,
) -> Result<local_v1::DeleteCanvasResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.CanvasService.DeleteCanvas is not implemented",
    ))
}

/// `polterware.local.v1.CanvasService.ListCanvasTypes`, not ported yet.
pub fn list_canvas_types(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListCanvasTypesRequest>,
) -> Result<local_v1::ListCanvasTypesResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.CanvasService.ListCanvasTypes is not implemented",
    ))
}
