//! Port of `backend/platform/coreclient`.
//!
//! Finds, starts, and talks to the local core (Decision 0007). The core
//! (`polterware serve`) is the only process that opens the SQLite store; the CLI
//! and the MCP bridge reach it through these helpers.
//!
//! The core speaks Connect-JSON over loopback. Core has no HTTP client crate, so
//! the unary call is written here on `std::net`: one JSON `POST` per call, as the
//! Desktop's client does.

use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::local_v1::{CheckRequest, CheckResponse, ShutdownRequest, ShutdownResponse};
use crate::platform::runstate;

/// Names the calling client (desktop, cli, tui, mcp) so the core can label audit
/// records and pick defaults.
pub const CLIENT_HEADER: &str = "Polterware-Client";

/// The timeout one core call gets when it has none of its own.
pub const CALL_TIMEOUT: Duration = Duration::from_secs(10);
/// The timeout a liveness probe and a shutdown request get.
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);
/// How long `start` waits for `core.json`.
const START_TIMEOUT: Duration = Duration::from_secs(15);
/// How long `start` waits between looks at `core.json`.
const POLL: Duration = Duration::from_millis(100);
/// How many polls a core that exited early leaves for the winner of the race.
const ATTACH_ATTEMPTS: usize = 20;
/// How many polls `stop` waits for the core process to exit.
const STOP_ATTEMPTS: usize = 50;
/// How many times `ensure` looks at a core before giving up.
const ENSURE_ATTEMPTS: usize = 3;
/// How long `ensure` waits for a core that just started to answer.
const RESTART_PAUSE: Duration = Duration::from_millis(200);

/// The errors coreclient returns, one per Go error site.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A formatted message, as Go's `fmt.Errorf` returns it.
    #[error("{0}")]
    Message(String),
    #[error("{message}")]
    Connect {
        message: String,
        details: Vec<serde_json::Value>,
    },
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Runstate(#[from] runstate::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

const HEALTH_CHECK: &str = "polterware.local.v1.HealthService/Check";
const MAINTENANCE_SHUTDOWN: &str = "polterware.local.v1.MaintenanceService/Shutdown";

/// Resolves this binary; the core is its own `serve` subcommand.
pub fn executable() -> Result<PathBuf> {
    let exe = std::env::current_exe()
        .map_err(|err| Error::Message(format!("resolve polterware executable: {err}")))?;
    Ok(fs::canonicalize(&exe).unwrap_or(exe))
}

/// Identifies a binary by modification time and size ("<unix seconds>-<bytes>").
/// The Desktop computes the same value.
pub fn build_id(exe: impl AsRef<Path>) -> String {
    let Ok(info) = fs::metadata(exe) else {
        return String::new();
    };
    let seconds = info
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|since| since.as_secs())
        .unwrap_or(0);
    format!("{seconds}-{}", info.len())
}

/// Reports whether build `a` is a newer binary than build `b`.
pub fn newer_build(a: &str, b: &str) -> bool {
    build_time(a) > build_time(b)
}

fn build_time(build: &str) -> i64 {
    build
        .split('-')
        .next()
        .unwrap_or_default()
        .parse::<i64>()
        .unwrap_or(0)
}

/// Returns a live core for `db_path`. It starts one when none is running and
/// replaces one that runs an older build than this binary.
pub fn ensure(db_path: impl AsRef<Path>) -> Result<runstate::Core> {
    let db_path = db_path.as_ref();
    let exe = executable()?;
    let build = build_id(&exe);
    for _ in 0..ENSURE_ATTEMPTS {
        let Some(core) = runstate::read_core(db_path) else {
            return start(&exe, db_path);
        };
        let health = new(&core, "cli")
            .health()
            .check_within(&CheckRequest::default(), PROBE_TIMEOUT);
        let Ok(health) = health else {
            // Listening but not answering yet (just started): retry shortly.
            std::thread::sleep(RESTART_PAUSE);
            continue;
        };
        if newer_build(&build, &health.build) {
            stop(db_path).map_err(|err| Error::Message(format!("replace older core: {err}")))?;
            continue;
        }
        return Ok(core);
    }
    Err(Error::Message(format!(
        "core for {} is not responding; see {}",
        db_path.display(),
        runstate::log_path(db_path).display()
    )))
}

/// Launches a detached core and waits for its `core.json`. When another core
/// wins the start race, that core is returned.
pub fn start(exe: impl AsRef<Path>, db_path: impl AsRef<Path>) -> Result<runstate::Core> {
    let db_path = db_path.as_ref();
    if let Some(core) = runstate::read_core(db_path) {
        return Ok(core);
    }
    runstate::ensure_dir(db_path)?;
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(runstate::log_path(db_path))?;
    let err_log = log.try_clone()?;

    let mut child = Command::new(exe.as_ref())
        .arg("-db")
        .arg(db_path)
        .args(["serve", "--listen", "127.0.0.1:0"])
        .stdin(Stdio::null())
        .stdout(log)
        .stderr(err_log)
        .process_group(0)
        .spawn()?;
    // Go waits for the child in a goroutine, which also reaps it: a core this
    // process started must not linger as a zombie, or `stop` would never see it
    // exit.
    let (exited_tx, exited) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = exited_tx.send(child.wait());
    });

    let deadline = Instant::now() + START_TIMEOUT;
    loop {
        if let Some(core) = runstate::read_core(db_path) {
            return Ok(core);
        }
        if let Ok(exited) = exited.try_recv() {
            // Another core took the lock first; attach to it.
            for _ in 0..ATTACH_ATTEMPTS {
                if let Some(core) = runstate::read_core(db_path) {
                    return Ok(core);
                }
                std::thread::sleep(POLL);
            }
            return Err(Error::Message(format!(
                "core exited before listening ({}); see {}",
                exited.map_or_else(|err| err.to_string(), |status| exit_error(&status)),
                runstate::log_path(db_path).display()
            )));
        }
        if Instant::now() >= deadline {
            return Err(Error::Message(format!(
                "timed out waiting for core; see {}",
                runstate::log_path(db_path).display()
            )));
        }
        std::thread::sleep(POLL);
    }
}

/// Asks the running core to shut down and waits for it to exit.
pub fn stop(db_path: impl AsRef<Path>) -> Result<()> {
    let db_path = db_path.as_ref();
    let Some(core) = runstate::read_core(db_path) else {
        return Ok(());
    };
    let shutdown = new(&core, "cli").maintenance().shutdown_within(
        &ShutdownRequest {
            reason: "stop".to_string(),
        },
        PROBE_TIMEOUT,
    );
    if shutdown.is_err() {
        terminate(core.pid)?;
    }
    for _ in 0..STOP_ATTEMPTS {
        if !runstate::pid_alive(core.pid) {
            return Ok(());
        }
        std::thread::sleep(POLL);
    }
    Err(Error::Message(format!(
        "core pid {} did not exit",
        core.pid
    )))
}

/// The typed Connect client of one core. Go's generated service clients
/// (`HealthServiceClient`, `MaintenanceServiceClient`, and the rest) are code
/// generated from the protos; the Rust core has none generated, so the calls
/// this package makes are spelled out and any other route goes through
/// [`Clients::unary`].
#[derive(Clone, Debug)]
pub struct Clients {
    core: runstate::Core,
    client: String,
}

/// Builds clients for `core`; `client` names the caller (`CLIENT_HEADER`).
pub fn new(core: &runstate::Core, client: &str) -> Clients {
    Clients {
        core: core.clone(),
        client: client.to_string(),
    }
}

/// Ensures a core for `db_path` and returns clients for it.
pub fn connect(db_path: impl AsRef<Path>, client: &str) -> Result<Clients> {
    let core = ensure(db_path)?;
    Ok(new(&core, client))
}

impl Clients {
    /// The core these clients talk to.
    pub fn core(&self) -> &runstate::Core {
        &self.core
    }

    /// The client name these clients set on every call.
    pub fn client(&self) -> &str {
        &self.client
    }

    pub fn health(&self) -> HealthService<'_> {
        HealthService(self)
    }

    pub fn maintenance(&self) -> MaintenanceService<'_> {
        MaintenanceService(self)
    }

    /// One unary Connect call in JSON:
    /// `POST <endpoint>/<package>.<Service>/<Method>`.
    pub fn unary<Req: Serialize + ?Sized, Res: DeserializeOwned>(
        &self,
        route: &str,
        request: &Req,
        timeout: Duration,
    ) -> Result<Res> {
        let body =
            serde_json::to_vec(request).map_err(|err| Error::Message(format!("{route}: {err}")))?;
        let (status, response) = post(&self.core, &self.client, route, &body, timeout)?;
        if status != 200 {
            return Err(status_error(status, &response));
        }
        serde_json::from_slice(&response).map_err(|err| Error::Message(format!("{route}: {err}")))
    }
}

/// `polterware.local.v1.HealthService`.
pub struct HealthService<'a>(&'a Clients);

impl HealthService<'_> {
    pub fn check(&self, request: &CheckRequest) -> Result<CheckResponse> {
        self.check_within(request, CALL_TIMEOUT)
    }

    pub fn check_within(&self, request: &CheckRequest, timeout: Duration) -> Result<CheckResponse> {
        self.0.unary(HEALTH_CHECK, request, timeout)
    }
}

/// `polterware.local.v1.MaintenanceService`.
pub struct MaintenanceService<'a>(&'a Clients);

impl MaintenanceService<'_> {
    pub fn shutdown(&self, request: &ShutdownRequest) -> Result<ShutdownResponse> {
        self.shutdown_within(request, CALL_TIMEOUT)
    }

    pub fn shutdown_within(
        &self,
        request: &ShutdownRequest,
        timeout: Duration,
    ) -> Result<ShutdownResponse> {
        self.0.unary(MAINTENANCE_SHUTDOWN, request, timeout)
    }
}

/// Sends one JSON `POST` over the core's loopback endpoint and returns the
/// status with the body.
fn post(
    core: &runstate::Core,
    client: &str,
    route: &str,
    body: &[u8],
    timeout: Duration,
) -> Result<(u16, Vec<u8>)> {
    let (host, port) = endpoint(&core.endpoint)?;
    let address: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|err| Error::Message(format!("core endpoint {}: {err}", core.endpoint)))?;
    let mut stream = TcpStream::connect_timeout(&address, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    let head = format!(
        "POST /{route} HTTP/1.1\r\n\
         Host: {host}:{port}\r\n\
         Content-Type: application/json\r\n\
         Connect-Protocol-Version: 1\r\n\
         Authorization: Bearer {token}\r\n\
         {CLIENT_HEADER}: {client}\r\n\
         Content-Length: {length}\r\n\
         Connection: close\r\n\r\n",
        route = route.trim_start_matches('/'),
        token = core.token,
        length = body.len(),
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;
    split_response(&response)
}

/// The host and port an endpoint names (`http://127.0.0.1:52341`).
fn endpoint(endpoint: &str) -> Result<(String, u16)> {
    let trimmed = endpoint.trim().trim_end_matches('/');
    let rest = trimmed
        .strip_prefix("http://")
        .ok_or_else(|| Error::Message(format!("the core endpoint {trimmed} is not http")))?;
    match rest.rsplit_once(':') {
        Some((host, port)) => {
            let port = port
                .parse::<u16>()
                .map_err(|err| Error::Message(format!("the core endpoint {trimmed}: {err}")))?;
            Ok((host.to_string(), port))
        }
        None => Ok((rest.to_string(), 80)),
    }
}

/// Splits one HTTP/1.1 response into its status code and body.
fn split_response(response: &[u8]) -> Result<(u16, Vec<u8>)> {
    let head_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| Error::Message("the core closed the connection".to_string()))?;
    let head = String::from_utf8_lossy(&response[..head_end]);
    let mut lines = head.split("\r\n");
    let status = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| Error::Message("the core sent no status line".to_string()))?;
    let mut body = &response[head_end + 4..];
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case("content-length")
            && let Ok(length) = value.trim().parse::<usize>()
        {
            body = &body[..length.min(body.len())];
        }
    }
    Ok((status, body.to_vec()))
}

/// The error a Connect response body carries, as the Desktop reads it.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(default)]
struct ConnectError {
    code: String,
    message: String,
    details: Vec<serde_json::Value>,
}

fn status_error(status: u16, body: &[u8]) -> Error {
    let error: ConnectError = serde_json::from_slice(body).unwrap_or_default();
    let message = match (error.code.is_empty(), error.message.is_empty()) {
        (_, false) => error.message,
        (false, true) => error.code,
        (true, true) => format!("HTTP {status}"),
    };
    Error::Connect {
        message,
        details: error.details,
    }
}

/// Signals `pid` to terminate, as Go's `syscall.Kill(pid, syscall.SIGTERM)`
/// does. A process that is already gone is not an error.
fn terminate(pid: i32) -> Result<()> {
    const SIGTERM: i32 = 15;
    const ESRCH: i32 = 3;
    // SAFETY: a signal to the pid named in `core.json`.
    if unsafe { kill(pid, SIGTERM) } == 0 {
        return Ok(());
    }
    let err = io::Error::last_os_error();
    if err.raw_os_error() == Some(ESRCH) {
        return Ok(());
    }
    Err(Error::Io(err))
}

unsafe extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

/// Names an exit the way Go's `os/exec` does: "exit status 3", "signal: killed".
fn exit_error(status: &std::process::ExitStatus) -> String {
    use std::os::unix::process::ExitStatusExt;
    let Some(signal) = status.signal() else {
        return format!("exit status {}", status.code().unwrap_or(-1));
    };
    let name = match signal {
        1 => "hangup",
        2 => "interrupt",
        3 => "quit",
        6 => "aborted",
        9 => "killed",
        13 => "broken pipe",
        14 => "alarm",
        15 => "terminated",
        _ => return format!("signal: {signal}"),
    };
    format!("signal: {name}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::net::TcpListener;
    use time::OffsetDateTime;

    /// One canned reply from a loopback server; the returned handle answers the
    /// request it reads.
    fn serve_once(body: &'static str) -> (String, std::thread::JoinHandle<String>) {
        serve_status("200 OK", body)
    }

    fn serve_status(
        status: &'static str,
        body: &'static str,
    ) -> (String, std::thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let address = listener.local_addr().expect("address");
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let request = read_request(&mut stream);
            let reply = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(reply.as_bytes());
            let _ = stream.flush();
            request
        });
        (format!("http://{address}"), handle)
    }

    fn read_request(stream: &mut TcpStream) -> String {
        let mut head = Vec::new();
        let mut byte = [0u8; 1];
        while !head.ends_with(b"\r\n\r\n") {
            match stream.read(&mut byte) {
                Ok(1) => head.push(byte[0]),
                _ => break,
            }
        }
        let text = String::from_utf8_lossy(&head).into_owned();
        let length = text
            .lines()
            .find_map(|line| line.strip_prefix("Content-Length: "))
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(0);
        let mut body = vec![0u8; length];
        let _ = stream.read_exact(&mut body);
        format!("{text}{}", String::from_utf8_lossy(&body))
    }

    fn core_at(endpoint: &str) -> runstate::Core {
        runstate::Core {
            pid: 1,
            endpoint: endpoint.to_string(),
            token: "token".to_string(),
            db_path: "/tmp/local.db".to_string(),
            build: "1-1".to_string(),
            started_at: OffsetDateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn build_ids_carry_the_time_and_size() {
        let dir = tempfile::tempdir().expect("temp dir");
        let exe = dir.path().join("polterware");
        fs::write(&exe, b"binary").expect("write");
        let id = build_id(&exe);
        let (seconds, size) = id.split_once('-').expect("build id");
        assert!(seconds.parse::<i64>().is_ok(), "{id}");
        assert_eq!(size, "6");
        assert_eq!(build_id(dir.path().join("missing")), "");
    }

    #[test]
    fn newer_build_compares_the_times_only() {
        assert!(newer_build("1700000000-1", "1600000000-999"));
        assert!(!newer_build("1600000000-999", "1700000000-1"));
        assert!(!newer_build("1700000000-1", "1700000000-1"));
        // An unparsable time reads as zero, as Go's ignored ParseInt error does.
        assert!(newer_build("2026-1", "0-1"));
        assert!(!newer_build("abc", "0-1"));
        assert!(!newer_build("", ""));
    }

    #[test]
    fn unary_posts_connect_json_and_reads_the_reply() {
        let (endpoint, handle) =
            serve_once(r#"{"status":"ok","version":"0.1.1","build":"7-42","pid":9}"#);
        let core = core_at(&endpoint);
        let clients = new(&core, "cli");
        let health = clients
            .health()
            .check(&CheckRequest::default())
            .expect("health");
        assert_eq!(health.status, "ok");
        assert_eq!(health.build, "7-42");
        assert_eq!(health.pid, 9);
        let request = handle.join().expect("server");
        assert!(
            request.starts_with("POST /polterware.local.v1.HealthService/Check HTTP/1.1\r\n"),
            "{request}"
        );
        assert!(
            request.contains("Authorization: Bearer token\r\n"),
            "{request}"
        );
        assert!(request.contains("Polterware-Client: cli\r\n"), "{request}");
        assert!(
            request.contains("Connect-Protocol-Version: 1\r\n"),
            "{request}"
        );
        assert!(request.ends_with("\r\n\r\n{}"), "{request}");
    }

    #[test]
    fn a_failed_call_reports_the_connect_message() {
        let (endpoint, handle) = serve_status(
            "404 Not Found",
            r#"{"code":"not_found","message":"no such route"}"#,
        );
        let clients = new(&core_at(&endpoint), "cli");
        let err = clients
            .maintenance()
            .shutdown(&ShutdownRequest {
                reason: "stop".to_string(),
            })
            .unwrap_err();
        assert_eq!(err.to_string(), "no such route");
        let request = handle.join().expect("server");
        assert!(
            request.starts_with("POST /polterware.local.v1.MaintenanceService/Shutdown "),
            "{request}"
        );
        assert!(request.contains(r#""reason":"stop""#), "{request}");
    }

    #[test]
    fn a_reply_without_a_connect_message_falls_back_to_the_status() {
        let (endpoint, handle) = serve_status("503 Service Unavailable", "down");
        let clients = new(&core_at(&endpoint), "cli");
        let err = clients
            .health()
            .check(&CheckRequest::default())
            .unwrap_err();
        assert_eq!(err.to_string(), "HTTP 503");
        handle.join().expect("server");
    }

    #[test]
    fn endpoints_are_read_as_host_and_port() {
        assert_eq!(
            endpoint("http://127.0.0.1:52341/").expect("endpoint"),
            ("127.0.0.1".to_string(), 52341)
        );
        assert_eq!(
            endpoint("http://localhost").expect("endpoint"),
            ("localhost".to_string(), 80)
        );
        assert_eq!(
            endpoint("http://[::1]:7000").expect("endpoint"),
            ("[::1]".to_string(), 7000)
        );
        assert!(endpoint("https://example.com:443").is_err());
        assert!(endpoint("http://127.0.0.1:notaport").is_err());
    }

    #[test]
    fn exit_errors_read_like_go() {
        let status = Command::new("sh")
            .args(["-c", "exit 3"])
            .status()
            .expect("status");
        assert_eq!(exit_error(&status), "exit status 3");
    }

    /// A stand-in core: it publishes `core.json` for its own pid and stays up
    /// until `stop` signals it. No socket, so `stop` takes the signal path.
    fn fake_core(dir: &Path) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let script = dir.join("fake-core");
        fs::write(
            &script,
            "#!/bin/sh\n\
             db=\"$2\"\n\
             run=\"$(dirname \"$db\")/run\"\n\
             mkdir -p \"$run\"\n\
             printf '{\"pid\":%d,\"endpoint\":\"http://127.0.0.1:1\",\"token\":\"token\",\"dbPath\":\"%s\",\"build\":\"1-1\",\"startedAt\":\"2026-01-01T00:00:00Z\"}\\n' \"$$\" \"$db\" > \"$run/core.json\"\n\
             sleep 60\n",
        )
        .expect("write fake core");
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).expect("chmod");
        script
    }

    #[test]
    fn start_returns_the_core_it_launched_and_stop_reaps_it() {
        let dir = tempfile::tempdir().expect("temp dir");
        let db = dir.path().join("local.db");
        let core = start(fake_core(dir.path()), &db).expect("start");
        assert!(core.pid > 0);
        assert_eq!(core.token, "token");
        assert!(runstate::pid_alive(core.pid));
        let again = start(fake_core(dir.path()), &db).expect("attach");
        assert_eq!(again.pid, core.pid, "a running core is returned as is");
        stop(&db).expect("stop");
        assert!(!runstate::pid_alive(core.pid), "stop reaps the core");
        assert!(runstate::read_core(&db).is_none());
    }
}
