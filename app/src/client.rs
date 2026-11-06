//! The core's Connect-JSON client. The core (`polterware serve`) is the only process that
//! opens the SQLite store (Decision 0007); this app finds the one that owns the database
//! through `<db dir>/run/core.json`, refuses an endpoint that is not loopback, and starts the
//! core only when asked to.

use std::fs::{self, OpenOptions};
use std::io::Read;
use std::marker::PhantomData;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// What the core is told the caller is (`Polterware-Client`).
const CLIENT_NAME: &str = "desktop";
const MAX_CORE_JSON_BYTES: u64 = 16_384;
const CALL_TIMEOUT: Duration = Duration::from_secs(10);
const START_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreError {
    /// No live core owns the database.
    NotRunning,
    Rpc(String),
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::NotRunning => f.write_str("the local core is not running"),
            CoreError::Rpc(message) => f.write_str(message),
        }
    }
}

/// A running core: where it listens and its bearer token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Core {
    pub endpoint: String,
    pub token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CoreFile {
    endpoint: String,
    token: String,
    #[serde(default)]
    pid: i32,
}

/// The database every Polterware client shares: `POLTERWARE_DB`, else
/// `~/Library/Application Support/polterware/local.db`, the Desktop's.
pub fn database_path() -> PathBuf {
    if let Some(path) = std::env::var_os("POLTERWARE_DB").filter(|path| !path.is_empty()) {
        return PathBuf::from(path);
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join("Library/Application Support/polterware/local.db")
}

fn run_dir(db: &Path) -> PathBuf {
    db.parent().unwrap_or(Path::new(".")).join("run")
}

/// The core that owns `db`. `POLTERWARE_CORE_ENDPOINT` and `POLTERWARE_CORE_TOKEN` name one
/// directly, as they do for the Electron Desktop.
pub fn discover(db: &Path) -> Result<Core, CoreError> {
    let endpoint = std::env::var("POLTERWARE_CORE_ENDPOINT").unwrap_or_default();
    let token = std::env::var("POLTERWARE_CORE_TOKEN").unwrap_or_default();
    if !endpoint.trim().is_empty() && !token.trim().is_empty() {
        return loopback(endpoint.trim(), token.trim());
    }
    let path = run_dir(db).join("core.json");
    let Ok(file) = fs::File::open(&path) else {
        return Err(CoreError::NotRunning);
    };
    let mut body = String::new();
    file.take(MAX_CORE_JSON_BYTES)
        .read_to_string(&mut body)
        .map_err(|error| CoreError::Rpc(format!("read {}: {error}", path.display())))?;
    let state: CoreFile = serde_json::from_str(&body)
        .map_err(|error| CoreError::Rpc(format!("parse {}: {error}", path.display())))?;
    if !pid_alive(state.pid) {
        return Err(CoreError::NotRunning);
    }
    loopback(&state.endpoint, &state.token)
}

/// A core at `endpoint`, refused unless it listens on this computer.
pub(crate) fn loopback(endpoint: &str, token: &str) -> Result<Core, CoreError> {
    let endpoint = endpoint.trim_end_matches('/');
    let local = ["http://127.0.0.1:", "http://localhost:", "http://[::1]:"]
        .iter()
        .any(|prefix| endpoint.starts_with(prefix));
    if !local {
        return Err(CoreError::Rpc(format!(
            "the core endpoint {endpoint} is not on this computer"
        )));
    }
    Ok(Core {
        endpoint: endpoint.to_string(),
        token: token.to_string(),
    })
}

pub(crate) fn pid_alive(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }
    // SAFETY: signal 0 only checks that the process exists.
    let alive = unsafe { libc::kill(pid, 0) } == 0;
    alive || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

/// The core binary, as `tooling/release/bundle.sh` packages it: first beside
/// the running bundle's executable, then `POLTERWARE_BIN`, the installed app, and the checkout.
pub fn cli_binary() -> Option<PathBuf> {
    cli_candidates(
        std::env::current_exe().ok().as_deref(),
        std::env::var_os("POLTERWARE_BIN"),
    )
    .into_iter()
    .find(|path| path.is_file())
}

fn cli_candidates(
    executable: Option<&Path>,
    override_path: Option<std::ffi::OsString>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(resources) = executable.and_then(crate::updater::bundle_resources) {
        candidates.push(resources.join("polterware"));
    }
    if let Some(path) = override_path.filter(|path| !path.is_empty()) {
        candidates.push(PathBuf::from(path));
    }
    candidates.push(PathBuf::from(
        "/Applications/Polterware.app/Contents/Resources/polterware",
    ));
    candidates.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist/polterware"));
    candidates
}

#[cfg(test)]
mod binary_tests {
    use super::*;
    #[test]
    fn bundle_binary_precedes_existing_candidates() {
        let paths = cli_candidates(
            Some(Path::new("/tmp/Polterware.app/Contents/MacOS/Polterware")),
            Some("/custom/core".into()),
        );
        assert_eq!(
            paths[0],
            Path::new("/tmp/Polterware.app/Contents/Resources/polterware")
        );
        assert_eq!(paths[1], Path::new("/custom/core"));
        assert_eq!(
            cli_candidates(Some(Path::new("/tmp/core")), None)[0],
            Path::new("/Applications/Polterware.app/Contents/Resources/polterware")
        );
    }
}

/// Where a core this app starts writes its log.
pub fn core_log(db: &Path) -> PathBuf {
    run_dir(db).join("core.log")
}

/// Starts `polterware serve` detached, logging to `run/core.log`, and waits for it to write
/// `core.json`, as the Electron Desktop does; `idle` is its `--idle` (`0` keeps it running). A
/// core that already runs is returned as is.
pub fn start(db: &Path, idle: &str) -> Result<Core, CoreError> {
    if let Ok(core) = discover(db) {
        return Ok(core);
    }
    let binary = cli_binary().ok_or_else(|| {
        CoreError::Rpc(
            "No polterware binary found. Set POLTERWARE_BIN or run pnpm build:cli.".into(),
        )
    })?;
    let dir = run_dir(db);
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&dir)
        .map_err(|error| CoreError::Rpc(format!("create {}: {error}", dir.display())))?;
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(dir.join("core.log"))
        .map_err(|error| CoreError::Rpc(format!("open the core log: {error}")))?;
    let err_log = log
        .try_clone()
        .map_err(|error| CoreError::Rpc(error.to_string()))?;
    let mut child = Command::new(&binary)
        .arg("-db")
        .arg(db)
        .args(["serve", "--listen", "127.0.0.1:0", "--idle", idle])
        .env_remove("POLTERWARE_SESSION_TOKEN")
        .stdin(Stdio::null())
        .stdout(log)
        .stderr(err_log)
        .process_group(0)
        .spawn()
        .map_err(|error| CoreError::Rpc(format!("start {}: {error}", binary.display())))?;
    let deadline = Instant::now() + START_TIMEOUT;
    while Instant::now() < deadline {
        if let Ok(core) = discover(db) {
            return Ok(core);
        }
        if let Ok(Some(status)) = child.try_wait() {
            // Exit 3: another core took the lock first, so attach to it.
            if status.code() == Some(3)
                && let Ok(core) = discover(db)
            {
                return Ok(core);
            }
            return Err(CoreError::Rpc(format!(
                "the core exited ({status}); see {}",
                dir.join("core.log").display()
            )));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(CoreError::Rpc(
        "the core did not start in 15 seconds".into(),
    ))
}

/// How long a stopped core has to exit.
const STOP_TIMEOUT: Duration = Duration::from_secs(10);

/// Asks the core that owns `db` to shut down (`stopCore`), signalling it when the call fails,
/// and waits for it to exit. No core running is already stopped.
pub fn stop(db: &Path) -> Result<(), CoreError> {
    let pid = core_pid(db);
    let Ok(core) = discover(db) else {
        return Ok(());
    };
    let asked = core
        .maintenance()
        .shutdown(&crate::contracts::ShutdownRequest {
            reason: "desktop-quit".into(),
        });
    if asked.is_err()
        && let Some(pid) = pid
    {
        // SAFETY: a signal to the core process named in `core.json`.
        unsafe {
            libc::kill(pid, libc::SIGTERM);
        }
    }
    let Some(pid) = pid else {
        return Ok(());
    };
    let deadline = Instant::now() + STOP_TIMEOUT;
    while Instant::now() < deadline {
        if !pid_alive(pid) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(CoreError::Rpc("the core did not stop in 10 seconds".into()))
}

/// The pid `core.json` names.
fn core_pid(db: &Path) -> Option<i32> {
    let body = fs::read_to_string(run_dir(db).join("core.json")).ok()?;
    let state: CoreFile = serde_json::from_str(&body).ok()?;
    (state.pid > 0).then_some(state.pid)
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct ConnectError {
    code: String,
    message: String,
}

impl ConnectError {
    /// The error's message, else its code, else `fallback`.
    fn into_message(self, fallback: impl FnOnce() -> String) -> String {
        match (self.code.is_empty(), self.message.is_empty()) {
            (_, false) => self.message,
            (false, true) => self.code,
            (true, true) => fallback(),
        }
    }
}

impl Core {
    fn post(&self, agent: &ureq::Agent, route: &str, content_type: &str) -> ureq::Request {
        agent
            .post(&format!("{}/{route}", self.endpoint))
            .set("Content-Type", content_type)
            .set("Connect-Protocol-Version", "1")
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Polterware-Client", CLIENT_NAME)
    }

    /// One unary Connect call in JSON: `POST <endpoint>/<package>.<Service>/<Method>`. The
    /// generated clients (`crate::contracts`) call it: `core.work().list_inbox(&request)`.
    pub fn unary<Req: Serialize, Res: DeserializeOwned>(
        &self,
        route: &str,
        request: &Req,
    ) -> Result<Res, CoreError> {
        self.unary_within(route, request, CALL_TIMEOUT)
    }

    /// [`Core::unary`] with its own timeout, for a quick probe.
    pub fn unary_within<Req: Serialize, Res: DeserializeOwned>(
        &self,
        route: &str,
        request: &Req,
        timeout: Duration,
    ) -> Result<Res, CoreError> {
        let body =
            serde_json::to_string(request).map_err(|error| CoreError::Rpc(error.to_string()))?;
        let response = self
            .post(&ureq::agent(), route, "application/json")
            .timeout(timeout)
            .send_string(&body);
        match response {
            Ok(response) => {
                let text = response
                    .into_string()
                    .map_err(|error| CoreError::Rpc(error.to_string()))?;
                serde_json::from_str(&text)
                    .map_err(|error| CoreError::Rpc(format!("{route}: {error}")))
            }
            Err(ureq::Error::Status(status, response)) => Err(status_error(status, response)),
            Err(ureq::Error::Transport(_)) => Err(CoreError::NotRunning),
        }
    }

    /// A server-streaming Connect call in JSON (`application/connect+json`). The stream has no
    /// read timeout: the core holds it open, and keeps running while it is, until it stops.
    pub fn server_stream<Req: Serialize, Res: DeserializeOwned>(
        &self,
        route: &str,
        request: &Req,
    ) -> Result<ServerStream<Res>, CoreError> {
        let body =
            serde_json::to_vec(request).map_err(|error| CoreError::Rpc(error.to_string()))?;
        // Only connecting is bounded: the stream stays open as long as the core holds it.
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(CALL_TIMEOUT)
            .build();
        let response = self
            .post(&agent, route, "application/connect+json")
            .send_bytes(&envelope(0, &body));
        match response {
            Ok(response) => Ok(ServerStream {
                route: route.to_string(),
                body: response.into_reader(),
                done: false,
                message: PhantomData,
            }),
            Err(ureq::Error::Status(status, response)) => Err(status_error(status, response)),
            Err(ureq::Error::Transport(_)) => Err(CoreError::NotRunning),
        }
    }
}

fn status_error(status: u16, response: ureq::Response) -> CoreError {
    let text = response.into_string().unwrap_or_default();
    let error = serde_json::from_str::<ConnectError>(&text).unwrap_or_default();
    CoreError::Rpc(error.into_message(|| format!("HTTP {status}")))
}

/// The end of a stream carries this flag, then `{"error": …, "metadata": …}`.
const END_STREAM: u8 = 0b10;
const COMPRESSED: u8 = 0b01;
/// A message larger than this is refused rather than read.
const MAX_MESSAGE_BYTES: usize = 32 << 20;

/// A Connect envelope: one flag byte, the length as a big-endian u32, then the bytes.
fn envelope(flags: u8, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len() + 5);
    out.push(flags);
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(body);
    out
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct EndStream {
    error: Option<ConnectError>,
}

/// The messages of a server stream, read as they arrive. It ends with `None` when the core
/// closes the stream; an error the core sends at its end, or a broken connection, comes as
/// one `Err` first.
pub struct ServerStream<T> {
    route: String,
    body: Box<dyn Read + Send + Sync + 'static>,
    done: bool,
    message: PhantomData<fn() -> T>,
}

impl<T: DeserializeOwned> ServerStream<T> {
    fn read_envelope(&mut self) -> Result<Option<(u8, Vec<u8>)>, CoreError> {
        let mut head = [0u8; 5];
        match read_full(&mut self.body, &mut head) {
            Ok(0) => return Ok(None),
            Ok(5) => {}
            Ok(_) => {
                return Err(CoreError::Rpc(format!(
                    "{}: the stream was cut",
                    self.route
                )));
            }
            Err(_) => return Err(CoreError::NotRunning),
        }
        let length = u32::from_be_bytes([head[1], head[2], head[3], head[4]]) as usize;
        if length > MAX_MESSAGE_BYTES {
            return Err(CoreError::Rpc(format!(
                "{}: a {length}-byte message is too large",
                self.route
            )));
        }
        let mut body = vec![0u8; length];
        match read_full(&mut self.body, &mut body) {
            Ok(read) if read == length => Ok(Some((head[0], body))),
            Ok(_) => Err(CoreError::Rpc(format!(
                "{}: the stream was cut",
                self.route
            ))),
            Err(_) => Err(CoreError::NotRunning),
        }
    }
}

impl<T: DeserializeOwned> Iterator for ServerStream<T> {
    type Item = Result<T, CoreError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let outcome = self.read_envelope();
        let (flags, body) = match outcome {
            Ok(Some(envelope)) => envelope,
            Ok(None) => {
                self.done = true;
                return None;
            }
            Err(error) => {
                self.done = true;
                return Some(Err(error));
            }
        };
        if flags & COMPRESSED != 0 {
            self.done = true;
            return Some(Err(CoreError::Rpc(format!(
                "{}: the core sent a compressed message",
                self.route
            ))));
        }
        if flags & END_STREAM != 0 {
            self.done = true;
            let end = serde_json::from_slice::<EndStream>(&body).unwrap_or_default();
            let route = &self.route;
            return end.error.map(|error| {
                Err(CoreError::Rpc(
                    error.into_message(|| format!("{route} failed")),
                ))
            });
        }
        Some(
            serde_json::from_slice(&body)
                .map_err(|error| CoreError::Rpc(format!("{}: {error}", self.route))),
        )
    }
}

/// Reads until `buf` is full or the stream ends; returns how much it read.
fn read_full(reader: &mut impl Read, buf: &mut [u8]) -> std::io::Result<usize> {
    let mut filled = 0;
    while filled < buf.len() {
        match reader.read(&mut buf[filled..]) {
            Ok(0) => break,
            Ok(read) => filled += read,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(filled)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{
        ListInboxResponse, UpdateCanvasRequest, WatchChangesRequest, WatchChangesResponse,
    };

    #[test]
    fn refuses_remote_endpoints() {
        assert!(loopback("http://127.0.0.1:4100/", "t").is_ok());
        assert!(loopback("http://localhost:4100", "t").is_ok());
        assert!(loopback("https://example.com", "t").is_err());
        assert!(loopback("http://127.0.0.1.example.com:80", "t").is_err());
    }

    #[test]
    fn reads_proto_json_with_defaults_omitted() {
        let inbox: ListInboxResponse = serde_json::from_str(
            r#"{"items":[{"id":"q1","kind":"decision","title":"Which cache?","where":"cache#T2",
                "options":["redis","none"],"details":{"provider":"vercel"},"unknownField":1}],
                "openCount":1}"#,
        )
        .unwrap();
        assert_eq!(inbox.open_count, 1);
        assert_eq!(inbox.blocking_count, 0);
        let item = &inbox.items[0];
        assert_eq!(item.r#where, "cache#T2");
        assert!(!item.blocking);
        assert_eq!(item.options, ["redis", "none"]);
        assert_eq!(item.details["provider"], "vercel");
    }

    #[test]
    fn writes_requests_in_camel_case_with_unset_optionals_left_out() {
        let request = UpdateCanvasRequest {
            project_id: "p1".into(),
            name: Some("Flow".into()),
            ..UpdateCanvasRequest::default()
        };
        let text = serde_json::to_string(&request).unwrap();
        assert!(text.contains(r#""projectId":"p1""#), "{text}");
        assert!(text.contains(r#""name":"Flow""#), "{text}");
        assert!(!text.contains("agentVisible"), "{text}");
        assert!(!text.contains("graph"), "{text}");
        let watch = serde_json::to_string(&WatchChangesRequest { after_seq: 12 }).unwrap();
        assert_eq!(watch, r#"{"afterSeq":"12"}"#);
    }

    fn stream_of(bytes: Vec<u8>) -> ServerStream<WatchChangesResponse> {
        ServerStream {
            route: "polterware.local.v1.ChangeService/WatchChanges".into(),
            body: Box::new(std::io::Cursor::new(bytes)),
            done: false,
            message: PhantomData,
        }
    }

    #[test]
    fn reads_a_server_stream_to_its_end() {
        let mut bytes = envelope(0, br#"{"seq":"3","at":"2026-11-04T10:00:00Z"}"#);
        bytes.extend(envelope(
            0,
            br#"{"seq":"4","kinds":["work"],"projectIds":["p1"]}"#,
        ));
        bytes.extend(envelope(END_STREAM, b"{}"));
        let events: Vec<_> = stream_of(bytes).collect::<Result<_, _>>().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].seq, 3);
        assert_eq!(events[1].kinds, ["work"]);
        assert_eq!(events[1].project_ids, ["p1"]);
    }

    #[test]
    fn a_stream_ends_with_the_cores_error() {
        let mut bytes = envelope(0, br#"{"seq":"1"}"#);
        bytes.extend(envelope(
            END_STREAM,
            br#"{"error":{"code":"unauthenticated","message":"invalid or missing session token"}}"#,
        ));
        let mut stream = stream_of(bytes);
        assert_eq!(stream.next().unwrap().unwrap().seq, 1);
        assert_eq!(
            stream.next().unwrap(),
            Err(CoreError::Rpc("invalid or missing session token".into()))
        );
        assert!(stream.next().is_none());
    }

    #[test]
    fn a_cut_stream_is_an_error() {
        let mut bytes = envelope(0, br#"{"seq":"1"}"#);
        bytes.truncate(bytes.len() - 2);
        let mut stream = stream_of(bytes);
        assert!(matches!(stream.next(), Some(Err(CoreError::Rpc(_)))));
        assert!(stream.next().is_none());
    }

    #[test]
    fn a_missing_core_file_is_not_running() {
        let dir = std::env::temp_dir().join(format!("pw-gpui-{}", std::process::id()));
        assert_eq!(discover(&dir.join("local.db")), Err(CoreError::NotRunning));
    }
}
