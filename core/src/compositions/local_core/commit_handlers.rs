//! Port of `backend/compositions/local-core/commit_handlers.go`.
//!
//! A card's commits: the ones linked to its tasks by hand, and the ones whose
//! subject cites a task (`slug#T2`) in a checkout's recent history.

use std::sync::Arc;

use crate::compositions::wedge;
use crate::local_v1;
use crate::providers::sqlite::format_rfc3339;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid};

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

/// `polterware.local.v1.WorkService.ListCardCommits`: lists a card's commits,
/// linked ones, then ones cited in a checkout's recent subjects.
pub fn list_card_commits(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListCardCommitsRequest>,
) -> Result<local_v1::ListCardCommitsResponse, ConnectError> {
    if request.message.project_id.is_empty() || request.message.slug.is_empty() {
        return Err(invalid("project id and slug are required"));
    }
    let store = core.store();
    let commits = wedge::card_commits(&store, &request.message.project_id, &request.message.slug)
        .map_err(|err| commit_err(&err))?;
    let mut out = Vec::with_capacity(commits.len());
    for commit in &commits {
        out.push(card_commit_proto(commit));
    }
    Ok(local_v1::ListCardCommitsResponse { commits: out })
}

/// `polterware.local.v1.WorkService.LinkTaskCommit`: links a commit to a task
/// given as slug#T2.
pub fn link_task_commit(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::LinkTaskCommitRequest>,
) -> Result<local_v1::LinkTaskCommitResponse, ConnectError> {
    if request.message.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let store = core.store();
    let commit = wedge::link_task_commit(
        &store,
        &request.message.project_id,
        &request.message.task,
        &request.message.sha,
    )
    .map_err(|err| commit_err(&err))?;
    Ok(local_v1::LinkTaskCommitResponse {
        commit: Some(card_commit_proto(&commit)),
    })
}

/// `polterware.local.v1.WorkService.UnlinkTaskCommit`: removes a stored commit
/// link.
pub fn unlink_task_commit(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::UnlinkTaskCommitRequest>,
) -> Result<local_v1::UnlinkTaskCommitResponse, ConnectError> {
    if request.message.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let store = core.store();
    wedge::unlink_task_commit(
        &store,
        &request.message.project_id,
        &request.message.task,
        &request.message.sha,
    )
    .map_err(|err| commit_err(&err))?;
    Ok(local_v1::UnlinkTaskCommitResponse::default())
}

/// A caller mistake is InvalidArgument; anything else is Internal.
fn commit_err(err: &anyhow::Error) -> ConnectError {
    if err.downcast_ref::<wedge::CommitInputError>().is_some()
        || err.to_string() == "unknown project"
    {
        return invalid(err.to_string());
    }
    internal(err.to_string())
}

fn card_commit_proto(commit: &wedge::CardCommit) -> local_v1::CardCommit {
    let committed_at = if commit.committed_at == time::OffsetDateTime::UNIX_EPOCH {
        String::new()
    } else {
        format_rfc3339(commit.committed_at)
    };
    local_v1::CardCommit {
        task_number: commit.task_number,
        sha: commit.sha.clone(),
        short_sha: commit.short_sha().to_string(),
        subject: commit.subject.clone(),
        url: commit.url.clone(),
        committed_at,
        repo: commit.repo.clone(),
        cited: commit.cited,
    }
}
