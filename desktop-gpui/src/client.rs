//! The core's Connect-JSON client. The core (`polterware serve`) is the only process that
//! opens the SQLite store (Decision 0007); this app finds the one that owns the database
//! through `<db dir>/run/core.json`, refuses an endpoint that is not loopback, and starts the
//! core only when asked to.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// What the core is told the caller is (`Polterware-Client`).
const CLIENT_NAME: &str = "desktop-gpui";
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

fn loopback(endpoint: &str, token: &str) -> Result<Core, CoreError> {
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

fn pid_alive(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }
    // SAFETY: signal 0 only checks that the process exists.
    let alive = unsafe { libc::kill(pid, 0) } == 0;
    alive || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

/// The `polterware` binary that serves the core: `POLTERWARE_BIN`, the installed app's, then
/// this checkout's `dist/polterware` (`pnpm build:cli`).
pub fn cli_binary() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os("POLTERWARE_BIN").filter(|path| !path.is_empty()) {
        candidates.push(PathBuf::from(path));
    }
    candidates.push(PathBuf::from(
        "/Applications/Polterware.app/Contents/Resources/polterware",
    ));
    candidates.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist/polterware"));
    candidates.into_iter().find(|path| path.is_file())
}

/// Starts `polterware serve` detached, logging to `run/core.log`, and waits for it to write
/// `core.json`, as the Electron Desktop does. A core that already runs is returned as is.
pub fn start(db: &Path) -> Result<Core, CoreError> {
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
        .args(["serve", "--listen", "127.0.0.1:0"])
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

#[derive(Deserialize)]
struct ConnectError {
    #[serde(default)]
    code: String,
    #[serde(default)]
    message: String,
}

/// One unary Connect call in JSON: `POST <endpoint>/polterware.local.v1.<service>/<method>`.
pub fn call<Req: Serialize, Res: DeserializeOwned>(
    core: &Core,
    service: &str,
    method: &str,
    request: &Req,
) -> Result<Res, CoreError> {
    let url = format!("{}/polterware.local.v1.{service}/{method}", core.endpoint);
    let body = serde_json::to_string(request).map_err(|error| CoreError::Rpc(error.to_string()))?;
    let response = ureq::post(&url)
        .timeout(CALL_TIMEOUT)
        .set("Content-Type", "application/json")
        .set("Connect-Protocol-Version", "1")
        .set("Authorization", &format!("Bearer {}", core.token))
        .set("Polterware-Client", CLIENT_NAME)
        .send_string(&body);
    match response {
        Ok(response) => {
            let text = response
                .into_string()
                .map_err(|error| CoreError::Rpc(error.to_string()))?;
            serde_json::from_str(&text)
                .map_err(|error| CoreError::Rpc(format!("{service}.{method}: {error}")))
        }
        Err(ureq::Error::Status(status, response)) => {
            let text = response.into_string().unwrap_or_default();
            let message = serde_json::from_str::<ConnectError>(&text)
                .ok()
                .map(
                    |error| match (error.code.is_empty(), error.message.is_empty()) {
                        (_, false) => error.message,
                        (false, true) => error.code,
                        (true, true) => format!("HTTP {status}"),
                    },
                )
                .unwrap_or_else(|| format!("HTTP {status}"));
            Err(CoreError::Rpc(message))
        }
        Err(ureq::Error::Transport(_)) => Err(CoreError::NotRunning),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct ListProjectsResponse {
    projects: Vec<Project>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ListProjectsRequest {
    include_archived: bool,
}

pub fn list_projects(core: &Core) -> Result<Vec<Project>, CoreError> {
    let response: ListProjectsResponse = call(
        core,
        "ProjectService",
        "ListProjects",
        &ListProjectsRequest {
            include_archived: false,
        },
    )?;
    Ok(response.projects)
}

/// One derived Inbox item (`InboxItem` in `work.proto`).
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct InboxItem {
    pub id: String,
    /// "conflict", "deploy", "decision" (an open card question), or "setup".
    pub kind: String,
    pub blocking: bool,
    pub title: String,
    pub detail: String,
    pub options: Vec<String>,
    #[serde(rename = "where")]
    pub location: String,
    pub source: String,
    /// "open" or "deferred".
    pub state: String,
    pub actions: Vec<String>,
    pub scope_node_ids: Vec<String>,
    pub details: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Inbox {
    pub items: Vec<InboxItem>,
    pub open_count: i32,
    pub blocking_count: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ListInboxRequest<'a> {
    project_id: &'a str,
}

/// The project's Inbox. Reading it writes nothing.
pub fn list_inbox(core: &Core, project_id: &str) -> Result<Inbox, CoreError> {
    call(
        core,
        "WorkService",
        "ListInbox",
        &ListInboxRequest { project_id },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_remote_endpoints() {
        assert!(loopback("http://127.0.0.1:4100/", "t").is_ok());
        assert!(loopback("http://localhost:4100", "t").is_ok());
        assert!(loopback("https://example.com", "t").is_err());
        assert!(loopback("http://127.0.0.1.example.com:80", "t").is_err());
    }

    #[test]
    fn reads_proto_json_with_defaults_omitted() {
        let inbox: Inbox = serde_json::from_str(
            r#"{"items":[{"id":"q1","kind":"decision","title":"Which cache?","where":"cache#T2",
                "options":["redis","none"],"details":{"provider":"vercel"}}],"openCount":1}"#,
        )
        .unwrap();
        assert_eq!(inbox.open_count, 1);
        assert_eq!(inbox.blocking_count, 0);
        let item = &inbox.items[0];
        assert_eq!(item.location, "cache#T2");
        assert!(!item.blocking);
        assert_eq!(item.options, ["redis", "none"]);
        assert_eq!(item.details["provider"], "vercel");
    }

    #[test]
    fn a_missing_core_file_is_not_running() {
        let dir = std::env::temp_dir().join(format!("pw-gpui-{}", std::process::id()));
        assert_eq!(discover(&dir.join("local.db")), Err(CoreError::NotRunning));
    }
}
