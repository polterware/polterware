//! Port of `backend/compositions/local-core/feature_handlers.go`.
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
            "/polterware.local.v1.WorkService/ReviewFeature",
            review_feature,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/CreateFeature",
            create_feature,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/AdvanceFeature",
            advance_feature,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/VerifyFeature",
            verify_feature,
        ),
    ]
}

/// `polterware.local.v1.WorkService.ReviewFeature`, not ported yet.
pub fn review_feature(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ReviewFeatureRequest>,
) -> Result<local_v1::ReviewFeatureResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.ReviewFeature is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.CreateFeature`, not ported yet.
pub fn create_feature(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::CreateFeatureRequest>,
) -> Result<local_v1::CreateFeatureResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.CreateFeature is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.AdvanceFeature`, not ported yet.
pub fn advance_feature(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::AdvanceFeatureRequest>,
) -> Result<local_v1::AdvanceFeatureResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.AdvanceFeature is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.VerifyFeature`, not ported yet.
pub fn verify_feature(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::VerifyFeatureRequest>,
) -> Result<local_v1::VerifyFeatureResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.VerifyFeature is not implemented",
    ))
}
