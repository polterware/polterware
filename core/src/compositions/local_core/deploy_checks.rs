//! Port of `backend/compositions/local-core/deploy_checks.go`.
//!
//! The background deploy check and the two deploy handlers: what a deploy can
//! link to, and asking the platforms again.

use std::sync::Arc;
use std::time::{Duration, Instant};

use time::OffsetDateTime;

use crate::compositions::wedge;
use crate::local_v1;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid};

/// A first pass shortly after the core starts, then one on this interval while
/// it runs. Each pass checks only links that are due.
const DEPLOY_CHECK_FIRST: Duration = Duration::from_secs(30);
const DEPLOY_CHECK_EVERY: Duration = Duration::from_secs(10 * 60);

/// How often the timer wakes to see whether the core stopped.
const STOP_POLL: Duration = Duration::from_millis(250);

/// Checks every project's deploys on a timer until the core stops. The writes
/// reach clients as change events, and failures raise Inbox items.
pub fn watch_deploys(core: &Arc<Core>) {
    let core = Arc::clone(core);
    std::thread::spawn(move || {
        let mut wait = DEPLOY_CHECK_FIRST;
        loop {
            if !wait_or_stopped(&core, wait) {
                return;
            }
            let _ = wedge::check_all_deploys(&core.store(), OffsetDateTime::now_utc());
            wait = DEPLOY_CHECK_EVERY;
        }
    });
}

/// Sleeps `wait`, answering false when the core stopped first.
fn wait_or_stopped(core: &Core, wait: Duration) -> bool {
    let deadline = Instant::now() + wait;
    loop {
        if *core.done().borrow() {
            return false;
        }
        let now = Instant::now();
        if now >= deadline {
            return true;
        }
        std::thread::sleep((deadline - now).min(STOP_POLL));
    }
}

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.IntegrationService/ListDeployTargets",
            list_deploy_targets,
        ),
        Route::unary(
            "/polterware.local.v1.IntegrationService/CheckDeploys",
            check_deploys,
        ),
    ]
}

pub fn list_deploy_targets(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListDeployTargetsRequest>,
) -> Result<local_v1::ListDeployTargetsResponse, ConnectError> {
    let message = request.message;
    if message.project_id.is_empty() || message.provider.is_empty() {
        return Err(invalid("project id and provider are required"));
    }
    let result = wedge::deploy_targets(
        &core.store(),
        &message.project_id,
        &message.node_id,
        &message.provider,
    )
    .map_err(|err| invalid(err.to_string()))?;
    Ok(local_v1::ListDeployTargetsResponse {
        message: result.message,
        targets: result
            .targets
            .iter()
            .map(|target| local_v1::DeployTarget {
                name: target.name.clone(),
                id: target.id.clone(),
                kind: target.kind.clone(),
                url: target.url.clone(),
                detected: target.detected,
            })
            .collect(),
    })
}

pub fn check_deploys(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::CheckDeploysRequest>,
) -> Result<local_v1::CheckDeploysResponse, ConnectError> {
    let message = request.message;
    if message.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let checked = wedge::check_deploys(
        &core.store(),
        wedge::CheckDeploysInput {
            project_id: message.project_id,
            node_id: message.node_id,
            force: message.force,
            now: None,
        },
    )
    .map_err(internal)?;
    Ok(local_v1::CheckDeploysResponse {
        checked: checked.len() as i32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositions::local_core::test_support::Harness;
    use axum::http::HeaderMap;

    #[test]
    fn the_deploy_handlers_require_their_keys() {
        let harness = Harness::start();
        let error = list_deploy_targets(
            Arc::clone(&harness.core),
            ConnectRequest {
                headers: HeaderMap::new(),
                message: local_v1::ListDeployTargetsRequest {
                    project_id: "proj-1".to_string(),
                    ..local_v1::ListDeployTargetsRequest::default()
                },
            },
        )
        .expect_err("provider is required");
        assert_eq!(error.message, "project id and provider are required");

        let error = check_deploys(
            Arc::clone(&harness.core),
            ConnectRequest {
                headers: HeaderMap::new(),
                message: local_v1::CheckDeploysRequest::default(),
            },
        )
        .expect_err("project is required");
        assert_eq!(error.message, "project id is required");
    }
}
