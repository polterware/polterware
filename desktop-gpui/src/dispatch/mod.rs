//! Polterware Dispatch, its own app with its own core (`dispatch serve`). Polterware reads a
//! few of its answers (Activity's Dispatch rows) and opens the rest in the Dispatch app through
//! its `dispatch://` scheme, as the Electron Desktop does (`src/main/dispatch/client.ts`): the
//! core is found through `<data dir>/run/core.json`, checked with `HealthService/Check`, and
//! started once, detached, when the app is installed.

pub mod contracts;

use std::fs;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::client::{Core, CoreError, ServerStream, loopback, pid_alive};
use contracts::{CheckRequest, CheckResponse};

const MAX_CORE_JSON_BYTES: u64 = 16_384;
/// How long a health check waits.
const HEALTH_TIMEOUT: Duration = Duration::from_secs(2);
/// A started core is looked for this many times, 100ms apart.
const START_ATTEMPTS: u32 = 50;

/// A running Dispatch core. The generated clients hang off it: `dispatch.dispatch().list_history(..)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dispatch(Core);

impl Dispatch {
    pub fn unary<Req: Serialize, Res: DeserializeOwned>(
        &self,
        route: &str,
        request: &Req,
    ) -> Result<Res, CoreError> {
        self.0.unary(route, request)
    }

    pub fn server_stream<Req: Serialize, Res: DeserializeOwned>(
        &self,
        route: &str,
        request: &Req,
    ) -> Result<ServerStream<Res>, CoreError> {
        self.0.server_stream(route, request)
    }

    fn healthy(&self) -> bool {
        self.0
            .unary_within::<_, CheckResponse>(
                "dispatch.v1.HealthService/Check",
                &CheckRequest {},
                HEALTH_TIMEOUT,
            )
            .is_ok()
    }
}

/// Where Polterware Dispatch keeps its database: `PWDISPATCH_DATA_DIR`, else its Application
/// Support folder.
pub fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var("PWDISPATCH_DATA_DIR")
        .ok()
        .map(|dir| dir.trim().to_string())
        .filter(|dir| !dir.is_empty())
    {
        return PathBuf::from(dir);
    }
    home().join("Library/Application Support/Polterware Dispatch")
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
}

/// The `dispatch` binary: `PWDISPATCH_BIN`, else the installed app's (under its current name,
/// then its old one), else one on `PATH`.
pub fn binary() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var("PWDISPATCH_BIN")
        .ok()
        .map(|path| path.trim().to_string())
        .filter(|path| !path.is_empty())
    {
        candidates.push(PathBuf::from(path));
    }
    for app in ["Dispatch.app", "Polterware Dispatch.app"] {
        let inner = Path::new(app).join("Contents/Resources/dispatch");
        candidates.push(Path::new("/Applications").join(&inner));
        candidates.push(home().join("Applications").join(&inner));
    }
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|dir| dir.join("dispatch")));
    }
    candidates.into_iter().find(|path| path.exists())
}

#[derive(Deserialize)]
struct CoreFile {
    #[serde(default)]
    pid: i32,
    #[serde(default)]
    endpoint: String,
    #[serde(default)]
    token: String,
}

/// The core `run/core.json` names, when its process lives.
fn read_core(dir: &Path) -> Option<(i32, Dispatch)> {
    let file = fs::File::open(dir.join("run/core.json")).ok()?;
    let mut body = String::new();
    file.take(MAX_CORE_JSON_BYTES)
        .read_to_string(&mut body)
        .ok()?;
    let core: CoreFile = serde_json::from_str(&body).ok()?;
    if core.endpoint.is_empty() || core.token.is_empty() || !pid_alive(core.pid) {
        return None;
    }
    let endpoint = loopback(&core.endpoint, &core.token).ok()?;
    Some((core.pid, Dispatch(endpoint)))
}

/// The core last connected to and its pid, so a later call skips the health check while
/// the same process runs. Held while connecting, so two callers start one core.
static CONNECTED: Mutex<Option<(i32, Dispatch)>> = Mutex::new(None);

/// The running Dispatch core, started first when the app is installed and none runs.
/// `None` without the app, or when its core does not come up.
pub fn connect() -> Option<Dispatch> {
    let mut connected = CONNECTED
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let dir = data_dir();
    let running = read_core(&dir);
    if let (Some((pid, core)), Some((cached_pid, cached))) = (&running, connected.as_ref())
        && pid == cached_pid
        && core == cached
    {
        return Some(core.clone());
    }
    let found = match running {
        Some((pid, core)) if core.healthy() => Some((pid, core)),
        _ => start(&dir),
    };
    *connected = found.clone();
    found.map(|(_, core)| core)
}

/// Starts `dispatch -db <data dir>/dispatch.db serve` detached and waits for it to answer.
fn start(dir: &Path) -> Option<(i32, Dispatch)> {
    let binary = binary()?;
    Command::new(binary)
        .arg("-db")
        .arg(dir.join("dispatch.db"))
        .arg("serve")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .ok()?;
    for _ in 0..START_ATTEMPTS {
        std::thread::sleep(Duration::from_millis(100));
        if let Some((pid, core)) = read_core(dir)
            && core.healthy()
        {
            return Some((pid, core));
        }
    }
    None
}

/// Opens only Dispatch links when the Electron Desktop's `dispatch-link.ts` and
/// `src/main/dispatch/client.ts` availability rule is met.
pub fn open_in_dispatch(url: &str, cx: &mut gpui::App) {
    if should_open(url, core_available(&data_dir()), binary().is_some()) {
        cx.open_url(url);
    }
}

// Electron's `src/main/dispatch/client.ts` availability checks a live core file,
// without the stricter endpoint validation used when connecting to it.
fn core_available(dir: &Path) -> bool {
    fs::read_to_string(dir.join("run/core.json"))
        .ok()
        .and_then(|body| serde_json::from_str::<CoreFile>(&body).ok())
        .is_some_and(|core| {
            !core.endpoint.is_empty() && !core.token.is_empty() && pid_alive(core.pid)
        })
}

fn should_open(url: &str, core_found: bool, binary_found: bool) -> bool {
    url.starts_with("dispatch://") && (core_found || binary_found)
}

/// `dispatchRunUrl`: the link that opens a run in the Dispatch app.
pub fn run_url(run_id: &str) -> String {
    let mut url = String::from("dispatch://run/");
    for byte in run_id.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte) {
            url.push(byte as char);
        } else {
            url.push_str(&format!("%{byte:02X}"));
        }
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_requires_a_dispatch_link_and_a_core_or_binary() {
        for core in [false, true] {
            for binary in [false, true] {
                assert_eq!(
                    should_open("dispatch://run/1", core, binary),
                    core || binary
                );
                assert!(!should_open("https://example.com", core, binary));
                assert!(!should_open("Dispatch://run/1", core, binary));
            }
        }
    }

    #[test]
    fn a_run_url_encodes_the_id_as_encode_uri_component_does() {
        assert_eq!(run_url("r-1_a.b"), "dispatch://run/r-1_a.b");
        assert_eq!(run_url("a b/c?"), "dispatch://run/a%20b%2Fc%3F");
        assert_eq!(run_url("é"), "dispatch://run/%C3%A9");
    }

    #[test]
    fn a_core_file_names_the_core_only_while_its_process_lives() {
        let dir = std::env::temp_dir().join(format!("pw-dispatch-test-{}", std::process::id()));
        fs::create_dir_all(dir.join("run")).unwrap();
        let write = |pid: i32, endpoint: &str| {
            fs::write(
                dir.join("run/core.json"),
                format!(r#"{{"pid":{pid},"endpoint":"{endpoint}","token":"t"}}"#),
            )
            .unwrap();
        };
        write(std::process::id() as i32, "http://127.0.0.1:4100");
        assert!(core_available(&dir));
        let (pid, core) = read_core(&dir).expect("a live core");
        assert_eq!(pid, std::process::id() as i32);
        assert_eq!(core.0.endpoint, "http://127.0.0.1:4100");
        // A core on another computer is refused.
        write(std::process::id() as i32, "http://example.com:4100");
        assert!(read_core(&dir).is_none());
        assert!(core_available(&dir));
        // A dead process names no core.
        write(0, "http://127.0.0.1:4100");
        assert!(!core_available(&dir));
        assert!(read_core(&dir).is_none());
        fs::remove_dir_all(&dir).unwrap();
    }
}
