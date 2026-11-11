//! Port of `backend/compositions/local-core/saved_filter_handlers.go`.
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
            "/polterware.local.v1.SavedFilterService/ListSavedFilters",
            list_saved_filters,
        ),
        Route::unary(
            "/polterware.local.v1.SavedFilterService/SaveSavedFilter",
            save_saved_filter,
        ),
        Route::unary(
            "/polterware.local.v1.SavedFilterService/DeleteSavedFilter",
            delete_saved_filter,
        ),
    ]
}

/// `polterware.local.v1.SavedFilterService.ListSavedFilters`, not ported yet.
pub fn list_saved_filters(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListSavedFiltersRequest>,
) -> Result<local_v1::ListSavedFiltersResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.SavedFilterService.ListSavedFilters is not implemented",
    ))
}

/// `polterware.local.v1.SavedFilterService.SaveSavedFilter`, not ported yet.
pub fn save_saved_filter(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::SaveSavedFilterRequest>,
) -> Result<local_v1::SaveSavedFilterResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.SavedFilterService.SaveSavedFilter is not implemented",
    ))
}

/// `polterware.local.v1.SavedFilterService.DeleteSavedFilter`, not ported yet.
pub fn delete_saved_filter(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::DeleteSavedFilterRequest>,
) -> Result<local_v1::DeleteSavedFilterResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.SavedFilterService.DeleteSavedFilter is not implemented",
    ))
}
