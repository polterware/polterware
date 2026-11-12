//! MCP stdio to the core's `/mcp`.
//!
//! Claude Code, Codex, and Cursor launch `polterware mcp` as a stdio server.
//! The bridge forwards each JSON-RPC line to the core's `/mcp` endpoint
//! (Decision 0007), starting the core when needed and initializing again when
//! the core it was talking to exited. It opens no store.

use std::fmt;
use std::io::{self, BufRead, Read, Write};
use std::net::{SocketAddr, TcpStream};

use serde_json::Value;

use crate::platform::coreclient::{self, CLIENT_HEADER};
use crate::platform::runstate;
use crate::providers::mcp::SESSION_HEADER;

use super::output::{CliError, CliResult};

/// The core no longer knows the MCP session (it restarted or expired the
/// session); the bridge initializes again and retries.
const SESSION_LOST: &str = "mcp session lost";

/// The largest stdio line Go's scanner accepts.
const MAX_LINE: usize = 4 << 20;

/// Bridges MCP stdio to the core's `/mcp` endpoint.
pub fn run_mcp(db_path: &str) -> CliResult {
    let mut bridge = McpBridge::new(db_path);
    let result = bridge.run();
    bridge.close();
    result
}

/// One failed delivery to the core.
#[derive(Debug)]
enum BridgeError {
    SessionLost,
    Message(String),
    Io(io::Error),
}

impl fmt::Display for BridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BridgeError::SessionLost => formatter.write_str(SESSION_LOST),
            BridgeError::Message(text) => formatter.write_str(text),
            BridgeError::Io(err) => formatter.write_str(&err.to_string()),
        }
    }
}

impl From<io::Error> for BridgeError {
    fn from(err: io::Error) -> BridgeError {
        BridgeError::Io(err)
    }
}

impl From<coreclient::Error> for BridgeError {
    fn from(err: coreclient::Error) -> BridgeError {
        BridgeError::Message(err.to_string())
    }
}

/// What the core answered.
#[derive(Debug, Default)]
struct Reply {
    status: u16,
    session_id: String,
    body: Vec<u8>,
}

struct McpBridge {
    db_path: String,
    core: Option<runstate::Core>,
    session: String,
    initialize: Option<Vec<u8>>,
}

impl McpBridge {
    fn new(db_path: &str) -> McpBridge {
        McpBridge {
            db_path: db_path.to_string(),
            core: None,
            session: String::new(),
            initialize: None,
        }
    }

    /// Reads one message per line from stdin and answers it on stdout.
    fn run(&mut self) -> CliResult {
        let stdin = io::stdin();
        let mut input = stdin.lock();
        let mut line = Vec::new();
        loop {
            line.clear();
            let read = input.read_until(b'\n', &mut line)?;
            if read == 0 {
                return Ok(());
            }
            if line.len() > MAX_LINE {
                return Err(CliError::message("bufio.Scanner: token too long"));
            }
            let message = trim_space(&line).to_vec();
            if message.is_empty() {
                continue;
            }
            if is_initialize(&message) {
                self.initialize = Some(message.clone());
                self.session.clear();
            }
            let response = self.deliver(&message);
            if !response.is_empty() {
                let mut stdout = io::stdout().lock();
                stdout.write_all(trim_space(&response))?;
                stdout.write_all(b"\n")?;
                stdout.flush()?;
            }
        }
    }

    /// Delivers one message, initializing again when the core lost the
    /// session, and answers what stdout should carry (empty for a notification
    /// that could not be delivered).
    fn deliver(&mut self, message: &[u8]) -> Vec<u8> {
        match self.send(message) {
            Ok(response) => response,
            Err(err) => {
                if matches!(&err, BridgeError::SessionLost)
                    && self.initialize.is_some()
                    && !is_initialize(message)
                {
                    match self.reinitialize() {
                        Ok(()) => match self.send(message) {
                            Ok(response) => return response,
                            Err(err) => return error_response(message, &err),
                        },
                        Err(err) => return error_response(message, &err),
                    }
                }
                error_response(message, &err)
            }
        }
    }

    /// Posts one message, starting or re-finding the core when needed.
    fn send(&mut self, message: &[u8]) -> Result<Vec<u8>, BridgeError> {
        for _ in 0..2 {
            if self.core.is_none() {
                self.core = Some(coreclient::ensure(&self.db_path)?);
            }
            match self.post(message) {
                Ok(reply) => {
                    if !reply.session_id.is_empty() {
                        self.session = reply.session_id;
                    }
                    return match reply.status {
                        200 | 202 => Ok(reply.body),
                        404 => {
                            self.session.clear();
                            Err(BridgeError::SessionLost)
                        }
                        status => Err(BridgeError::Message(format!(
                            "core /mcp returned {status}: {}",
                            String::from_utf8_lossy(trim_space(&reply.body))
                        ))),
                    };
                }
                Err(_) => {
                    // The core exited (idle or replaced): find or start it again.
                    self.core = None;
                    if !self.session.is_empty() {
                        self.session.clear();
                        if !is_initialize(message) {
                            return Err(BridgeError::SessionLost);
                        }
                    }
                }
            }
        }
        Err(BridgeError::Message(format!(
            "core unavailable; see {}",
            runstate::log_path(&self.db_path).display()
        )))
    }

    /// Sends one POST or DELETE to the core and reads its answer.
    fn post(&self, message: &[u8]) -> io::Result<Reply> {
        let core = self
            .core
            .as_ref()
            .ok_or_else(|| io::Error::other("no core"))?;
        let (host, port) = endpoint(&core.endpoint)?;
        let address: SocketAddr = format!("{host}:{port}")
            .parse()
            .map_err(|err| io::Error::other(format!("core endpoint {}: {err}", core.endpoint)))?;
        let mut stream = TcpStream::connect_timeout(&address, std::time::Duration::from_secs(10))?;
        self.request(
            &mut stream,
            "POST",
            some_headers(&core.token, &self.session),
            message,
        )
    }

    /// Sends a prebuilt request and reads the whole response.
    fn request(
        &self,
        stream: &mut TcpStream,
        method: &str,
        headers: Vec<(String, String)>,
        body: &[u8],
    ) -> io::Result<Reply> {
        let (host, port) = endpoint(&self.core.as_ref().expect("core").endpoint)?;
        let mut request = format!("{method} /mcp HTTP/1.1\r\nHost: {host}:{port}\r\n");
        for (name, value) in headers {
            request.push_str(&format!("{name}: {value}\r\n"));
        }
        request.push_str(&format!("Content-Length: {}\r\n", body.len()));
        request.push_str("Connection: close\r\n\r\n");
        stream.write_all(request.as_bytes())?;
        stream.write_all(body)?;
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw)?;
        split_response(&raw)
    }

    /// Replays the client's initialize so tool calls continue after the core
    /// restarted; the replayed response is not forwarded.
    fn reinitialize(&mut self) -> Result<(), BridgeError> {
        self.session.clear();
        let initialize = self.initialize.clone().ok_or(BridgeError::SessionLost)?;
        self.send(&initialize)?;
        Ok(())
    }

    /// Closes the session after stdin ends.
    fn close(&mut self) {
        if self.session.is_empty() {
            return;
        }
        let Some(core) = &self.core else {
            return;
        };
        let Ok((host, port)) = endpoint(&core.endpoint) else {
            return;
        };
        let Ok(address) = format!("{host}:{port}").parse::<SocketAddr>() else {
            return;
        };
        let Ok(mut stream) =
            TcpStream::connect_timeout(&address, std::time::Duration::from_secs(2))
        else {
            return;
        };
        let mut headers = vec![(
            "Authorization".to_string(),
            format!("Bearer {}", core.token),
        )];
        headers.push((SESSION_HEADER.to_string(), self.session.clone()));
        let _ = self.request(&mut stream, "DELETE", headers, &[]);
    }
}

/// The headers a POST carries: JSON, the token, the client name, and the
/// session when one is open.
fn some_headers(token: &str, session: &str) -> Vec<(String, String)> {
    let mut headers = vec![
        ("Content-Type".to_string(), "application/json".to_string()),
        ("Authorization".to_string(), format!("Bearer {token}")),
        (CLIENT_HEADER.to_string(), "mcp".to_string()),
    ];
    if !session.is_empty() {
        headers.push((SESSION_HEADER.to_string(), session.to_string()));
    }
    headers
}

/// Reads the host and port of a loopback endpoint, as `coreclient` does.
fn endpoint(endpoint: &str) -> io::Result<(String, u16)> {
    let trimmed = endpoint.trim().trim_end_matches('/');
    let rest = trimmed
        .strip_prefix("http://")
        .ok_or_else(|| io::Error::other(format!("the core endpoint {trimmed} is not http")))?;
    match rest.rsplit_once(':') {
        Some((host, port)) => {
            let port = port
                .parse::<u16>()
                .map_err(|err| io::Error::other(format!("the core endpoint {trimmed}: {err}")))?;
            Ok((host.to_string(), port))
        }
        None => Ok((rest.to_string(), 80)),
    }
}

/// Splits one HTTP/1.1 response into the status, the session header, and the
/// body, decoding chunked framing when the core uses it.
fn split_response(response: &[u8]) -> io::Result<Reply> {
    let head_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| io::Error::other("the core closed the connection"))?;
    let head = String::from_utf8_lossy(&response[..head_end]);
    let mut lines = head.split("\r\n");
    let status = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| io::Error::other("the core sent no status line"))?;
    let mut session_id = String::new();
    let mut length: Option<usize> = None;
    let mut chunked = false;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        match name.trim().to_ascii_lowercase().as_str() {
            "content-length" => length = value.trim().parse::<usize>().ok(),
            "transfer-encoding" => {
                chunked = value.to_ascii_lowercase().contains("chunked");
            }
            name if name == SESSION_HEADER.to_ascii_lowercase() => {
                session_id = value.trim().to_string();
            }
            _ => {}
        }
    }
    let raw = &response[head_end + 4..];
    let body = if chunked {
        decode_chunked(raw)?
    } else {
        match length {
            Some(length) => raw[..length.min(raw.len())].to_vec(),
            None => raw.to_vec(),
        }
    };
    Ok(Reply {
        status,
        session_id,
        body,
    })
}

/// Decodes an HTTP/1.1 chunked body.
fn decode_chunked(raw: &[u8]) -> io::Result<Vec<u8>> {
    let mut body = Vec::new();
    let mut pos = 0;
    loop {
        let Some(line_end) = raw[pos..].windows(2).position(|window| window == b"\r\n") else {
            return Err(io::Error::other("malformed chunk"));
        };
        let line = String::from_utf8_lossy(&raw[pos..pos + line_end]);
        let size = usize::from_str_radix(line.split(';').next().unwrap_or("").trim(), 16)
            .map_err(|_| io::Error::other("malformed chunk size"))?;
        pos += line_end + 2;
        if size == 0 {
            return Ok(body);
        }
        let end = pos + size;
        if end > raw.len() {
            return Err(io::Error::other("truncated chunk"));
        }
        body.extend_from_slice(&raw[pos..end]);
        pos = end + 2;
    }
}

/// Go's `bytes.TrimSpace` for the ASCII whitespace MCP lines carry.
fn trim_space(bytes: &[u8]) -> &[u8] {
    let is_space = |byte: &u8| matches!(byte, b' ' | b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r');
    let start = bytes
        .iter()
        .position(|byte| !is_space(byte))
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|byte| !is_space(byte))
        .map_or(start, |index| index + 1);
    &bytes[start..end]
}

/// Reports whether a message is an MCP `initialize` request.
fn is_initialize(message: &[u8]) -> bool {
    match serde_json::from_slice::<Value>(message) {
        Ok(Value::Object(object)) => {
            object.get("method").and_then(Value::as_str) == Some("initialize")
        }
        _ => false,
    }
}

/// Answers a request the bridge could not deliver; notifications get no
/// answer.
fn error_response(message: &[u8], err: &BridgeError) -> Vec<u8> {
    let Ok(Value::Object(object)) = serde_json::from_slice::<Value>(message) else {
        return Vec::new();
    };
    let Some(id) = object.get("id") else {
        return Vec::new();
    };
    let body = crate::platform::json::compact(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": -32000, "message": err.to_string()},
    }));
    body.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn initialize_is_recognized_by_method() {
        assert!(is_initialize(
            br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#
        ));
        assert!(!is_initialize(
            br#"{"jsonrpc":"2.0","method":"tools/list"}"#
        ));
        assert!(!is_initialize(b"not json"));
    }

    #[test]
    fn a_failed_request_gets_a_json_rpc_error_and_a_notification_gets_none() {
        let response = error_response(
            br#"{"jsonrpc":"2.0","id":7,"method":"tools/call"}"#,
            &BridgeError::SessionLost,
        );
        assert_eq!(
            String::from_utf8(response).expect("utf8"),
            r#"{"error":{"code":-32000,"message":"mcp session lost"},"id":7,"jsonrpc":"2.0"}"#
        );
        assert!(
            error_response(
                br#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
                &BridgeError::SessionLost
            )
            .is_empty()
        );
    }

    #[test]
    fn an_explicit_null_id_is_still_answered() {
        let response = error_response(
            br#"{"jsonrpc":"2.0","id":null,"method":"tools/list"}"#,
            &BridgeError::Message("down".to_string()),
        );
        assert_eq!(
            String::from_utf8(response).expect("utf8"),
            r#"{"error":{"code":-32000,"message":"down"},"id":null,"jsonrpc":"2.0"}"#
        );
    }

    #[test]
    fn responses_are_split_into_status_session_and_body() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nMcp-Session-Id: s1\r\nContent-Length: 7\r\n\r\n{\"a\":1}";
        let reply = split_response(raw).expect("reply");
        assert_eq!(reply.status, 200);
        assert_eq!(reply.session_id, "s1");
        assert_eq!(reply.body, b"{\"a\":1}");
    }

    #[test]
    fn chunked_bodies_are_decoded() {
        let raw =
            b"HTTP/1.1 202 Accepted\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n0\r\n\r\n";
        let reply = split_response(raw).expect("reply");
        assert_eq!(reply.status, 202);
        assert_eq!(reply.body, b"hello");
    }

    #[test]
    fn trim_space_matches_bytes_trim_space_for_lines() {
        assert_eq!(trim_space(b"  {\"a\":1}\r\n"), b"{\"a\":1}");
        assert_eq!(trim_space(b"\n\t "), b"");
    }
}
