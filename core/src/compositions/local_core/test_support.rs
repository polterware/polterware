//! Test support for the local core: a core on a temporary database, the HTTP
//! client the tests speak Connect with, and the frame reader of a server stream.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;
use tokio::runtime::Runtime;

use super::connect::Route;
use super::server::{Core, Server, listen, serve_routes};
use crate::providers::sqlite::Store;

/// The tests install the process-wide change recorder, so one core runs at a
/// time (as the Go package's tests run one after another).
static ONE_CORE: Mutex<()> = Mutex::new(());

/// The token every test core mints with.
pub const TEST_TOKEN: &str = "test-session-token";

/// A core, its server, and the temporary database under it.
pub struct Harness {
    pub core: Arc<Core>,
    pub endpoint: String,
    // Kept for the server task the harness starts; dropped after the server.
    _runtime: Runtime,
    server: Option<Server>,
    _dir: TempDir,
    _one: MutexGuard<'static, ()>,
}

impl Harness {
    /// Starts a core and serves it on an ephemeral loopback port.
    pub fn start() -> Harness {
        Harness::start_with(super::routes())
    }

    /// Starts a core and serves an explicit route table, so a test can stand a
    /// handler in for one it does not own.
    pub fn start_with(routes: Vec<Route>) -> Harness {
        let one = ONE_CORE.lock().unwrap_or_else(|poison| poison.into_inner());
        let dir = tempfile::tempdir().expect("temp dir");
        let store = Store::new(dir.path().join("core.db")).expect("open store");
        let core = Core::new(store, TEST_TOKEN);
        core.set_build("test-build");
        let (listener, endpoint) = listen("127.0.0.1:0").expect("bind a loopback port");
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("runtime");
        let server = {
            let _entered = runtime.enter();
            serve_routes(listener, Arc::clone(&core), routes)
        };
        Harness {
            core,
            endpoint,
            _runtime: runtime,
            server: Some(server),
            _dir: dir,
            _one: one,
        }
    }

    /// A unary call with a JSON body.
    pub fn unary(&self, path: &str, token: Option<&str>, body: &str) -> Reply {
        self.request(
            "POST",
            path,
            token,
            Some("application/json"),
            body.as_bytes(),
        )
    }

    /// One HTTP request.
    pub fn request(
        &self,
        method: &str,
        path: &str,
        token: Option<&str>,
        content_type: Option<&str>,
        body: &[u8],
    ) -> Reply {
        let address = self.address();
        let mut stream = TcpStream::connect(&address).expect("connect to the core");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("read timeout");
        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .expect("write timeout");
        let head = Self::head(method, path, &address, token, content_type, body.len());
        stream.write_all(head.as_bytes()).expect("write head");
        stream.write_all(body).expect("write body");
        let mut raw = Vec::new();
        let _ = stream.read_to_end(&mut raw);
        parse(&raw)
    }

    /// Opens a server stream and answers its frames while it is still open.
    pub fn open_stream(&self, path: &str, token: Option<&str>, body: &[u8]) -> OpenStream {
        let address = self.address();
        let mut stream = TcpStream::connect(&address).expect("connect to the core");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("read timeout");
        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .expect("write timeout");
        let head = Self::head(
            "POST",
            path,
            &address,
            token,
            Some("application/connect+json"),
            body.len(),
        );
        stream.write_all(head.as_bytes()).expect("write head");
        stream.write_all(body).expect("write body");
        let mut raw = Vec::new();
        loop {
            if let Some(split) = raw.windows(4).position(|window| window == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&raw[..split]).into_owned();
                let rest = raw[split + 4..].to_vec();
                return OpenStream::new(stream, &head, rest);
            }
            let mut buffer = [0u8; 4096];
            let count = stream.read(&mut buffer).expect("read the response head");
            assert!(count > 0, "the core closed before its response head");
            raw.extend_from_slice(&buffer[..count]);
        }
    }

    fn address(&self) -> String {
        self.endpoint
            .strip_prefix("http://")
            .unwrap_or(&self.endpoint)
            .to_string()
    }

    fn head(
        method: &str,
        path: &str,
        address: &str,
        token: Option<&str>,
        content_type: Option<&str>,
        length: usize,
    ) -> String {
        let mut head =
            format!("{method} {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n");
        if let Some(token) = token {
            head.push_str(&format!("Authorization: Bearer {token}\r\n"));
        }
        if let Some(content_type) = content_type {
            head.push_str(&format!("Content-Type: {content_type}\r\n"));
        }
        head.push_str(&format!("Content-Length: {length}\r\n\r\n"));
        head
    }

    /// Waits until the core reports that it stopped, or `limit` passes.
    pub fn wait_stopped(&self, limit: Duration) -> bool {
        let started = std::time::Instant::now();
        loop {
            if *self.core.done().borrow() {
                return true;
            }
            if started.elapsed() >= limit {
                return false;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        if let Some(server) = self.server.take() {
            server.close();
        }
        // The change recorder is process-wide; the tests that check its absence
        // (platform/db/sqlite) expect nothing installed.
        crate::platform::db::sqlite::set_change_recorder(None);
    }
}

/// One HTTP reply.
#[derive(Debug, Clone)]
pub struct Reply {
    pub status: u16,
    pub content_type: String,
    pub accept_post: String,
    pub accept_encoding: String,
    pub allow: String,
    pub session_id: String,
    pub body: Vec<u8>,
}

impl Reply {
    /// The body as JSON.
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap_or(Value::Null)
    }

    /// The body as text.
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    /// The `Allow` header of a refused method.
    pub fn allow(&self) -> &str {
        &self.allow
    }

    /// The Connect error code the body carries.
    pub fn error_code(&self) -> String {
        self.json()["code"].as_str().unwrap_or_default().to_string()
    }

    /// The frames of a server stream: the flags and the payload of each
    /// envelope.
    pub fn frames(&self) -> Vec<(u8, Vec<u8>)> {
        let mut frames = Vec::new();
        let mut rest = &self.body[..];
        while rest.len() >= 5 {
            let flags = rest[0];
            let length = u32::from_be_bytes([rest[1], rest[2], rest[3], rest[4]]) as usize;
            if rest.len() < 5 + length {
                break;
            }
            frames.push((flags, rest[5..5 + length].to_vec()));
            rest = &rest[5 + length..];
        }
        frames
    }
}

fn parse(raw: &[u8]) -> Reply {
    let split = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
        .unwrap_or(raw.len());
    let head = String::from_utf8_lossy(&raw[..split]).into_owned();
    let mut lines = head.split("\r\n");
    let status = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse().ok())
        .unwrap_or(0);
    let mut reply = Reply {
        status,
        content_type: String::new(),
        accept_post: String::new(),
        accept_encoding: String::new(),
        allow: String::new(),
        session_id: String::new(),
        body: Vec::new(),
    };
    let mut chunked = false;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim().to_string();
        match name.as_str() {
            "content-type" => reply.content_type = value,
            "accept-post" => reply.accept_post = value,
            "accept-encoding" | "connect-accept-encoding" => reply.accept_encoding = value,
            "allow" => reply.allow = value,
            "mcp-session-id" => reply.session_id = value,
            "transfer-encoding" => chunked = value.eq_ignore_ascii_case("chunked"),
            _ => {}
        }
    }
    let body = &raw[split.min(raw.len())..];
    reply.body = if chunked {
        dechunk(body)
    } else {
        body.to_vec()
    };
    reply
}

fn dechunk(body: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut rest = body;
    loop {
        let Some(end) = rest.windows(2).position(|window| window == b"\r\n") else {
            break;
        };
        let size = String::from_utf8_lossy(&rest[..end]).into_owned();
        let size = usize::from_str_radix(size.trim(), 16).unwrap_or(0);
        rest = &rest[end + 2..];
        if size == 0 || rest.len() < size {
            break;
        }
        out.extend_from_slice(&rest[..size]);
        rest = &rest[(size + 2).min(rest.len())..];
    }
    out
}

/// A server stream read frame by frame over a real connection, so a test sees
/// each envelope as the core writes it, before the stream ends. Dropping it
/// closes the connection, as a client that leaves does.
pub struct OpenStream {
    stream: TcpStream,
    /// Bytes read from the socket that are not yet de-chunked.
    raw: Vec<u8>,
    /// Body bytes that are not yet a whole Connect envelope.
    body: Vec<u8>,
    chunked: bool,
    status: u16,
    content_type: String,
    ended: bool,
}

impl OpenStream {
    fn new(stream: TcpStream, head: &str, rest: Vec<u8>) -> OpenStream {
        let mut status = 0;
        let mut content_type = String::new();
        let mut chunked = false;
        for (index, line) in head.split("\r\n").enumerate() {
            if index == 0 {
                status = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(0);
                continue;
            }
            let Some((name, value)) = line.split_once(':') else {
                continue;
            };
            match name.trim().to_ascii_lowercase().as_str() {
                "content-type" => content_type = value.trim().to_string(),
                "transfer-encoding" => chunked = value.trim().eq_ignore_ascii_case("chunked"),
                _ => {}
            }
        }
        let mut open = OpenStream {
            stream,
            raw: Vec::new(),
            body: Vec::new(),
            chunked,
            status,
            content_type,
            ended: false,
        };
        if chunked {
            open.raw = rest;
            open.decode();
        } else {
            open.body = rest;
        }
        open
    }

    /// The response status.
    pub fn status(&self) -> u16 {
        self.status
    }

    /// The response content type.
    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    /// The next envelope, waiting at most `limit` for it; `None` once the
    /// stream ended or the wait passed.
    pub fn next_frame(&mut self, limit: Duration) -> Option<(u8, Vec<u8>)> {
        let deadline = Instant::now() + limit;
        loop {
            if let Some(frame) = take_frame(&mut self.body) {
                return Some(frame);
            }
            if self.ended {
                return None;
            }
            let now = Instant::now();
            if now >= deadline {
                return None;
            }
            self.stream.set_read_timeout(Some(deadline - now)).ok()?;
            let mut buffer = [0u8; 4096];
            match self.stream.read(&mut buffer) {
                Ok(0) => self.ended = true,
                Ok(count) => {
                    if self.chunked {
                        self.raw.extend_from_slice(&buffer[..count]);
                        self.decode();
                    } else {
                        self.body.extend_from_slice(&buffer[..count]);
                    }
                }
                Err(err)
                    if matches!(
                        err.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    return None;
                }
                Err(_) => self.ended = true,
            }
        }
    }

    /// Moves every whole chunk in `raw` into `body`.
    fn decode(&mut self) {
        while !self.ended {
            let Some(end) = self.raw.windows(2).position(|window| window == b"\r\n") else {
                return;
            };
            let size = String::from_utf8_lossy(&self.raw[..end]).into_owned();
            let size = usize::from_str_radix(size.trim(), 16).unwrap_or(0);
            if size == 0 {
                self.ended = true;
                self.raw.clear();
                return;
            }
            let start = end + 2;
            if self.raw.len() < start + size + 2 {
                return;
            }
            self.body.extend_from_slice(&self.raw[start..start + size]);
            self.raw.drain(..start + size + 2);
        }
    }
}

/// One Connect envelope out of `body`, if a whole one is there.
fn take_frame(body: &mut Vec<u8>) -> Option<(u8, Vec<u8>)> {
    if body.len() < 5 {
        return None;
    }
    let length = u32::from_be_bytes([body[1], body[2], body[3], body[4]]) as usize;
    if body.len() < 5 + length {
        return None;
    }
    let flags = body[0];
    let payload = body[5..5 + length].to_vec();
    body.drain(..5 + length);
    Some((flags, payload))
}

/// A project whose only repository is cloned at `root`, as Go's
/// `initWithRepo` helper makes one.
pub fn init_with_repo(
    store: &Store,
    name: &str,
    root: &str,
) -> Result<crate::domains::projects::Project, Box<dyn std::error::Error>> {
    let project = crate::compositions::wedge::init_project(
        store,
        crate::compositions::wedge::InitInput {
            name: name.to_string(),
            ..crate::compositions::wedge::InitInput::default()
        },
    )?;
    crate::compositions::wedge::ensure_repository(store, &project.id, root)?;
    Ok(project)
}

/// Replaces the values of `keys` anywhere in `value`, so volatile fields
/// compare.
pub fn mask(value: &mut Value, keys: &[String]) {
    match value {
        Value::Object(map) => {
            for (name, item) in map.iter_mut() {
                if keys.iter().any(|key| key == name) {
                    *item = Value::String("<masked>".to_string());
                } else {
                    mask(item, keys);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                mask(item, keys);
            }
        }
        _ => {}
    }
}
