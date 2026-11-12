//! The detached core daemon.
//!
//! `polterware serve` is the only process that opens the store (Decision 0007).
//! It takes the per-database lock, listens on loopback, writes `core.json` with
//! the endpoint, token, and build id the Desktop reads, starts the deploy
//! watcher (`serve_connect`) and the AGENTS.md sync, and exits on a signal, on
//! `MaintenanceService.Shutdown`, or after `--idle` without clients.

use std::sync::Arc;
use std::time::Duration;

use time::OffsetDateTime;
use tokio::signal::unix::{SignalKind, signal};

use crate::compositions::local_core::server::{listen, serve_connect};
use crate::compositions::local_core::{self, Core};
use crate::platform::coreclient;
use crate::platform::runstate;
use crate::providers::sqlite::Store;

use super::db::clean;
use super::flags::{FlagSet, format_go_duration};
use super::output::{CliError, CliResult};

/// How long the core waits without requests or open change streams before it
/// exits. Zero keeps the core running until it is stopped (Decision 0007);
/// `--idle` or `POLTERWARE_CORE_IDLE` opt in to idle exit.
pub const DEFAULT_IDLE: i64 = 0;

/// The exit code of a second core on one database.
const EXIT_LOCKED: i32 = 3;

/// Lets a burst of writes land before AGENTS.md is rewritten.
const AGENTS_SYNC_DELAY: Duration = Duration::from_secs(2);

/// How often the daemon looks at the idle clock, as Go's ticker does.
const IDLE_TICK: Duration = Duration::from_secs(30);

/// Runs the core: the only process that opens the store. The Desktop, CLI and
/// MCP bridge attach to it through `core.json`.
pub fn run_serve(db_path: &str, args: &[String]) -> CliResult {
    let mut flags = FlagSet::new("serve");
    flags.string_flag("listen", "127.0.0.1:0", "loopback listen address");
    flags.string_flag(
        "token",
        &std::env::var("POLTERWARE_SESSION_TOKEN").unwrap_or_default(),
        "session bearer token (default: minted)",
    );
    flags.duration_flag(
        "idle",
        idle_from_env(),
        "exit after this long without clients (0, the default, keeps running)",
    );
    flags.parse(args).map_err(flag_error)?;

    let address = flags.string_value("listen").to_string();
    let session_token = flags.string_value("token").to_string();
    let idle_nanos = flags.duration_value("idle");

    let lock = match runstate::lock_core(db_path) {
        Ok(lock) => lock,
        Err(runstate::Error::CoreLocked) => {
            match runstate::read_core(db_path) {
                Some(existing) => eprintln!(
                    "polterware core already running for this database (pid {}, {})",
                    existing.pid, existing.endpoint
                ),
                None => eprintln!("polterware core already running for this database"),
            }
            return Err(CliError::Exit(EXIT_LOCKED));
        }
        Err(err) => {
            return Err(CliError::message(format!("core lock: {err}")));
        }
    };

    let store = Store::new(db_path).map_err(|err| CliError::message(format!("db open: {err}")))?;

    let token = if session_token.is_empty() {
        local_core::mint_token()
    } else {
        session_token
    };
    let (listener, endpoint) =
        listen(&address).map_err(|err| CliError::message(format!("listen: {err}")))?;
    eprintln!("polterware core listening on {endpoint} with session-token authentication");

    let core = Core::new(store, &token);
    core.set_db_dir(&parent_dir(db_path));
    if let Ok(exe) = coreclient::executable() {
        core.set_build(&coreclient::build_id(&exe));
    }

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|err| CliError::message(format!("runtime: {err}")))?;
    let server = {
        let _guard = runtime.enter();
        serve_connect(listener, Arc::clone(&core))
    };
    {
        let core = Arc::clone(&core);
        let mut err_out = std::io::stderr();
        std::thread::spawn(move || {
            local_core::agents_sync::run_agents_sync(
                &core,
                core.done(),
                AGENTS_SYNC_DELAY,
                &mut err_out,
            );
        });
    }

    let info = runstate::Core {
        pid: std::process::id() as i32,
        endpoint: endpoint.clone(),
        token: token.clone(),
        db_path: db_path.to_string(),
        build: core.build(),
        started_at: OffsetDateTime::now_utc(),
    };
    if let Err(err) = runstate::write_json(runstate::core_path(db_path), &info) {
        eprintln!("run state: {err}");
    }

    let idle = (idle_nanos > 0).then(|| Duration::from_nanos(idle_nanos.unsigned_abs()));
    runtime.block_on(async {
        let mut done = core.done();
        let mut sigint = signal(SignalKind::interrupt()).expect("SIGINT handler");
        let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM handler");
        let mut sighup = signal(SignalKind::hangup()).expect("SIGHUP handler");
        let mut ticker = tokio::time::interval_at(
            tokio::time::Instant::now() + IDLE_TICK,
            IDLE_TICK,
        );
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                _ = done.changed() => break,
                _ = sigint.recv() => break,
                _ = sigterm.recv() => break,
                _ = sighup.recv() => break,
                _ = ticker.tick() => {
                    if let Some(after) = idle
                        && core.idle(after)
                    {
                        eprintln!("polterware core idle for {}; exiting", format_go_duration(after));
                        break;
                    }
                }
            }
        }
    });

    core.stop();
    core.mcp.close_all();
    server.close();
    runstate::remove_core(db_path, std::process::id() as i32);
    drop(lock);
    Ok(())
}

/// The `--idle` default: `POLTERWARE_CORE_IDLE` when it parses, else zero.
pub fn idle_from_env() -> i64 {
    match std::env::var("POLTERWARE_CORE_IDLE") {
        Ok(value) if !value.is_empty() => {
            super::flags::parse_go_duration(&value).unwrap_or(DEFAULT_IDLE)
        }
        _ => DEFAULT_IDLE,
    }
}

/// The directory that holds the database, as `filepath.Dir` reads it.
fn parent_dir(db_path: &str) -> String {
    match db_path.rsplit_once('/') {
        Some((parent, _)) if !parent.is_empty() => clean(parent),
        Some(_) => "/".to_string(),
        None => ".".to_string(),
    }
}

/// A flag parse error as the command's error: Go's text, already printed with
/// the usage.
fn flag_error(err: super::flags::FlagError) -> CliError {
    CliError::Message(err.message())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_v1::{CheckRequest, ShutdownRequest};
    use pretty_assertions::assert_eq;
    use std::time::Instant;

    #[test]
    fn parent_directories_follow_filepath_dir() {
        assert_eq!(parent_dir("/tmp/data/local.db"), "/tmp/data");
        assert_eq!(parent_dir("local.db"), ".");
        assert_eq!(parent_dir("/local.db"), "/");
    }

    /// Starts the daemon against a temporary database and answers the
    /// Desktop's handshake: `core.json` next to the database, and the health
    /// check it probes before attaching.
    #[test]
    fn serve_publishes_core_json_and_answers_health() {
        let dir = tempfile::tempdir().expect("temp dir");
        let db = dir.path().join("local.db");
        let db_path = db.to_string_lossy().into_owned();
        let serve_db = db_path.clone();
        let handle = std::thread::spawn(move || {
            run_serve(
                &serve_db,
                &[
                    "--listen".to_string(),
                    "127.0.0.1:0".to_string(),
                    "--token".to_string(),
                    "serve-test-token".to_string(),
                ],
            )
        });

        let deadline = Instant::now() + Duration::from_secs(20);
        let core = loop {
            if let Some(core) = runstate::read_core(&db_path) {
                break core;
            }
            assert!(
                Instant::now() < deadline,
                "core.json was not written: {:?}",
                handle
            );
            std::thread::sleep(Duration::from_millis(50));
        };
        assert!(core.endpoint.starts_with("http://127.0.0.1:"));
        assert_eq!(core.token, "serve-test-token");
        assert_eq!(core.db_path, db_path);
        assert!(!core.build.is_empty(), "the build id names this binary");

        let clients = coreclient::new(&core, "cli");
        let health = clients
            .health()
            .check(&CheckRequest::default())
            .expect("health");
        assert_eq!(health.status, "ok");
        assert_eq!(health.version, "0.1.0");
        assert_eq!(health.build, core.build);
        assert_eq!(health.pid, core.pid);

        // The daemon runs in this process, so stop it through the same
        // MaintenanceService.Shutdown the Desktop sends rather than by pid.
        clients
            .maintenance()
            .shutdown(&ShutdownRequest {
                reason: "stop".to_string(),
            })
            .expect("shutdown");
        assert!(handle.join().expect("serve").is_ok());
        assert!(runstate::read_core(&db_path).is_none());
    }
}
