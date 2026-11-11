//! The Connect wire the local core speaks, shared by every handler.
//!
//! Unary calls are one `POST` with `application/json` in and out. Server streams
//! answer `application/connect+json` with enveloped messages: a flag byte, the
//! length as a big-endian `u32`, then the payload; the last envelope carries the
//! end-of-stream message (`{}` on success, the error otherwise). Errors are
//! Connect's JSON shape (`code`, `message`) with the status Connect maps the
//! code to, and bodies are protojson exactly as `connectrpc.com/connect` writes
//! them: lowerCamelCase field names, zero values left out, `int64` as a string.
//!
//! The generated messages of `crate::local_v1` are serde types that emit zero
//! values, so [`encode`] normalizes the JSON they produce instead of changing
//! the generated files.
//!
//! The HTTP layer is axum. A server-stream handler produces its messages into a
//! [`StreamSender`] that another thread drains while it runs: each message
//! becomes one envelope as the handler sends it, so the client reads the
//! messages while the stream is open, as connect-go delivers them. The
//! end-of-stream envelope is the last frame, written after the handler
//! returns. Dropping the response (a client that leaves) closes the channel,
//! so the handler's next send fails and it unwinds.

use std::pin::Pin;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use futures_util::stream::{self, Stream, StreamExt};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::sync::mpsc::error::{SendError, TrySendError};
use tokio::sync::{mpsc, oneshot};

use super::lifecycle::client_of;
use super::server::Core;

/// The content type of a unary call, in and out.
pub const JSON_CONTENT_TYPE: &str = "application/json";
/// The content type of a server stream.
pub const CONNECT_JSON_CONTENT_TYPE: &str = "application/connect+json";
/// The `Accept-Post` a unary handler answers with when the content type is
/// refused. The Go core also speaks protobuf and gRPC here; the port speaks
/// Connect JSON only, so it lists only that.
pub const UNARY_ACCEPT_POST: &str = "application/json";
/// The `Accept-Post` a streaming handler answers with.
pub const STREAM_ACCEPT_POST: &str = "application/connect+json";
/// The compression a client may ask for. Answers are never compressed here,
/// because neither the Desktop nor the CLI asks for it.
pub const ACCEPT_ENCODING: &str = "gzip";
/// The flag of the envelope that ends a stream.
pub const END_STREAM_FLAG: u8 = 0b0000_0010;
/// The largest request body the core reads.
const MAX_BODY: usize = 32 << 20;
/// Room for one stream's messages: every client sends its backlog at once.
const STREAM_BUFFER: usize = 4096;

/// The JSON names of the generated `int64` fields: protojson writes them as
/// strings and leaves a zero out.
const INT64_FIELDS: [&str; 5] = ["afterSeq", "nextOffset", "offset", "seq", "sequence"];

/// A Connect error code, with the name and HTTP status connect-go gives it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    Canceled,
    Unknown,
    InvalidArgument,
    DeadlineExceeded,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    ResourceExhausted,
    FailedPrecondition,
    Aborted,
    OutOfRange,
    Unimplemented,
    Internal,
    Unavailable,
    DataLoss,
    Unauthenticated,
}

impl Code {
    /// The wire name, as Connect's error JSON writes it.
    pub fn name(self) -> &'static str {
        match self {
            Code::Canceled => "canceled",
            Code::Unknown => "unknown",
            Code::InvalidArgument => "invalid_argument",
            Code::DeadlineExceeded => "deadline_exceeded",
            Code::NotFound => "not_found",
            Code::AlreadyExists => "already_exists",
            Code::PermissionDenied => "permission_denied",
            Code::ResourceExhausted => "resource_exhausted",
            Code::FailedPrecondition => "failed_precondition",
            Code::Aborted => "aborted",
            Code::OutOfRange => "out_of_range",
            Code::Unimplemented => "unimplemented",
            Code::Internal => "internal",
            Code::Unavailable => "unavailable",
            Code::DataLoss => "data_loss",
            Code::Unauthenticated => "unauthenticated",
        }
    }

    /// The HTTP status connect-go answers with, a literal for a literal.
    pub fn http_status(self) -> u16 {
        match self {
            Code::Canceled => 499,
            Code::Unknown => 500,
            Code::InvalidArgument => 400,
            Code::DeadlineExceeded => 504,
            Code::NotFound => 404,
            Code::AlreadyExists => 409,
            Code::PermissionDenied => 403,
            Code::ResourceExhausted => 429,
            Code::FailedPrecondition => 400,
            Code::Aborted => 409,
            Code::OutOfRange => 400,
            Code::Unimplemented => 501,
            Code::Internal => 500,
            Code::Unavailable => 503,
            Code::DataLoss => 500,
            Code::Unauthenticated => 401,
        }
    }

    /// Reads a wire name; unknown names are `None`.
    pub fn from_name(name: &str) -> Option<Code> {
        [
            Code::Canceled,
            Code::Unknown,
            Code::InvalidArgument,
            Code::DeadlineExceeded,
            Code::NotFound,
            Code::AlreadyExists,
            Code::PermissionDenied,
            Code::ResourceExhausted,
            Code::FailedPrecondition,
            Code::Aborted,
            Code::OutOfRange,
            Code::Unimplemented,
            Code::Internal,
            Code::Unavailable,
            Code::DataLoss,
            Code::Unauthenticated,
        ]
        .into_iter()
        .find(|code| code.name() == name)
    }
}

/// A Connect error: the code, and the message the client reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectError {
    pub code: Code,
    pub message: String,
}

impl ConnectError {
    pub fn new(code: Code, message: impl Into<String>) -> ConnectError {
        ConnectError {
            code,
            message: message.into(),
        }
    }

    /// The port of the package's `invalid`.
    pub fn invalid(message: impl Into<String>) -> ConnectError {
        ConnectError::new(Code::InvalidArgument, message)
    }

    /// The port of the package's `internal`.
    pub fn internal(message: impl Into<String>) -> ConnectError {
        ConnectError::new(Code::Internal, message)
    }

    pub fn not_found(message: impl Into<String>) -> ConnectError {
        ConnectError::new(Code::NotFound, message)
    }

    pub fn failed_precondition(message: impl Into<String>) -> ConnectError {
        ConnectError::new(Code::FailedPrecondition, message)
    }

    /// The error a handler that is not ported yet answers with, worded as
    /// connect-go's generated `Unimplemented…Handler` stubs word it.
    pub fn unimplemented(message: impl Into<String>) -> ConnectError {
        ConnectError::new(Code::Unimplemented, message)
    }

    pub fn unauthenticated(message: impl Into<String>) -> ConnectError {
        ConnectError::new(Code::Unauthenticated, message)
    }

    /// The error as Connect's JSON body: `{"code":"…","message":"…"}`.
    pub fn json(&self) -> String {
        format!(
            "{{\"code\":{},\"message\":{}}}",
            quote(self.code.name()),
            quote(&self.message)
        )
    }
}

impl std::fmt::Display for ConnectError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code.name(), self.message)
    }
}

impl std::error::Error for ConnectError {}

/// One call's request: the HTTP headers and the message.
pub struct ConnectRequest<Req> {
    pub headers: HeaderMap,
    pub message: Req,
}

impl<Req> ConnectRequest<Req> {
    /// The client that made the call, from the `Polterware-Client` header.
    pub fn client(&self) -> String {
        client_of(&self.headers)
    }
}

/// Where a server-stream handler sends its messages. A handler that runs long
/// (a change watch) keeps the handle and sends each message as it happens; it
/// stops when [`StreamSender::is_closed`] turns true, after the client left.
pub struct StreamSender<T> {
    sender: mpsc::Sender<T>,
}

impl<T> StreamSender<T> {
    fn new(sender: mpsc::Sender<T>) -> StreamSender<T> {
        StreamSender { sender }
    }

    /// Sends without waiting for room; the error says why the message was not
    /// queued.
    pub fn try_send(&self, message: T) -> Result<(), TrySendError<T>> {
        self.sender.try_send(message)
    }

    /// Waits for room and sends. A handler that must not drop an event uses it.
    pub fn blocking_send(&self, message: T) -> Result<(), SendError<T>> {
        self.sender.blocking_send(message)
    }

    /// Whether the client is gone: the response was dropped, so nothing reads
    /// the stream any more.
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }
}

impl<T> Clone for StreamSender<T> {
    fn clone(&self) -> StreamSender<T> {
        StreamSender {
            sender: self.sender.clone(),
        }
    }
}

/// A server stream in flight: its envelopes, in order, as the handler writes
/// them. The end-of-stream envelope is always the last frame.
pub struct StreamBody {
    pub frames: Pin<Box<dyn Stream<Item = Vec<u8>> + Send>>,
}

impl StreamBody {
    /// A stream that answers one end-of-stream envelope carrying `error`, for a
    /// call that failed before its handler ran.
    pub fn failed(error: ConnectError) -> StreamBody {
        StreamBody::one(end_stream(Some(&error)))
    }

    /// A stream of one frame.
    fn one(frame: Vec<u8>) -> StreamBody {
        StreamBody {
            frames: Box::pin(stream::iter(vec![frame])),
        }
    }
}

/// A unary handler: the JSON request body in, the JSON response body out.
pub type UnaryHandler =
    Arc<dyn Fn(Arc<Core>, &[u8], &HeaderMap) -> Result<Vec<u8>, ConnectError> + Send + Sync>;

/// A server-stream handler: the JSON request body in, the stream out.
pub type StreamHandler = Arc<dyn Fn(Arc<Core>, &[u8], &HeaderMap) -> StreamBody + Send + Sync>;

/// One route of one service.
#[derive(Clone)]
pub struct Route {
    pub path: &'static str,
    pub auth: bool,
    pub rpc: Rpc,
}

impl Route {
    /// A unary route, behind the session token.
    pub fn unary<Req, Res, F>(path: &'static str, handler: F) -> Route
    where
        Req: DeserializeOwned + Default + Send + 'static,
        Res: Serialize + Send + 'static,
        F: Fn(Arc<Core>, ConnectRequest<Req>) -> Result<Res, ConnectError> + Send + Sync + 'static,
    {
        Route {
            path,
            auth: true,
            rpc: Rpc::Unary(unary(handler)),
        }
    }

    /// A server-stream route, behind the session token.
    pub fn server_stream<Req, Res, F>(path: &'static str, handler: F) -> Route
    where
        Req: DeserializeOwned + Default + Send + 'static,
        Res: Serialize + Send + 'static,
        F: Fn(Arc<Core>, ConnectRequest<Req>, StreamSender<Res>) -> Result<(), ConnectError>
            + Send
            + Sync
            + 'static,
    {
        Route {
            path,
            auth: true,
            rpc: Rpc::ServerStream(server_stream(handler)),
        }
    }

    /// A route that needs no token, as the health check does.
    pub fn public(mut self) -> Route {
        self.auth = false;
        self
    }
}

/// The RPC behind a route.
#[derive(Clone)]
pub enum Rpc {
    Unary(UnaryHandler),
    ServerStream(StreamHandler),
}

/// Wraps a typed unary handler into the JSON-in, JSON-out shape the router
/// calls.
pub fn unary<Req, Res, F>(handler: F) -> UnaryHandler
where
    Req: DeserializeOwned + Default + Send + 'static,
    Res: Serialize + Send + 'static,
    F: Fn(Arc<Core>, ConnectRequest<Req>) -> Result<Res, ConnectError> + Send + Sync + 'static,
{
    Arc::new(
        move |core: Arc<Core>, body: &[u8], headers: &HeaderMap| -> Result<Vec<u8>, ConnectError> {
            let message: Req = decode(body)?;
            let response = handler(
                core,
                ConnectRequest {
                    headers: headers.clone(),
                    message,
                },
            )?;
            encode(&response)
        },
    )
}

/// Wraps a typed server-stream handler: every message it sends becomes one
/// envelope, written as soon as it is sent, and the stream ends with Connect's
/// end-of-stream message once the handler returns.
///
/// The handler runs on its own thread because it may block between messages
/// (a watch waits for the next change). The frames are read while it runs, so
/// nothing waits for it to finish.
pub fn server_stream<Req, Res, F>(handler: F) -> StreamHandler
where
    Req: DeserializeOwned + Default + Send + 'static,
    Res: Serialize + Send + 'static,
    F: Fn(Arc<Core>, ConnectRequest<Req>, StreamSender<Res>) -> Result<(), ConnectError>
        + Send
        + Sync
        + 'static,
{
    let handler = Arc::new(handler);
    Arc::new(
        move |core: Arc<Core>, body: &[u8], headers: &HeaderMap| -> StreamBody {
            let payload = match decode_stream_request(body) {
                Ok(payload) => payload.to_vec(),
                Err(err) => return StreamBody::failed(err),
            };
            let message: Req = if payload.is_empty() {
                Req::default()
            } else {
                match decode(&payload) {
                    Ok(message) => message,
                    Err(err) => return StreamBody::failed(err),
                }
            };
            let (sender, messages) = mpsc::channel(STREAM_BUFFER);
            let sender = StreamSender::new(sender);
            let (done, finished) = oneshot::channel();
            let handler = Arc::clone(&handler);
            let request = ConnectRequest {
                headers: headers.clone(),
                message,
            };
            let spawned = std::thread::Builder::new()
                .name("connect-stream".to_string())
                .spawn(move || {
                    let result = handler(core, request, sender);
                    // The client may have left; the result is then dropped.
                    let _ = done.send(result);
                });
            if let Err(err) = spawned {
                return StreamBody::failed(ConnectError::internal(format!(
                    "start stream handler: {err}"
                )));
            }
            StreamBody {
                frames: Box::pin(frames(messages, finished)),
            }
        },
    )
}

/// The stream's frames: one envelope per message, then the end-of-stream frame
/// carrying what the handler returned.
fn frames<Res>(
    messages: mpsc::Receiver<Res>,
    finished: oneshot::Receiver<Result<(), ConnectError>>,
) -> impl Stream<Item = Vec<u8>> + Send
where
    Res: Serialize + Send + 'static,
{
    /// The channel is done once the handler stopped; the end frame follows.
    enum State<Res> {
        Open {
            messages: mpsc::Receiver<Res>,
            finished: oneshot::Receiver<Result<(), ConnectError>>,
        },
        Ended,
    }

    stream::unfold(State::Open { messages, finished }, |state| async move {
        let (mut messages, finished) = match state {
            State::Open { messages, finished } => (messages, finished),
            State::Ended => return None,
        };
        match messages.recv().await {
            Some(message) => {
                let frame = match encode(&message) {
                    Ok(payload) => envelope(0, &payload),
                    Err(err) => return Some((end_stream(Some(&err)), State::Ended)),
                };
                Some((frame, State::Open { messages, finished }))
            }
            None => {
                let result = finished.await.unwrap_or_else(|_| {
                    Err(ConnectError::internal(
                        "stream handler ended without a result",
                    ))
                });
                let frame = match result {
                    Ok(()) => end_stream(None),
                    Err(err) => end_stream(Some(&err)),
                };
                Some((frame, State::Ended))
            }
        }
    })
}

/// A Connect envelope: the flag byte, the length as a big-endian `u32`, and the
/// payload.
pub fn envelope(flags: u8, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len() + 5);
    out.push(flags);
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(body);
    out
}

/// The end-of-stream envelope: `{}` when the stream ended cleanly, otherwise
/// the error the stream ended with.
pub fn end_stream(error: Option<&ConnectError>) -> Vec<u8> {
    let body = match error {
        None => "{}".to_string(),
        Some(error) => format!("{{\"error\":{}}}", error.json()),
    };
    envelope(END_STREAM_FLAG, body.as_bytes())
}

/// Reads the request message of a server-stream call: the payload of its first
/// envelope. An empty body is the default message, as an empty envelope is.
pub fn decode_stream_request(body: &[u8]) -> Result<&[u8], ConnectError> {
    if body.is_empty() {
        return Ok(&[]);
    }
    if body.len() < 5 {
        return Err(ConnectError::new(
            Code::Unknown,
            "protocol error: incomplete envelope: EOF",
        ));
    }
    let flags = body[0];
    let length = u32::from_be_bytes([body[1], body[2], body[3], body[4]]) as usize;
    if body.len() < 5 + length {
        return Err(ConnectError::new(
            Code::Unknown,
            "protocol error: incomplete envelope: EOF",
        ));
    }
    if flags & 0b0000_0001 != 0 {
        return Err(ConnectError::new(
            Code::Internal,
            "protocol error: sent compressed message without compression support",
        ));
    }
    if flags & END_STREAM_FLAG != 0 {
        return Err(ConnectError::new(
            Code::Unknown,
            "final message has protocol-specific flags: EOF",
        ));
    }
    Ok(&body[5..5 + length])
}

/// Encodes one message the way protojson does: lowerCamelCase names (the
/// generated types carry them), zero values left out, `int64` as a string.
pub fn encode<T: Serialize + ?Sized>(message: &T) -> Result<Vec<u8>, ConnectError> {
    let text = serde_json::to_string(message)
        .map_err(|err| ConnectError::internal(format!("marshal message: {err}")))?;
    let text = normalize(&text)
        .map_err(|err| ConnectError::internal(format!("marshal message: {err}")))?;
    Ok(text.into_bytes())
}

/// Decodes one message. A malformed body is `unknown`, as connect-go leaves an
/// uncoded codec error, and keeps its message.
pub fn decode<T: DeserializeOwned>(body: &[u8]) -> Result<T, ConnectError> {
    if body.is_empty() {
        return Err(ConnectError::new(
            Code::Unknown,
            "zero-length payload is not a valid JSON object",
        ));
    }
    serde_json::from_slice(body).map_err(|err| {
        ConnectError::new(
            Code::Unknown,
            format!("unmarshal into {}: {err}", std::any::type_name::<T>()),
        )
    })
}

/// Rewrites protojson's JSON: drops the fields serde emitted with a zero value,
/// in place and in order, so the bytes stay those protojson would write.
fn normalize(text: &str) -> Result<String, String> {
    let mut parser = Parser {
        bytes: text.as_bytes(),
        pos: 0,
    };
    let mut out = String::with_capacity(text.len());
    parser.value(&mut out, None)?;
    parser.skip_space();
    if parser.pos != text.len() {
        return Err("trailing characters".to_string());
    }
    Ok(out)
}

/// Whether a JSON value is a proto3 zero, which protojson leaves out.
enum Kind {
    Zero,
    Other,
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn skip_space(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    /// Reads one value into `out`, and reports whether it is a protobuf zero.
    /// `int64` names the field it is the value of, when it is a field.
    fn value(&mut self, out: &mut String, int64: Option<&str>) -> Result<Kind, String> {
        self.skip_space();
        match self
            .peek()
            .ok_or_else(|| "unexpected end of JSON".to_string())?
        {
            b'{' => {
                self.object(out)?;
                Ok(Kind::Other)
            }
            b'[' => self.array(out),
            b'"' => {
                let raw = self.string()?;
                let empty = raw == "\"\"";
                let zero = int64.is_some() && raw == "\"0\"";
                out.push_str(&raw);
                if empty || zero {
                    Ok(Kind::Zero)
                } else {
                    Ok(Kind::Other)
                }
            }
            b't' | b'f' | b'n' => {
                let word = self.word()?;
                out.push_str(word);
                if word == "false" || word == "null" {
                    Ok(Kind::Zero)
                } else {
                    Ok(Kind::Other)
                }
            }
            _ => {
                let number = self.number()?;
                let zero = number
                    .parse::<f64>()
                    .map(|value| value == 0.0)
                    .unwrap_or(false);
                if number.contains(['.', 'e', 'E']) {
                    // A double: Go's protojson writes 1.0 as 1.
                    if zero {
                        out.push('0');
                    } else if let Ok(value) = number.parse::<f64>() {
                        out.push_str(&value.to_string());
                    } else {
                        out.push_str(number);
                    }
                } else {
                    out.push_str(number);
                }
                if zero {
                    Ok(Kind::Zero)
                } else {
                    Ok(Kind::Other)
                }
            }
        }
    }

    /// One object: the fields protojson keeps, comma separated.
    fn object(&mut self, out: &mut String) -> Result<(), String> {
        self.pos += 1; // '{'
        out.push('{');
        let mut first = true;
        loop {
            self.skip_space();
            match self.peek() {
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                Some(b',') => {
                    self.pos += 1;
                    continue;
                }
                None => return Err("unterminated object".to_string()),
                _ => {}
            }
            let key = self.string()?;
            self.skip_space();
            if self.peek() != Some(b':') {
                return Err("expected a colon".to_string());
            }
            self.pos += 1;
            let name = &key[1..key.len().saturating_sub(1)];
            let int64 = INT64_FIELDS.iter().find(|field| **field == name).copied();
            let mut value = String::new();
            let kind = self.value(&mut value, int64)?;
            if matches!(kind, Kind::Zero) {
                continue;
            }
            if !first {
                out.push(',');
            }
            first = false;
            out.push_str(&key);
            out.push(':');
            out.push_str(&value);
        }
        out.push('}');
        Ok(())
    }

    /// One array; empty is a zero (protojson leaves it out).
    fn array(&mut self, out: &mut String) -> Result<Kind, String> {
        self.pos += 1; // '['
        out.push('[');
        let mut first = true;
        loop {
            self.skip_space();
            match self.peek() {
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                Some(b',') => {
                    self.pos += 1;
                    continue;
                }
                None => return Err("unterminated array".to_string()),
                _ => {}
            }
            let mut value = String::new();
            self.value(&mut value, None)?;
            if !first {
                out.push(',');
            }
            first = false;
            out.push_str(&value);
        }
        out.push(']');
        if first {
            Ok(Kind::Zero)
        } else {
            Ok(Kind::Other)
        }
    }

    /// A JSON string literal, escapes included, as it was written.
    fn string(&mut self) -> Result<String, String> {
        if self.peek() != Some(b'"') {
            return Err("expected a string".to_string());
        }
        let start = self.pos;
        self.pos += 1;
        while let Some(byte) = self.peek() {
            match byte {
                b'\\' => self.pos += 2,
                b'"' => {
                    self.pos += 1;
                    return std::str::from_utf8(&self.bytes[start..self.pos])
                        .map(str::to_string)
                        .map_err(|_| "invalid UTF-8".to_string());
                }
                _ => self.pos += 1,
            }
        }
        Err("unterminated string".to_string())
    }

    fn word(&mut self) -> Result<&'_ str, String> {
        let start = self.pos;
        while self.peek().is_some_and(|byte| byte.is_ascii_alphanumeric()) {
            self.pos += 1;
        }
        std::str::from_utf8(&self.bytes[start..self.pos]).map_err(|_| "invalid UTF-8".to_string())
    }

    fn number(&mut self) -> Result<&'_ str, String> {
        let start = self.pos;
        while self.peek().is_some_and(|byte| {
            byte.is_ascii_digit() || matches!(byte, b'-' | b'+' | b'.' | b'e' | b'E')
        }) {
            self.pos += 1;
        }
        if start == self.pos {
            return Err("expected a value".to_string());
        }
        std::str::from_utf8(&self.bytes[start..self.pos]).map_err(|_| "invalid UTF-8".to_string())
    }
}

/// A JSON string literal. protojson escapes the same characters serde does, so
/// delegating keeps the bytes identical.
fn quote(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string())
}

/// The content type with its parameters and case removed, as connect-go reads
/// it.
fn canonical_content_type(headers: &HeaderMap) -> String {
    let value = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let base = value.split(';').next().unwrap_or_default().trim();
    base.to_ascii_lowercase()
}

/// Checks the session token: the `Authorization` header must carry
/// `Bearer <token>`. The comparison does not stop at the first wrong byte.
pub fn authorize(core: &Core, headers: &HeaderMap) -> Result<(), ConnectError> {
    let value = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let matches = value
        .strip_prefix("Bearer ")
        .is_some_and(|candidate| constant_time_eq(candidate.as_bytes(), core.token.as_bytes()));
    if !matches {
        return Err(ConnectError::unauthenticated(
            "invalid or missing session token",
        ));
    }
    core.touch();
    Ok(())
}

/// Go's `subtle.ConstantTimeCompare`.
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (a, b) in left.iter().zip(right) {
        difference |= a ^ b;
    }
    difference == 0
}

/// Builds the router for every route, in the order given.
pub fn router(core: Arc<Core>, routes: Vec<Route>) -> axum::Router {
    let mut router = axum::Router::new();
    for route in routes {
        let path = route.path;
        let core = Arc::clone(&core);
        let route = Arc::new(route);
        router = router.route(
            path,
            axum::routing::post(move |request: Request| {
                let core = Arc::clone(&core);
                let route = Arc::clone(&route);
                async move { serve(core, route, request).await }
            }),
        );
    }
    router
}

/// Serves one call: the protocol checks, the token, the handler, and the
/// Connect answer.
async fn serve(core: Arc<Core>, route: Arc<Route>, request: Request) -> Response {
    let headers = request.headers().clone();
    let content_type = canonical_content_type(&headers);
    match &route.rpc {
        Rpc::Unary(handler) => {
            if content_type != JSON_CONTENT_TYPE {
                return unsupported(UNARY_ACCEPT_POST);
            }
            if route.auth
                && let Err(err) = authorize(&core, &headers)
            {
                return error_response(&err);
            }
            let body = match read_body(request).await {
                Ok(body) => body,
                Err(err) => return error_response(&err),
            };
            let _call = core.calls.begin(&client_of(&headers));
            let core = Arc::clone(&core);
            match handler(core, &body, &headers) {
                Ok(bytes) => {
                    let mut response = Response::new(Body::from(bytes));
                    let out = response.headers_mut();
                    out.insert(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(JSON_CONTENT_TYPE),
                    );
                    out.insert(
                        header::ACCEPT_ENCODING,
                        HeaderValue::from_static(ACCEPT_ENCODING),
                    );
                    response
                }
                Err(err) => error_response(&err),
            }
        }
        Rpc::ServerStream(handler) => {
            if content_type != CONNECT_JSON_CONTENT_TYPE {
                return unsupported(STREAM_ACCEPT_POST);
            }
            if route.auth
                && let Err(err) = authorize(&core, &headers)
            {
                return stream_response(StreamBody::failed(err));
            }
            let body = match read_body(request).await {
                Ok(body) => body,
                Err(err) => return stream_response(StreamBody::failed(err)),
            };
            stream_response(handler(core, &body, &headers))
        }
    }
}

/// Reads a request body, refusing one larger than the core accepts.
async fn read_body(request: Request) -> Result<Vec<u8>, ConnectError> {
    match axum::body::to_bytes(request.into_body(), MAX_BODY).await {
        Ok(body) => Ok(body.to_vec()),
        Err(err) => Err(ConnectError::internal(format!("read body: {err}"))),
    }
}

/// A Connect error answer: the status the code maps to, and the error as JSON.
fn error_response(err: &ConnectError) -> Response {
    let status =
        StatusCode::from_u16(err.code.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut response = (status, err.json()).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(JSON_CONTENT_TYPE),
    );
    response
}

/// A refused content type: `415` with the types the route accepts.
fn unsupported(accept_post: &'static str) -> Response {
    let mut response = StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();
    response.headers_mut().insert(
        axum::http::HeaderName::from_static("accept-post"),
        HeaderValue::from_static(accept_post),
    );
    response
}

/// A server stream's answer: the frames as the handler produces them, with the
/// end-of-stream frame last.
fn stream_response(body: StreamBody) -> Response {
    let frames = body.frames.map(Ok::<Vec<u8>, std::io::Error>);
    let mut response = Response::new(Body::from_stream(frames));
    let out = response.headers_mut();
    out.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(CONNECT_JSON_CONTENT_TYPE),
    );
    out.insert(
        axum::http::HeaderName::from_static("connect-accept-encoding"),
        HeaderValue::from_static(ACCEPT_ENCODING),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_v1;
    use std::time::Duration;

    #[test]
    fn codes_carry_connect_names_and_statuses() {
        assert_eq!(Code::Unauthenticated.name(), "unauthenticated");
        assert_eq!(Code::Unauthenticated.http_status(), 401);
        assert_eq!(Code::NotFound.http_status(), 404);
        assert_eq!(Code::Canceled.http_status(), 499);
        assert_eq!(Code::FailedPrecondition.http_status(), 400);
        assert_eq!(
            Code::from_name("invalid_argument"),
            Some(Code::InvalidArgument)
        );
        assert_eq!(Code::from_name("nope"), None);
    }

    #[test]
    fn errors_are_connect_json() {
        let error = ConnectError::invalid("project is required");
        assert_eq!(
            error.json(),
            r#"{"code":"invalid_argument","message":"project is required"}"#
        );
        assert_eq!(error.to_string(), "invalid_argument: project is required");
        let quoted = ConnectError::internal("a \"quoted\" reason\n");
        assert_eq!(
            quoted.json(),
            r#"{"code":"internal","message":"a \"quoted\" reason\n"}"#
        );
    }

    #[test]
    fn protojson_leaves_zero_values_out() {
        let response = local_v1::SaveProjectResponse {
            project: Some(local_v1::Project {
                id: "proj-1".to_string(),
                name: "One".to_string(),
                created_at: "2026-01-02T03:04:05Z".to_string(),
                ..local_v1::Project::default()
            }),
        };
        assert_eq!(
            String::from_utf8(encode(&response).unwrap()).unwrap(),
            r#"{"project":{"id":"proj-1","name":"One","createdAt":"2026-01-02T03:04:05Z"}}"#
        );
        let empty = local_v1::ListProjectsResponse::default();
        assert_eq!(String::from_utf8(encode(&empty).unwrap()).unwrap(), "{}");
    }

    #[test]
    fn protojson_keeps_counters_and_whole_repeated_fields() {
        let response = local_v1::ListAgentRunsResponse {
            runs: vec![
                local_v1::AgentRun::default(),
                local_v1::AgentRun {
                    id: "run-1".to_string(),
                    ..local_v1::AgentRun::default()
                },
            ],
        };
        assert_eq!(
            String::from_utf8(encode(&response).unwrap()).unwrap(),
            r#"{"runs":[{},{"id":"run-1"}]}"#
        );
    }

    #[test]
    fn protojson_writes_int64_as_a_string_and_drops_a_zero_one() {
        let response = local_v1::WatchChangesResponse {
            seq: 0,
            at: "now".to_string(),
            ..local_v1::WatchChangesResponse::default()
        };
        assert_eq!(
            String::from_utf8(encode(&response).unwrap()).unwrap(),
            r#"{"at":"now"}"#
        );
        let response = local_v1::WatchChangesResponse {
            seq: 7,
            ..local_v1::WatchChangesResponse::default()
        };
        assert_eq!(
            String::from_utf8(encode(&response).unwrap()).unwrap(),
            r#"{"seq":"7"}"#
        );
        // A string that happens to be "0" is kept unless it is an int64 field.
        let item = local_v1::KnowledgeItem {
            title: "0".to_string(),
            ..local_v1::KnowledgeItem::default()
        };
        assert_eq!(
            String::from_utf8(encode(&item).unwrap()).unwrap(),
            r#"{"title":"0"}"#
        );
    }

    #[test]
    fn protojson_writes_the_double_as_go_does() {
        let response = local_v1::CheckResponse {
            status: "ok".to_string(),
            uptime_sec: 1.0,
            pid: 0,
            ..local_v1::CheckResponse::default()
        };
        assert_eq!(
            String::from_utf8(encode(&response).unwrap()).unwrap(),
            r#"{"status":"ok","uptimeSec":1}"#
        );
        let response = local_v1::CheckResponse {
            uptime_sec: 0.5,
            ..local_v1::CheckResponse::default()
        };
        assert_eq!(
            String::from_utf8(encode(&response).unwrap()).unwrap(),
            r#"{"uptimeSec":0.5}"#
        );
    }

    #[test]
    fn protojson_keeps_escapes_and_unicode() {
        let item = local_v1::KnowledgeItem {
            title: "a \"quoted\" <tag> & ünïcode\n".to_string(),
            ..local_v1::KnowledgeItem::default()
        };
        let text = String::from_utf8(encode(&item).unwrap()).unwrap();
        assert_eq!(text, "{\"title\":\"a \\\"quoted\\\" <tag> & ünïcode\\n\"}");
        let read: local_v1::KnowledgeItem = decode(text.as_bytes()).unwrap();
        assert_eq!(read.title, item.title);
    }

    #[test]
    fn envelopes_frame_the_message() {
        assert_eq!(envelope(0, b"{}"), vec![0, 0, 0, 0, 2, b'{', b'}']);
        assert_eq!(
            end_stream(None),
            vec![END_STREAM_FLAG, 0, 0, 0, 2, b'{', b'}']
        );
        let error = ConnectError::unauthenticated("invalid or missing session token");
        let frame = end_stream(Some(&error));
        assert_eq!(frame[0], END_STREAM_FLAG);
        let length = u32::from_be_bytes([frame[1], frame[2], frame[3], frame[4]]) as usize;
        assert_eq!(
            std::str::from_utf8(&frame[5..5 + length]).unwrap(),
            r#"{"error":{"code":"unauthenticated","message":"invalid or missing session token"}}"#
        );
    }

    #[test]
    fn stream_requests_read_their_first_envelope() {
        let body = envelope(0, br#"{"afterSeq":"4"}"#);
        assert_eq!(
            decode_stream_request(&body).unwrap(),
            br#"{"afterSeq":"4"}"#
        );
        assert_eq!(decode_stream_request(&[]).unwrap(), b"");
        assert!(decode_stream_request(&[0, 0]).is_err());
    }

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime")
    }

    fn watch_message(seq: i64) -> local_v1::WatchChangesResponse {
        local_v1::WatchChangesResponse {
            seq,
            ..local_v1::WatchChangesResponse::default()
        }
    }

    #[test]
    fn a_stream_yields_each_message_before_the_handler_finishes() {
        let runtime = runtime();
        let (sender, receiver) = mpsc::channel(8);
        let (done, finished) = oneshot::channel();
        let mut stream = Box::pin(frames(receiver, finished));

        let sender = StreamSender::new(sender);
        sender.try_send(watch_message(1)).expect("room for one");
        let first = runtime
            .block_on(stream.next())
            .expect("a frame while the handler still runs");
        let length = u32::from_be_bytes([first[1], first[2], first[3], first[4]]) as usize;
        assert_eq!(first[0], 0);
        assert_eq!(
            std::str::from_utf8(&first[5..5 + length]).unwrap(),
            r#"{"seq":"1"}"#
        );

        // Nothing else is written while the handler keeps its end open.
        let waiting = runtime.block_on(async {
            tokio::time::timeout(Duration::from_millis(50), stream.next()).await
        });
        assert!(
            waiting.is_err(),
            "the stream waited for the handler, as it must"
        );

        drop(sender);
        done.send(Ok(())).expect("the stream is still read");
        let last = runtime.block_on(stream.next()).expect("an end stream");
        assert_eq!(last, end_stream(None));
        assert!(
            runtime.block_on(stream.next()).is_none(),
            "nothing after the end"
        );
    }

    #[test]
    fn a_stream_ends_with_the_error_its_handler_returned() {
        let runtime = runtime();
        let (sender, receiver) = mpsc::channel::<local_v1::WatchChangesResponse>(8);
        let (done, finished) = oneshot::channel();
        let mut stream = Box::pin(frames(receiver, finished));

        let sender = StreamSender::new(sender);
        sender.try_send(watch_message(2)).expect("room for one");
        drop(sender);
        done.send(Err(ConnectError::invalid("watch refused")))
            .expect("the stream is still read");

        let first = runtime.block_on(stream.next()).expect("the message");
        assert_eq!(first[0], 0);
        let last = runtime.block_on(stream.next()).expect("an end stream");
        assert_eq!(last[0], END_STREAM_FLAG);
        let length = u32::from_be_bytes([last[1], last[2], last[3], last[4]]) as usize;
        assert_eq!(
            std::str::from_utf8(&last[5..5 + length]).unwrap(),
            r#"{"error":{"code":"invalid_argument","message":"watch refused"}}"#
        );
        assert!(runtime.block_on(stream.next()).is_none());
    }

    #[test]
    fn an_unknown_field_is_discarded() {
        let request: local_v1::ListProjectsRequest =
            decode(br#"{"includeArchived":true,"unknown":"value"}"#).unwrap();
        assert!(request.include_archived);
    }

    #[test]
    fn a_bad_body_is_uncoded_like_connect() {
        let err = decode::<local_v1::ListProjectsRequest>(b"").unwrap_err();
        assert_eq!(err.code, Code::Unknown);
        assert_eq!(
            err.message,
            "zero-length payload is not a valid JSON object"
        );
    }

    #[test]
    fn tokens_compare_without_an_early_exit() {
        assert!(constant_time_eq(b"token", b"token"));
        assert!(!constant_time_eq(b"token", b"nope"));
        assert!(!constant_time_eq(b"token", b"tokens"));
    }
}
