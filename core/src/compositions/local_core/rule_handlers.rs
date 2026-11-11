//! Port of `backend/compositions/local-core/rule_handlers.go`.
//!
//! The handlers are not ported yet: each answers Connect `unimplemented`, worded as
//! the generated `Unimplemented…Handler` of `localv1connect` words it.

use std::sync::Arc;

use crate::local_v1;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::Core;

pub fn routes() -> Vec<Route> {
    vec![Route::unary(
        "/polterware.local.v1.WorkService/ListRules",
        list_rules,
    )]
}

/// `polterware.local.v1.WorkService.ListRules`, not ported yet.
pub fn list_rules(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListRulesRequest>,
) -> Result<local_v1::ListRulesResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.ListRules is not implemented",
    ))
}
