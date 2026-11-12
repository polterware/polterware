//! AGENTS.md sync: the per-project switch and the background pass.
//!
//! AGENTS.md sync: the switch per project, and the background pass that
//! rewrites a checkout's generated block a moment after a change that can
//! affect it.

use std::collections::HashSet;
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::watch;

use crate::compositions::wedge;
use crate::domains::polter;
use crate::local_v1;
use crate::platform::events;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid};

/// The change kinds that can change an AGENTS.md block: Structure, Decisions,
/// invariants, checkouts, and the sync setting itself.
const AGENTS_SYNC_KINDS: [&str; 4] = [
    events::KIND_WORK,
    events::KIND_KNOWLEDGE,
    events::KIND_SETTINGS,
    events::KIND_PROJECTS,
];

/// The longest the background loop sleeps before checking the stop signal
/// again; the stop channel has no blocking wait of its own.
const STOP_POLL: Duration = Duration::from_millis(10);

pub fn routes() -> Vec<Route> {
    vec![Route::unary(
        "/polterware.local.v1.WorkService/SetAgentsSync",
        set_agents_sync,
    )]
}

/// Turns AGENTS.md sync on or off for a project. Turning it on writes the
/// blocks at once, so the caller sees what changed.
pub fn set_agents_sync(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SetAgentsSyncRequest>,
) -> Result<local_v1::SetAgentsSyncResponse, ConnectError> {
    let project_id = request.message.project_id.clone();
    if project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    if core.store().get_project(&project_id).is_err() {
        return Err(invalid(format!(
            "unknown project {}",
            polter::go_quote(&project_id)
        )));
    }
    let enabled = request.message.enabled;
    core.store()
        .set_agents_sync(&project_id, enabled)
        .map_err(internal)?;
    let mut response = local_v1::SetAgentsSyncResponse {
        enabled,
        files: Vec::new(),
    };
    if enabled {
        let files = wedge::sync_agents_blocks(&core.store(), &project_id)
            .map_err(|err| internal(err.to_string()))?;
        response.files = projection_files_proto(&files);
    }
    Ok(response)
}

/// Rewrites the AGENTS.md blocks of the projects with sync on, `delay` after a
/// change that can affect them, until `stop` closes. Changes that arrive in the
/// meantime join the same pass. Writing files never touches the store, so a
/// pass does not start another. Failures go to `err_out`.
pub fn run_agents_sync<W: Write>(
    core: &Arc<Core>,
    stop: watch::Receiver<bool>,
    delay: Duration,
    err_out: &mut W,
) {
    let subscription = core.bus.subscribe(0);
    let mut pending: HashSet<String> = HashSet::new();
    let mut every_project = false;
    let mut fire: Option<Instant> = None;
    loop {
        if *stop.borrow() {
            return;
        }
        if let Some(deadline) = fire
            && Instant::now() >= deadline
        {
            fire = None;
            sync_agents(core, &pending, every_project, err_out);
            pending.clear();
            every_project = false;
            continue;
        }
        let wait = match fire {
            Some(deadline) => (deadline - Instant::now()).min(STOP_POLL),
            None => STOP_POLL,
        };
        let Some(change) = subscription.recv_timeout(wait) else {
            continue;
        };
        let relevant = change.resync
            || change
                .kinds
                .iter()
                .any(|kind| AGENTS_SYNC_KINDS.contains(&kind.as_str()));
        if !relevant {
            continue;
        }
        if change.resync || change.project_ids.is_empty() {
            every_project = true;
        }
        for id in change.project_ids {
            pending.insert(id);
        }
        if fire.is_none() {
            fire = Some(Instant::now() + delay);
        }
    }
}

fn sync_agents<W: Write>(
    core: &Core,
    pending: &HashSet<String>,
    every_project: bool,
    err_out: &mut W,
) {
    let ids = match core.store().agents_sync_projects() {
        Ok(ids) => ids,
        Err(err) => {
            let _ = writeln!(err_out, "agents sync: {err}");
            return;
        }
    };
    for id in ids {
        if !every_project && !pending.contains(&id) {
            continue;
        }
        if let Err(err) = wedge::sync_agents_blocks(&core.store(), &id) {
            let _ = writeln!(err_out, "agents sync {id}: {err}");
        }
    }
}

fn projection_files_proto(files: &[wedge::ProjectionFile]) -> Vec<local_v1::ProjectionFile> {
    files
        .iter()
        .map(|file| local_v1::ProjectionFile {
            repo: file.repo.clone(),
            path: file.path.clone(),
            action: file.action.clone(),
            reason: file.reason.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositions::local_core::test_support::{Harness, init_with_repo};
    use std::sync::Mutex;

    /// A writer the test reads after the background pass wrote to it.
    struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

    impl Write for SharedBuffer {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn agents_sync_rewrites_the_block_after_changes() {
        let harness = Harness::start();
        let root = tempfile::tempdir().expect("temp dir");
        let root_path = root.path().to_string_lossy().into_owned();
        let project = {
            let store = harness.core.store();
            init_with_repo(&store, "Shop", &root_path).expect("project")
        };
        let agents = std::path::Path::new(&root_path).join("AGENTS.md");
        let read = || std::fs::read_to_string(&agents).unwrap_or_default();
        let apply = |text: &str| {
            let store = harness.core.store();
            crate::compositions::wedge::apply_polter(
                &store,
                crate::compositions::wedge::ApplyInput {
                    project_id: project.id.clone(),
                    auto_start_label: "cli".to_string(),
                    text: format!(
                        "---\npolter: 1\nsummary: Map the shop\nrationale: Agents read the block\n---\n\n{text}"
                    ),
                    ..crate::compositions::wedge::ApplyInput::default()
                },
            )
            .expect("apply");
        };

        let (stop, stopped) = watch::channel(false);
        let errors = Arc::new(Mutex::new(Vec::new()));
        let writer = SharedBuffer(Arc::clone(&errors));
        let core = Arc::clone(&harness.core);
        let handle = std::thread::spawn(move || {
            let mut writer = writer;
            run_agents_sync(&core, stopped, Duration::from_millis(10), &mut writer);
        });
        let deadline = Instant::now() + Duration::from_secs(1);
        while harness.core.bus.subscribers() == 0 && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }

        apply("# shop: Shop\nlevel: domain\n\n! Money is integer cents\n");
        std::thread::sleep(Duration::from_millis(300));
        assert!(
            read().is_empty(),
            "sync is off by default, yet AGENTS.md was written:\n{}",
            read()
        );

        let on = set_agents_sync(
            Arc::clone(&harness.core),
            ConnectRequest {
                headers: axum::http::HeaderMap::new(),
                message: local_v1::SetAgentsSyncRequest {
                    project_id: project.id.clone(),
                    enabled: true,
                },
            },
        )
        .expect("turn sync on");
        assert_eq!(
            on.files.len(),
            1,
            "turning sync on writes the block at once"
        );
        assert_eq!(on.files[0].path, "AGENTS.md");
        assert_eq!(on.files[0].action, wedge::PROJECTION_WRITE);
        assert!(
            read().contains("Money is integer cents (from shop)"),
            "AGENTS.md:\n{}",
            read()
        );
        assert!(
            harness
                .core
                .store()
                .get_project_settings(&project.id)
                .expect("settings")
                .agents_sync
        );

        apply(
            "# shop: Shop\nlevel: domain\n\n! Money is integer cents\n! Refunds keep the original currency\n",
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        while !read().contains("Refunds keep the original currency (from shop)")
            && Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(
            read().contains("Refunds keep the original currency (from shop)"),
            "AGENTS.md never got the new rule:\n{}",
            read()
        );
        assert!(
            !std::path::Path::new(&root_path)
                .join("docs/decision")
                .exists(),
            "sync writes only AGENTS.md"
        );

        set_agents_sync(
            Arc::clone(&harness.core),
            ConnectRequest {
                headers: axum::http::HeaderMap::new(),
                message: local_v1::SetAgentsSyncRequest {
                    project_id: project.id.clone(),
                    enabled: false,
                },
            },
        )
        .expect("turn sync off");
        apply(
            "# shop: Shop\nlevel: domain\n\n! Money is integer cents\n! Refunds keep the original currency\n! Prices include tax\n",
        );
        std::thread::sleep(Duration::from_millis(300));
        assert!(
            !read().contains("Prices include tax"),
            "sync off must leave AGENTS.md alone:\n{}",
            read()
        );

        let _ = stop.send(true);
        handle.join().expect("background pass");
        let errors = errors.lock().unwrap_or_else(|poison| poison.into_inner());
        assert!(
            errors.is_empty(),
            "sync errors: {}",
            String::from_utf8_lossy(&errors)
        );
    }

    #[test]
    fn the_switch_requires_a_known_project() {
        let harness = Harness::start();
        let error = set_agents_sync(
            Arc::clone(&harness.core),
            ConnectRequest {
                headers: axum::http::HeaderMap::new(),
                message: local_v1::SetAgentsSyncRequest::default(),
            },
        )
        .expect_err("project id is required");
        assert_eq!(error.message, "project id is required");

        let error = set_agents_sync(
            Arc::clone(&harness.core),
            ConnectRequest {
                headers: axum::http::HeaderMap::new(),
                message: local_v1::SetAgentsSyncRequest {
                    project_id: "nope".to_string(),
                    enabled: true,
                },
            },
        )
        .expect_err("unknown project");
        assert_eq!(error.message, "unknown project \"nope\"");
    }
}
