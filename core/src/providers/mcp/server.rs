//! One MCP session, the JSON-RPC
//! messages it answers, and the line loop over stdio.

use std::io::{BufRead, BufReader, Read, Write};

use serde_json::{Map, Value, json};
use time::OffsetDateTime;

use crate::compositions::wedge;
use crate::domains::agentaccess;
use crate::providers::sqlite::Store;

use super::helpers::string_from;
use super::tools::tool_definitions;
use super::wire;
use super::{Error, Result};

/// The MCP server's name in `initialize`.
pub const SERVER_NAME: &str = "polterware-mcp";

/// The MCP server's version in `initialize`.
pub const SERVER_VERSION: &str = "0.6.0";

/// The protocol version this server speaks.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// What `initialize` tells the agent: the four tools and when to read what.
pub const INSTRUCTIONS: &str = "Polterware MCP: four tools. polter.query reads compact text: \"brief <slug|path|file>\" before you work on a card, node, or file (tasks, invariants, the Rules that bind it, milestone); \"search <terms>\" finds anything; \"show <id>\" opens one record; also \"commits <slug>\", \"status\", \"inbox\", \"cards\", \"handoff <slug>\", \"structure [<path>]\", \"rules [status]\", \"milestones\", \"roadmap\", and \"canvas [<name|id>]\" (agent-visible canvases only). polter.apply writes Work cards, Structure nodes, and Rules as Polter text with summary and rationale (read polter.format once; pass topic for the rest). doc.upsert writes a doc with a Markdown body. Writes join this session's run; there is nothing to start or complete.";

/// One line as Go's `bufio.Scanner` bounds it (the initial buffer is 64 KiB).
const MAX_LINE: usize = 1024 * 1024;

/// One MCP session: the client it serves, and the open AgentRun each write
/// joins.
///
/// The store is a parameter, not a field: the core holds the one SQLite
/// connection behind its own lock, and a session never outlives it.
#[derive(Debug, Clone, PartialEq)]
pub struct Server {
    pub id: String,
    pub client_name: String,
    pub client_version: String,
    pub started_at: OffsetDateTime,
    pub last_seen: OffsetDateTime,

    /// This MCP session's open agent run per project, for polter.apply and
    /// doc.upsert.
    pub session: wedge::SessionRuns,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            id: String::new(),
            client_name: String::new(),
            client_version: String::new(),
            started_at: OffsetDateTime::UNIX_EPOCH,
            last_seen: OffsetDateTime::UNIX_EPOCH,
            session: wedge::SessionRuns::default(),
        }
    }
}

/// One JSON-RPC request. `id` is `None` when the field is absent (a
/// notification) and `Some(Value::Null)` when the client sent `null`.
#[derive(Debug, Clone, Default, PartialEq)]
struct RpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

/// One JSON-RPC error, as Go writes it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RpcError {
    code: i32,
    message: String,
}

/// One JSON-RPC response, in Go's field order: jsonrpc, id, result, error.
#[derive(Debug, Clone, Default, PartialEq)]
struct RpcResponse {
    id: Option<Value>,
    result: Option<Value>,
    error: Option<RpcError>,
}

impl RpcResponse {
    /// Marshals the envelope the way Go's struct does: `omitempty` keeps an
    /// absent id, a nil result, and an absent error out.
    fn marshal(&self) -> Vec<u8> {
        let mut out = String::from("{\"jsonrpc\":\"2.0\"");
        if let Some(id) = &self.id {
            out.push_str(",\"id\":");
            wire::write(&mut out, id);
        }
        if let Some(result) = &self.result {
            out.push_str(",\"result\":");
            wire::write(&mut out, result);
        }
        if let Some(error) = &self.error {
            out.push_str(",\"error\":");
            wire::write(
                &mut out,
                &json!({"code": error.code, "message": error.message}),
            );
        }
        out.push('}');
        out.into_bytes()
    }
}

impl Server {
    /// A fresh session, the zero value Go builds with `&Server{}`.
    pub fn new() -> Server {
        Server::default()
    }

    /// Completes the agent runs this session opened.
    pub fn close(&mut self, store: &Store) {
        self.session.close(store);
    }

    /// Reads `initialize`'s client info; a payload without a name changes
    /// nothing.
    fn record_client(&mut self, params: Option<&Value>) {
        let Some(Value::Object(params)) = params else {
            return;
        };
        let Some(Value::Object(info)) = params.get("clientInfo") else {
            return;
        };
        let name = string_from(info.get("name"));
        if name.is_empty() {
            return;
        }
        self.client_version = string_from(info.get("version"));
        self.client_name = name.clone();
        self.session.label = name;
    }

    /// Reads one JSON-RPC message per line from `input` and writes responses to
    /// `output`: one message per line, and none for a notification.
    pub fn serve(
        &mut self,
        store: &Store,
        input: impl Read,
        output: &mut impl Write,
    ) -> std::io::Result<()> {
        let mut reader = BufReader::new(input);
        let mut line: Vec<u8> = Vec::new();
        loop {
            line.clear();
            if reader.read_until(b'\n', &mut line)? == 0 {
                return Ok(());
            }
            if line.len() > MAX_LINE {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "bufio.Scanner: token too long",
                ));
            }
            if line.last() == Some(&b'\n') {
                line.pop();
            }
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            if line.is_empty() {
                continue;
            }
            if let Some(body) = self.handle(store, &line) {
                output.write_all(&body)?;
                output.write_all(b"\n")?;
            }
        }
    }

    /// Answers one JSON-RPC message; `None` means it was a notification and
    /// nothing is written.
    pub(crate) fn handle(&mut self, store: &Store, line: &[u8]) -> Option<Vec<u8>> {
        let Some(request) = parse_request(line) else {
            return Some(
                RpcResponse {
                    error: Some(RpcError {
                        code: -32700,
                        message: "parse error".to_string(),
                    }),
                    ..RpcResponse::default()
                }
                .marshal(),
            );
        };
        if request.method == "notifications/initialized"
            || request.method == "notifications/cancelled"
        {
            return None;
        }
        let outcome = self.dispatch(store, &request);
        request.id.as_ref()?;
        Some(
            match outcome {
                Ok(result) => RpcResponse {
                    id: request.id,
                    result: Some(result),
                    error: None,
                },
                Err(error) => RpcResponse {
                    id: request.id,
                    result: None,
                    error: Some(RpcError {
                        code: -32000,
                        message: error.to_string(),
                    }),
                },
            }
            .marshal(),
        )
    }

    /// Answers one method. A tool that fails is reported in band, so an agent
    /// reads the message instead of a transport error.
    fn dispatch(&mut self, store: &Store, request: &RpcRequest) -> Result<Value> {
        match request.method.as_str() {
            "initialize" => {
                self.record_client(request.params.as_ref());
                Ok(json!({
                    "protocolVersion": PROTOCOL_VERSION,
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": SERVER_NAME, "version": SERVER_VERSION},
                    "instructions": INSTRUCTIONS,
                }))
            }
            "tools/list" => Ok(json!({"tools": tool_definitions()})),
            "tools/call" => {
                let (name, arguments) = tool_call_params(request.params.as_ref())?;
                Ok(match self.call_tool(store, &name, arguments) {
                    Ok(text) => json!({"content": [{"type": "text", "text": text}]}),
                    Err(error) => json!({
                        "content": [{"type": "text", "text": error.to_string()}],
                        "isError": true,
                    }),
                })
            }
            "ping" => Ok(json!({})),
            other => Err(Error::message(format!("method not found: {other}"))),
        }
    }

    /// Runs one tool and appends the call to the audit log it returns either
    /// way.
    pub(crate) fn call_tool(
        &mut self,
        store: &Store,
        name: &str,
        args: Map<String, Value>,
    ) -> Result<String> {
        let project_id = string_from(args.get("project_id"));
        let agent_run_id = string_from(args.get("agent_run_id"));
        let arguments_json = wire::marshal(&Value::Object(args.clone()));
        let outcome = self.invoke_tool(store, name, &args);
        let (result_summary, ok) = match &outcome {
            Ok(text) => (agentaccess::summarize_result(text, 240), true),
            Err(error) => (
                agentaccess::summarize_result(&error.to_string(), 240),
                false,
            ),
        };
        let _ = store.append_agent_tool_call(agentaccess::ToolCall {
            project_id,
            agent_run_id,
            tool_name: name.to_string(),
            arguments_json,
            result_summary,
            ok,
            source: "mcp".to_string(),
            ..agentaccess::ToolCall::default()
        });
        outcome
    }
}

/// Reads `tools/call`'s name and arguments the way Go's decoder does: a
/// mistyped field fails the whole message.
fn tool_call_params(params: Option<&Value>) -> Result<(String, Map<String, Value>)> {
    let params = match params {
        None | Some(Value::Null) => return Ok((String::new(), Map::new())),
        Some(Value::Object(map)) => map,
        Some(other) => {
            return Err(unmarshal_error(
                other,
                "params",
                "struct { Name string; Arguments map[string]interface {} }",
            ));
        }
    };
    let name = match params.get("name") {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(text)) => text.clone(),
        Some(other) => {
            return Err(unmarshal_error(other, "name", "string"));
        }
    };
    let arguments = match params.get("arguments") {
        None | Some(Value::Null) => Map::new(),
        Some(Value::Object(map)) => map.clone(),
        Some(other) => {
            return Err(unmarshal_error(
                other,
                "arguments",
                "map[string]interface {}",
            ));
        }
    };
    Ok((name, arguments))
}

/// Names a JSON value the way Go's decoder does in its unmarshal errors.
fn json_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn unmarshal_error(value: &Value, field: &str, kind: &str) -> Error {
    Error::message(format!(
        "json: cannot unmarshal {} into Go struct field .{field} of type {kind}",
        json_kind(value)
    ))
}

/// Parses one line into a request; `None` is Go's unmarshal failure, which
/// answers with a parse error. A `null` line and a missing field are not
/// failures there.
fn parse_request(line: &[u8]) -> Option<RpcRequest> {
    let value: Value = serde_json::from_slice(line).ok()?;
    let map = match value {
        Value::Null => return Some(RpcRequest::default()),
        Value::Object(map) => map,
        _ => return None,
    };
    let mut request = RpcRequest::default();
    match map.get("id") {
        None => {}
        Some(id) => request.id = Some(id.clone()),
    }
    match map.get("method") {
        None | Some(Value::Null) => {}
        Some(Value::String(method)) => request.method = method.clone(),
        Some(_) => return None,
    }
    match map.get("jsonrpc") {
        None | Some(Value::Null) => {}
        Some(Value::String(_)) => {}
        Some(_) => return None,
    }
    request.params = map.get("params").cloned();
    Some(request)
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use serde_json::json;
    use time::OffsetDateTime;

    use super::super::test_support::{ROADMAP, TestServer};
    use super::super::tools::{retired_tools, tool_definitions};
    use super::super::{TOOL_DOC_UPSERT, TOOL_POLTER_APPLY, TOOL_POLTER_FORMAT, TOOL_POLTER_QUERY};
    use super::*;
    use crate::domains::agentrun;
    use crate::domains::canvas;
    use crate::domains::knowledge;
    use crate::providers::sqlite::test_support::TempStore;

    /// `dispatch` for a method with no params, as Go's tests call it.
    fn dispatch(server: &mut TestServer, method: &str) -> Value {
        let request = RpcRequest {
            method: method.to_string(),
            ..RpcRequest::default()
        };
        let TestServer { server, fixture } = server;
        server
            .dispatch(&fixture.temp.store, &request)
            .expect("dispatch")
    }

    #[test]
    fn tools_list_is_a_stable_contract() {
        let mut server = TestServer::new();
        let result = dispatch(&mut server, "tools/list");
        let tools = result
            .get("tools")
            .and_then(Value::as_array)
            .expect("tools array");
        let want = [
            TOOL_POLTER_QUERY,
            TOOL_POLTER_APPLY,
            TOOL_POLTER_FORMAT,
            TOOL_DOC_UPSERT,
        ];
        assert_eq!(tools.len(), want.len(), "tools: {tools:?}");
        for (index, name) in want.iter().enumerate() {
            assert_eq!(
                tools[index].get("name").and_then(Value::as_str),
                Some(*name),
                "tool[{index}]"
            );
        }
    }

    #[test]
    fn initialize_keeps_its_version_and_instructions() {
        let mut server = TestServer::new();
        let payload = dispatch(&mut server, "initialize");
        let version = payload
            .get("serverInfo")
            .and_then(|info| info.get("version"))
            .and_then(Value::as_str);
        assert_eq!(version, Some(SERVER_VERSION));
        let instructions = payload
            .get("instructions")
            .and_then(Value::as_str)
            .expect("instructions");
        for want in [
            "polter.query",
            "polter.apply",
            "polter.format",
            "doc.upsert",
            "brief",
            "search",
            "show",
            "commits",
        ] {
            assert!(
                instructions.contains(want),
                "instructions missing {want:?}: {instructions}"
            );
        }
        for (name, _) in retired_tools() {
            assert!(
                !instructions.contains(name),
                "instructions still name retired {name}: {instructions}"
            );
        }
    }

    #[test]
    fn retired_tools_point_to_their_replacement() {
        let mut server = TestServer::new();
        for (name, _) in retired_tools() {
            let args = server.args(&[]);
            let error = match server.call(name, args) {
                Ok(text) => panic!("{name} answered {text}"),
                Err(error) => error,
            };
            assert!(
                error.0.contains(&format!("{name} was removed: use ")),
                "{name}: {error}"
            );
        }
        for name in ["search", "compile_context", "unknown"] {
            let args = server.args(&[]);
            let error = match server.call(name, args) {
                Ok(text) => panic!("{name} answered {text}"),
                Err(error) => error,
            };
            assert!(error.0.contains("unknown tool"), "{name}: {error}");
        }
    }

    #[test]
    fn polter_query_shows_one_record_and_commits() {
        let mut server = TestServer::new();
        for (id, want) in [
            (
                "decision-sqlite-local",
                "rule decision-sqlite-local [active] One local core owns SQLite.\nscope: whole project\n",
            ),
            (
                "fact-no-network",
                "rule fact-no-network [active] Imports never call the network.\nscope: whole project\n",
            ),
            ("rb-service", "doc rb-service \"Service recovery\"\n"),
            (
                "doc-architecture",
                "doc doc-architecture \"System architecture\"\n",
            ),
        ] {
            let text = server.query(&format!("show {id}"));
            assert!(text.contains(want), "show {id}:\n{text}\nwant {want:?}");
        }
        let args = server.query_args("show nope");
        let error = match server.call(TOOL_POLTER_QUERY, args) {
            Ok(text) => panic!("show nope answered {text}"),
            Err(error) => error,
        };
        assert!(error.0.contains("search <terms>"), "unknown id: {error}");

        let text = server.query("commits commits-panel");
        assert!(
            text.contains("commits for commits-panel"),
            "commits: {text}"
        );
    }

    #[test]
    fn doc_upsert_creates_updates_and_scopes_in_the_session_run() {
        let mut server = TestServer::new();
        let text = "---\npolter: 1\nsummary: Map auth\nrationale: Docs need a scope\n---\n\n# services: Services\nlevel: domain\n\n# services/auth: Auth\nlevel: component · under: services\n";
        let result = wedge::apply_polter(
            server.store(),
            wedge::ApplyInput {
                project_id: server.project_id().to_string(),
                auto_start_label: "cli".to_string(),
                text: text.to_string(),
                ..wedge::ApplyInput::default()
            },
        );
        assert!(result.is_ok(), "map auth: {result:?}");
        let seeded = server.list_runs();

        let args = server.args(&[("title", json!("No reason"))]);
        match server.call(TOOL_DOC_UPSERT, args) {
            Ok(text) => panic!("a doc write without summary and rationale answered {text}"),
            Err(error) => assert!(error.0.contains("pass summary and rationale"), "{error}"),
        }

        let args = server.args(&[
            ("summary", json!("Document sessions")),
            ("rationale", json!("Agents keep asking")),
            ("title", json!("Session handling")),
            ("body", json!("## Sessions\n\nCookies are signed.")),
            ("doc_type", json!("runbook")),
            ("tags", json!(["auth"])),
            ("scope", json!("services/auth")),
        ]);
        let created = server.call(TOOL_DOC_UPSERT, args).expect("create doc");
        assert!(
            created.starts_with("created doc ") && created.contains("\"Session handling\""),
            "create: {created}"
        );
        let id = created
            .split_whitespace()
            .nth(2)
            .expect("doc id")
            .to_string();
        let shown = server.query(&format!("show {id}"));
        for want in ["scope: services/auth", "tags: auth", "Cookies are signed."] {
            assert!(
                shown.contains(want),
                "show after create missing {want:?}:\n{shown}"
            );
        }

        let args = server.args(&[
            ("summary", json!("Clarify sessions")),
            ("rationale", json!("Rotation was missing")),
            ("id", json!(id)),
            ("body", json!("Cookies rotate daily.")),
            ("scope", json!("")),
        ]);
        server.call(TOOL_DOC_UPSERT, args).expect("update doc");
        let item = server.store().get_knowledge(&id).expect("doc after update");
        assert_eq!(item.title, "Session handling");
        assert_eq!(item.body, "Cookies rotate daily.");
        assert_eq!(item.doc_type, knowledge::DOC_TYPE_DOC);
        let shown = server.query(&format!("show {id}"));
        assert!(
            !shown.contains("scope:"),
            "empty scope must clear it:\n{shown}"
        );

        let args = server.args(&[
            ("summary", json!("x")),
            ("rationale", json!("y")),
            ("id", json!("decision-sqlite-local")),
            ("body", json!("overwrite")),
        ]);
        match server.call(TOOL_DOC_UPSERT, args) {
            Ok(text) => panic!("doc.upsert must not write a Decision: {text}"),
            Err(error) => assert!(error.0.contains("unknown doc"), "{error}"),
        }

        let runs = server.new_runs(&seeded);
        assert_eq!(
            runs.len(),
            1,
            "expected one session run, got {}",
            runs.len()
        );
        let record = server
            .store()
            .get_agent_run_record(&runs[0].id)
            .expect("run record");
        assert_eq!(record.steps.len(), 2, "steps: {:?}", record.steps);
        let payload: Value = serde_json::from_str(&record.steps[0].payload_json).expect("payload");
        assert_eq!(
            payload.get("tool").and_then(Value::as_str),
            Some(TOOL_DOC_UPSERT)
        );
    }

    #[test]
    fn polter_apply_writes_rules_the_brief_reads() {
        let mut server = TestServer::new();
        let text = "---\npolter: 1\nsummary: Add a rule\nrationale: Agents must keep secrets out of logs\n---\n\n#### rule no-secret-logs: Never log secrets.\n";
        let args = server.args(&[("text", json!(text))]);
        server.call(TOOL_POLTER_APPLY, args).expect("apply rule");
        let args = server.args(&[("text", json!(text))]);
        let again = server.call(TOOL_POLTER_APPLY, args).expect("reapply rule");
        assert!(
            again.contains("no changes"),
            "a rule is identified by its slug: {again}"
        );

        let shown = server.query("show no-secret-logs");
        assert!(
            shown.contains(
                "rule no-secret-logs [active] Never log secrets.\nscope: whole project\n"
            ),
            "show rule:\n{shown}"
        );

        let retired = "---\npolter: 1\nsummary: Back a fact\nrationale: x\n---\n\n#### one-core: One local core owns SQLite.\nevidence: file:backend/serve.go\n";
        let args = server.args(&[("text", json!(retired))]);
        match server.call(TOOL_POLTER_APPLY, args) {
            Ok(out) => assert!(
                out.contains("evidence: is retired"),
                "evidence must be refused: {out}"
            ),
            Err(error) => assert!(
                error.0.contains("evidence: is retired"),
                "evidence must be refused: {error}"
            ),
        }
    }

    /// Go keeps these two in `canvas_test.go`, which has no Rust file of its
    /// own: they exercise `tools/call` and `tools/list`.
    #[test]
    fn polter_query_canvas_hides_private() {
        let mut server = TestServer::new();
        let project_id = server.project_id().to_string();
        let shared = server
            .store()
            .create_canvas(
                canvas::Canvas {
                    id: "canvas-shared-public".to_string(),
                    project_id: project_id.clone(),
                    name: "Shared map".to_string(),
                    r#type: canvas::TYPE_CANVAS.to_string(),
                    agent_visible: true,
                    layout_json: r#"{"marker":"LAYOUT_ONLY_SECRET"}"#.to_string(),
                    graph: canvas::Graph {
                        nodes: vec![
                            canvas::Node {
                                id: "api".to_string(),
                                kind: "service".to_string(),
                                label: "Public API".to_string(),
                                r#ref: Some(canvas::Ref {
                                    kind: canvas::REF_FACT.to_string(),
                                    id: "fact-1".to_string(),
                                }),
                                ..canvas::Node::default()
                            },
                            canvas::Node {
                                id: "db".to_string(),
                                kind: "database".to_string(),
                                label: "Store".to_string(),
                                ..canvas::Node::default()
                            },
                            canvas::Node {
                                id: "note".to_string(),
                                kind: canvas::KIND_NOTE.to_string(),
                                label: "Sketch".to_string(),
                                ..canvas::Node::default()
                            },
                        ],
                        ..canvas::Graph::default()
                    },
                    ..canvas::Canvas::default()
                },
                "run-1",
                "batch-shared",
            )
            .expect("shared canvas");
        let private = server
            .store()
            .create_canvas(
                canvas::Canvas {
                    id: "canvas-private-secret".to_string(),
                    project_id,
                    name: "Private board".to_string(),
                    r#type: canvas::TYPE_CANVAS.to_string(),
                    agent_visible: false,
                    graph: canvas::Graph {
                        nodes: vec![canvas::Node {
                            id: "hid".to_string(),
                            kind: canvas::KIND_NOTE.to_string(),
                            label: "SECRET_CANVAS_NODE".to_string(),
                            ..canvas::Node::default()
                        }],
                        ..canvas::Graph::default()
                    },
                    ..canvas::Canvas::default()
                },
                "run-1",
                "batch-private",
            )
            .expect("private canvas");

        let list = server.query("canvas");
        assert!(
            list.contains(&shared.id) && list.contains("Shared map"),
            "list missing shared canvas:\n{list}"
        );
        assert!(
            !list.contains(&private.id)
                && !list.contains("Private board")
                && !list.contains("SECRET_CANVAS_NODE"),
            "private canvas leaked into the list:\n{list}"
        );

        for query in [
            format!("canvas {}", private.id),
            "canvas Private board".to_string(),
        ] {
            let args = server.query_args(&query);
            let error = match server.call(TOOL_POLTER_QUERY, args) {
                Ok(text) => panic!("{query} returned {text}"),
                Err(error) => error,
            };
            assert!(
                !error.0.contains("SECRET_CANVAS_NODE") && !error.0.contains("agent_visible"),
                "{query} leaked the private canvas: {error}"
            );
            assert!(error.0.contains("unknown canvas"), "{query}: {error}");
        }

        let body = server.query("canvas Shared map");
        let want = "flowchart LR\n  n1[\"Public API · service\"]\n  n2[\"Store · database\"]\n  %% note: Sketch\n  class n2 unverified\n";
        assert_eq!(body, want, "export:\n{body}\nwant:\n{want}");
        assert!(
            !body.contains("LAYOUT_ONLY_SECRET")
                && !body.contains("layout")
                && !body.contains("SECRET_CANVAS_NODE"),
            "export leaked layout or the private canvas:\n{body}"
        );
        let by_id = server.query(&format!("canvas {}", shared.id));
        assert_eq!(by_id, body);
    }

    #[test]
    fn polter_query_mentions_canvas() {
        // The description stays short; the query field lists every read, canvas
        // included.
        let mut description = String::new();
        let mut query = String::new();
        for tool in tool_definitions() {
            if tool.get("name").and_then(Value::as_str) != Some(TOOL_POLTER_QUERY) {
                continue;
            }
            description = tool
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            query = tool
                .get("inputSchema")
                .and_then(|schema| schema.get("properties"))
                .and_then(|properties| properties.get("query"))
                .and_then(|field| field.get("description"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
        }
        assert!(!description.is_empty(), "description: {description}");
        assert!(query.contains("canvas [<name|id>]"), "query: {query}");

        let mut server = TestServer::new();
        let instructions = dispatch(&mut server, "initialize")
            .get("instructions")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        assert!(
            instructions.contains("canvas [<name|id>]") && instructions.contains("agent-visible"),
            "instructions missing canvas: {instructions}"
        );
    }

    #[test]
    fn the_stdio_loop_frames_the_go_server_byte_for_byte() {
        // The fixtures under `testdata/` are recorded wire replies: the same
        // lines into `Server::serve` and the bytes it writes back, which must
        // stay identical.
        let TempStore { store, .. } = TempStore::new();
        let mut server = Server::new();
        let initialize = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"clientInfo\":{\"name\":\"fixture\",\"version\":\"1.0.0\"}}}";
        let tools_list = "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\"}";
        let input = format!("{initialize}\n{tools_list}\n");
        let mut out: Vec<u8> = Vec::new();
        server
            .serve(&store, input.as_bytes(), &mut out)
            .expect("serve");
        let expected = format!(
            "{}{}",
            include_str!("testdata/initialize.json"),
            include_str!("testdata/tools_list.json")
        );
        assert_eq!(String::from_utf8(out).expect("utf-8"), expected);
    }

    #[test]
    fn tool_call_replies_match_the_go_server_byte_for_byte() {
        // The fixture also covers the in-band error: the text of a tool that
        // fails carries `isError`, and every reply keeps Go's key order.
        let TempStore { store, .. } = TempStore::new();
        let mut server = Server::new();
        let input = concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"polter.format\",\"arguments\":{}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"no.such.tool\",\"arguments\":{}}}\n",
        );
        let mut out: Vec<u8> = Vec::new();
        server
            .serve(&store, input.as_bytes(), &mut out)
            .expect("serve");
        assert_eq!(
            String::from_utf8(out).expect("utf-8"),
            include_str!("testdata/tools_call.json")
        );
    }

    #[test]
    fn the_stdio_loop_frames_errors_and_notifications_like_go() {
        let TempStore { store, .. } = TempStore::new();
        let mut server = Server::new();
        let input = concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"ping\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"no/such\"}\n",
            "{not json\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"tools/list\"}\n",
        );
        let mut out: Vec<u8> = Vec::new();
        server
            .serve(&store, input.as_bytes(), &mut out)
            .expect("serve");
        assert_eq!(
            String::from_utf8(out).expect("utf-8"),
            include_str!("testdata/framing.json")
        );
    }

    #[test]
    fn an_empty_line_is_skipped_and_a_line_without_a_newline_still_answers() {
        let TempStore { store, .. } = TempStore::new();
        let mut server = Server::new();
        let input = "\n\r\n{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"ping\"}";
        let mut out: Vec<u8> = Vec::new();
        server
            .serve(&store, input.as_bytes(), &mut out)
            .expect("serve");
        assert_eq!(
            String::from_utf8(out).expect("utf-8"),
            "{\"jsonrpc\":\"2.0\",\"id\":7,\"result\":{}}\n"
        );
    }

    #[test]
    fn a_run_is_completed_when_its_session_closes() {
        let mut server = TestServer::new();
        let args = server.args(&[("text", json!(ROADMAP))]);
        server.call(TOOL_POLTER_APPLY, args).expect("apply");
        let runs = server.list_runs();
        let opened = runs
            .iter()
            .find(|run| run.status == agentrun::STATUS_OPEN)
            .expect("an open run");
        let id = opened.id.clone();
        server.close_session();
        let run = server.store().get_agent_run(&id).expect("run after close");
        assert_eq!(run.status, agentrun::STATUS_COMPLETED);
    }

    #[test]
    fn the_session_records_the_client_that_initialized_it() {
        let TempStore { store, .. } = TempStore::new();
        let mut server = Server::new();
        let line = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"codex","version":"0.9.1"}}}"#;
        let body = server.handle(&store, line).expect("a reply");
        let reply: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(reply.get("id"), Some(&json!(1)));
        assert_eq!(server.client_name, "codex");
        assert_eq!(server.client_version, "0.9.1");
        assert_eq!(server.session.label, "codex");
    }

    #[test]
    fn a_request_without_a_client_name_changes_nothing() {
        let TempStore { store, .. } = TempStore::new();
        let mut server = Server::new();
        let line = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        server.handle(&store, line).expect("a reply");
        assert!(server.client_name.is_empty());
        assert!(server.session.label.is_empty());
    }

    #[test]
    fn a_notification_writes_nothing() {
        // No store is reached: the method is dropped before dispatch.
        let TempStore { store, .. } = TempStore::new();
        let mut server = Server::new();
        assert!(
            server
                .handle(
                    &store,
                    br#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#
                )
                .is_none()
        );
        assert!(
            server
                .handle(
                    &store,
                    br#"{"jsonrpc":"2.0","method":"notifications/cancelled"}"#
                )
                .is_none()
        );
    }

    #[test]
    fn a_malformed_line_is_a_parse_error() {
        let TempStore { store, .. } = TempStore::new();
        let mut server = Server::new();
        let body = server.handle(&store, b"{not json").expect("a reply");
        assert_eq!(
            String::from_utf8(body).expect("utf-8"),
            r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"parse error"}}"#
        );
    }

    #[test]
    fn a_null_line_is_not_a_parse_error() {
        let TempStore { store, .. } = TempStore::new();
        let mut server = Server::new();
        // Go unmarshals null into the request without an error; the method is
        // empty, and with no id there is nothing to answer.
        assert!(server.handle(&store, b"null").is_none());
    }

    #[test]
    fn an_explicit_null_id_is_answered() {
        let TempStore { store, .. } = TempStore::new();
        let mut server = Server::new();
        let body = server
            .handle(&store, br#"{"jsonrpc":"2.0","id":null,"method":"ping"}"#)
            .expect("a reply");
        assert_eq!(
            String::from_utf8(body).expect("utf-8"),
            r#"{"jsonrpc":"2.0","id":null,"result":{}}"#
        );
    }

    #[test]
    fn every_tool_call_is_appended_to_the_audit_log() {
        let mut server = TestServer::new();
        let args = server.args(&[("query", json!("status"))]);
        server.call(TOOL_POLTER_QUERY, args).expect("query");
        let calls = server.store().list_agent_tool_calls(10).expect("calls");
        assert_eq!(calls.len(), 1, "calls: {calls:?}");
        assert_eq!(calls[0].tool_name, TOOL_POLTER_QUERY);
        assert_eq!(calls[0].source, "mcp");
        assert!(calls[0].ok);
        assert!(!calls[0].result_summary.is_empty());
        // The arguments are kept as Go marshals them: keys sorted.
        assert_eq!(
            calls[0].arguments_json,
            format!(
                "{{\"project_id\":\"{}\",\"query\":\"status\"}}",
                server.project_id()
            )
        );
        assert!(calls[0].created_at > OffsetDateTime::UNIX_EPOCH);
    }
}
