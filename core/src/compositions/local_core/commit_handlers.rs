//! Port of `backend/compositions/local-core/commit_handlers.go`.
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
            "/polterware.local.v1.WorkService/ListCardCommits",
            list_card_commits,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/LinkTaskCommit",
            link_task_commit,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/UnlinkTaskCommit",
            unlink_task_commit,
        ),
    ]
}

/// `polterware.local.v1.WorkService.ListCardCommits`, not ported yet.
pub fn list_card_commits(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListCardCommitsRequest>,
) -> Result<local_v1::ListCardCommitsResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.ListCardCommits is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.LinkTaskCommit`, not ported yet.
pub fn link_task_commit(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::LinkTaskCommitRequest>,
) -> Result<local_v1::LinkTaskCommitResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.LinkTaskCommit is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.UnlinkTaskCommit`, not ported yet.
pub fn unlink_task_commit(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::UnlinkTaskCommitRequest>,
) -> Result<local_v1::UnlinkTaskCommitResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.UnlinkTaskCommit is not implemented",
    ))
}
