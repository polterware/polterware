//! Port of `backend/runtimes/cli/output.go` and `main.go`'s usage text.
//!
//! The CLI answers with Go-shaped text: `writeJSON` (encoding/json with a
//! two-space indent), `writeProtoJSON` (protojson with proto field names and
//! unpopulated fields), `exitError` and `fail`, and the usage every command
//! shares.

use std::io::{self, Write};

use serde::Serialize;

use crate::platform::coreclient;

/// How a command ends without a successful result.
///
/// The port of Go's two error paths: `exitError`, which leaves the process with
/// a code after the command printed its own output, and every other error,
/// which `fail` prints as `polterware: <message>` with exit 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    /// The port of `exitError`: no message, just the code.
    Exit(i32),
    /// A message `fail` prints with the `polterware: ` prefix.
    Message(String),
}

impl CliError {
    /// A plain message, as `fmt.Errorf` and `errors.New` build one.
    pub fn message(text: impl Into<String>) -> CliError {
        CliError::Message(text.into())
    }
}

impl From<coreclient::Error> for CliError {
    fn from(err: coreclient::Error) -> CliError {
        CliError::Message(err.to_string())
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::Message(err.to_string())
    }
}

impl From<crate::platform::runstate::Error> for CliError {
    fn from(err: crate::platform::runstate::Error) -> CliError {
        CliError::Message(err.to_string())
    }
}

impl From<crate::providers::sqlite::Error> for CliError {
    fn from(err: crate::providers::sqlite::Error) -> CliError {
        CliError::Message(err.to_string())
    }
}

pub type CliResult = Result<(), CliError>;

/// Ends a command the way Go's `fail` does: an `exitError` keeps its code and
/// prints nothing more; any other error prints `polterware: <message>` and
/// exits 1.
pub fn fail(err: CliError) -> i32 {
    match err {
        CliError::Exit(code) => code,
        CliError::Message(text) => {
            eprintln!("polterware: {text}");
            1
        }
    }
}

/// The usage every command shares, byte for byte as Go prints it.
pub fn usage_text() -> &'static str {
    r#"Usage:
  polterware [-db path] init [--name <name>] [--dir <clone> | --no-dir] [--id <id>] [--json]
  polterware [-db path] repo list --project <id> [--json]
  polterware [-db path] brief <card slug|Structure path|file> [--budget N] [--project <id>]   (read before working on it)
  polterware [-db path] search "<terms>" [--budget N] [--scope <node>[,<node>]] [--exact] [--project <id>]   (context is an alias)
  polterware [-db path] structure [--node <path>] [--at <point>] [--project <id>]
  polterware [-db path] structure history [--project <id>]
  polterware [-db path] canvas ls [--agents] [--scope <node>[,<node>]] [--exact] [--project <id>]
  polterware [-db path] canvas show <id|name> [--format mermaid|json|dot] [--project <id>]
  polterware [-db path] canvas share <id|name> on|off [--project <id>]
  polterware [-db path] export [--dry-run] [--sync on|off] [--json] [--project <id>]
  polterware [-db path] apply <file|-> [--dry-run] [--project <id>] [--summary ...] [--rationale ...] [--agent-run-id ...]
  polterware [-db path] lint <file|dir|-> [--project <id>]   (silent on success)
  polterware [-db path] format [structure|rules|decisions|facts|feature]   (the Polter format core, or one topic)
  polterware [-db path] query "cards" | "card <slug>" | "status" | "inbox" | "handoff <slug>" | "brief <target>" | "search <terms>" | "show <id>" | "commits <slug>" | "conflicts" | "conflict <slug>" | "structure [<path>] [at <point>]" | "structure history" | "decisions [state]" | "decision <decision>" | "feature <slug> [step]" | "verify <slug>" | "milestones" | "roadmap" | "canvas [<name|id>]" [--scope <node>[,<node>]] [--exact] [--project <id>]
  polterware [-db path] status [--scope <node>[,<node>]] [--exact] [--project <id>]
  polterware [-db path] handoff [--card <slug>] [--note "..."] [--project <id>]
  polterware [-db path] task claim|done|release <slug>#T2 [--agent name] [--commit <sha>] [--project <id>]
  polterware [-db path] decision list [state] | show <decision> | propose "<title>" | activate <decision> | challenge <decision> --reason "..." | supersede <decision> (--by <decision> | --title "...") | trigger <decision> "..." [--project <id>]
  polterware [-db path] feature new "<intent>" [--title "..."] [--under <node>] [--project <id>]
  polterware [-db path] feature shape <slug> | contract <slug> [--accept] | plan <slug> [--approve] [--project <id>]
  polterware [-db path] verify <slug|slug#T2> [--project <id>]   (exit 1 when a check fails)
  polterware [-db path] inbox [<id> decide "<answer>" | delegate | defer | ignore] [--agent name] [--scope <node>[,<node>]] [--exact] [--project <id>]
  polterware [-db path] conflict raise <slug> "<request>" --rule <slug> | --decision <ref> [--card <slug>] [--option "<text>"]... | decide <slug> "<answer>" | wait <slug> [--timeout 30m] | withdraw <slug> | list [--project <id>]
  polterware [-db path] deploys [<path>] | check [<path>] [--force] | targets <provider> [--node <path>] [--project <id>]   (deploy links, the platform's latest deploy, what a deploy can link to)
  polterware [-db path] scope <decision|fact|rule|item> <id> [<node path>...] [--project <id>]   (the nodes replace its contexts; none clears them)
  polterware [-db path] setup [doctor] | claude [--uninstall | --enable | --disable | --skill-only] | codex [--uninstall] [--project <id>]
  polterware [-db path] setup mcp [list] [--json] | add <claude|codex|cursor|all> | remove <claude|codex|cursor|all>   (register the MCP server with each client)
  polterware [-db path] hooks [list] | enable|disable <status|stop|lint|context|session|all>
  polterware [-db path] hook session-start [--label name] | session-end | lint | context | status | stop   (Claude Code hooks; JSON on stdin)
  polterware [-db path] run|runs|wait|result|cancel|merge|route|tiers|usage|...   (moved to Polterware Dispatch: runs dispatch <command> when it is installed)
  polterware [-db path] serve [--listen addr] [--token token]   (local core server for the Desktop)
  polterware [-db path] mcp                                     (MCP server over stdio for coding agents)
"#
}

/// Prints the usage to stderr, as Go's `usage` does.
pub fn usage() {
    eprint!("{}", usage_text());
}

/// A stub command: prints `<command>: not ported yet` to stderr and ends with
/// exit code 2 until its Go file is ported.
pub fn not_ported(command: &str) -> CliResult {
    eprintln!("{command}: not ported yet");
    Err(CliError::Exit(2))
}

/// The JSON text Go's `json.Encoder` writes with a two-space indent: sorted
/// keys for maps (serde_json's default map), `<`, `>`, `&`, U+2028, and U+2029
/// escaped as Go escapes them, and a trailing newline.
pub fn json_string<T: Serialize + ?Sized>(value: &T) -> Result<String, CliError> {
    let mut text =
        serde_json::to_string_pretty(value).map_err(|err| CliError::Message(err.to_string()))?;
    text = go_escape_html(&text);
    text.push('\n');
    Ok(text)
}

/// Prints a value as `writeJSON` does.
pub fn write_json<T: Serialize + ?Sized>(value: &T) -> CliResult {
    let text = json_string(value)?;
    let mut out = io::stdout().lock();
    out.write_all(text.as_bytes())?;
    out.flush()?;
    Ok(())
}

/// The protojson text Go writes with `Multiline: true, Indent: "  ",
/// UseProtoNames: true, EmitUnpopulated: true`: two-space indentation, proto
/// field names, and every field the generated Rust type holds, plus a trailing
/// newline.
///
/// A nil message field is the one difference: protojson writes `null` for it
/// under `EmitUnpopulated`, while the generated Rust type's `Option` is skipped
/// before this function sees it. The CLI callers today (`repo list --json` and
/// `export --json`) print repeated fields only, so their bytes match Go; a
/// caller that prints a message field must pass it filled in.
pub fn proto_json_string<T: Serialize + ?Sized>(message: &T) -> Result<String, CliError> {
    let compact =
        serde_json::to_string(message).map_err(|err| CliError::Message(err.to_string()))?;
    let mut printer = JsonPrinter {
        bytes: compact.as_bytes(),
        pos: 0,
    };
    let mut out = String::with_capacity(compact.len() * 2);
    printer.value(&mut out, 0)?;
    printer.skip_space();
    if printer.pos != compact.len() {
        return Err(CliError::Message("trailing characters".to_string()));
    }
    out.push('\n');
    Ok(out)
}

/// Prints a core response as `writeProtoJSON` does.
pub fn write_proto_json<T: Serialize + ?Sized>(message: &T) -> CliResult {
    let text = proto_json_string(message)?;
    let mut out = io::stdout().lock();
    out.write_all(text.as_bytes())?;
    out.flush()?;
    Ok(())
}

/// Go's `encoding/json` HTML escaping: the five characters it never writes
/// literally inside a string.
fn go_escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            ch => out.push(ch),
        }
    }
    out
}

/// Rewrites a compact JSON value into protojson's multiline layout, with
/// lowerCamelCase keys turned back into proto field names.
struct JsonPrinter<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl JsonPrinter<'_> {
    fn skip_space(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn value(&mut self, out: &mut String, depth: usize) -> Result<(), CliError> {
        self.skip_space();
        match self.peek() {
            Some(b'{') => self.object(out, depth),
            Some(b'[') => self.array(out, depth),
            Some(b'"') => {
                out.push_str(&self.string()?);
                Ok(())
            }
            Some(_) => {
                out.push_str(&self.word());
                Ok(())
            }
            None => Err(CliError::Message("unexpected end of JSON".to_string())),
        }
    }

    fn object(&mut self, out: &mut String, depth: usize) -> Result<(), CliError> {
        self.pos += 1;
        self.skip_space();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            out.push_str("{}");
            return Ok(());
        }
        out.push_str("{\n");
        loop {
            self.skip_space();
            let key = self.string()?;
            let key: String = serde_json::from_str(&key)
                .map_err(|err| CliError::Message(format!("key: {err}")))?;
            let key = proto_name(&key);
            self.skip_space();
            if self.peek() != Some(b':') {
                return Err(CliError::Message("expected a colon".to_string()));
            }
            self.pos += 1;
            out.push_str(&indent(depth + 1));
            out.push_str(&serde_json::to_string(&key).map_err(json_error)?);
            out.push_str(": ");
            self.value(out, depth + 1)?;
            self.skip_space();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                    out.push_str(",\n");
                }
                Some(b'}') => {
                    self.pos += 1;
                    out.push('\n');
                    out.push_str(&indent(depth));
                    out.push('}');
                    return Ok(());
                }
                _ => return Err(CliError::Message("expected a comma".to_string())),
            }
        }
    }

    fn array(&mut self, out: &mut String, depth: usize) -> Result<(), CliError> {
        self.pos += 1;
        self.skip_space();
        if self.peek() == Some(b']') {
            self.pos += 1;
            out.push_str("[]");
            return Ok(());
        }
        out.push_str("[\n");
        loop {
            out.push_str(&indent(depth + 1));
            self.value(out, depth + 1)?;
            self.skip_space();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                    out.push_str(",\n");
                }
                Some(b']') => {
                    self.pos += 1;
                    out.push('\n');
                    out.push_str(&indent(depth));
                    out.push(']');
                    return Ok(());
                }
                _ => return Err(CliError::Message("expected a comma".to_string())),
            }
        }
    }

    /// Copies one JSON string token, escapes included.
    fn string(&mut self) -> Result<String, CliError> {
        if self.peek() != Some(b'"') {
            return Err(CliError::Message("expected a string".to_string()));
        }
        let start = self.pos;
        self.pos += 1;
        while let Some(byte) = self.peek() {
            match byte {
                b'\\' => {
                    self.pos += 2;
                }
                b'"' => {
                    self.pos += 1;
                    return Ok(String::from_utf8_lossy(&self.bytes[start..self.pos]).into_owned());
                }
                _ => self.pos += 1,
            }
        }
        Err(CliError::Message("unterminated string".to_string()))
    }

    /// Copies one scalar token: a number, a bool, or null.
    fn word(&mut self) -> String {
        let start = self.pos;
        while let Some(byte) = self.peek() {
            if matches!(byte, b',' | b']' | b'}' | b' ' | b'\n' | b'\t' | b'\r') {
                break;
            }
            self.pos += 1;
        }
        let word = String::from_utf8_lossy(&self.bytes[start..self.pos]).into_owned();
        // Go's protojson writes a double whose value is integral without the
        // `.0` serde_json keeps.
        if word.contains('.')
            && !word.contains('e')
            && !word.contains('E')
            && let Ok(value) = word.parse::<f64>()
            && value.fract() == 0.0
            && value.is_finite()
        {
            return format!("{value:.0}");
        }
        word
    }
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

fn json_error(err: serde_json::Error) -> CliError {
    CliError::Message(err.to_string())
}

/// The inverse of protojson's lowerCamelCase: `uptimeSec` becomes `uptime_sec`.
fn proto_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for ch in name.chars() {
        if ch.is_ascii_uppercase() {
            out.push('_');
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_v1;
    use pretty_assertions::assert_eq;

    #[test]
    fn usage_is_the_go_text() {
        let text = usage_text();
        assert!(text.starts_with("Usage:\n  polterware [-db path] init"));
        assert!(text.ends_with("(MCP server over stdio for coding agents)\n"));
        assert_eq!(text.lines().count(), 33);
    }

    #[test]
    fn json_is_pretty_sorted_and_html_escaped_like_go() {
        let text = json_string(&serde_json::json!({"name": "a<b&c", "id": "p1"})).expect("json");
        assert_eq!(
            text,
            "{\n  \"id\": \"p1\",\n  \"name\": \"a\\u003cb\\u0026c\"\n}\n"
        );
    }

    #[test]
    fn proto_json_uses_proto_names_and_keeps_zero_values() {
        let text = proto_json_string(&local_v1::CheckResponse {
            status: "ok".to_string(),
            version: "0.1.0".to_string(),
            uptime_sec: 1.0,
            build: String::new(),
            pid: 0,
        })
        .expect("proto json");
        assert_eq!(
            text,
            "{\n  \"status\": \"ok\",\n  \"version\": \"0.1.0\",\n  \"uptime_sec\": 1,\n  \"build\": \"\",\n  \"pid\": 0\n}\n"
        );
    }

    #[test]
    fn proto_json_nests_objects_and_arrays() {
        let text = proto_json_string(&local_v1::ResolveProjectResponse {
            project: Some(local_v1::Project {
                id: "p1".to_string(),
                name: "Demo".to_string(),
                ..local_v1::Project::default()
            }),
        })
        .expect("proto json");
        assert!(
            text.starts_with("{\n  \"project\": {\n    \"id\": \"p1\","),
            "{text}"
        );
        assert!(text.contains("\"created_at\": \"\""), "{text}");
        assert!(!text.contains("createdAt"), "{text}");
    }

    /// The bytes Go's protojson writes for `polterware repo list --json`
    /// (captured from the Go binary against a project whose clone declares a
    /// GitHub remote).
    #[test]
    fn proto_json_matches_go_for_a_repository_list() {
        let message = local_v1::ListProjectReposResponse {
            repos: vec![local_v1::ProjectRepo {
                id: "01M75PVRQYZ5M96VNJ0C03WQ2R".to_string(),
                project_id: "proj-65ef9ecf".to_string(),
                name: "demo".to_string(),
                local_path: "/tmp/clone".to_string(),
                remote: "github.com/acme/demo".to_string(),
            }],
        };
        assert_eq!(
            proto_json_string(&message).expect("proto json"),
            "{\n  \"repos\": [\n    {\n      \"id\": \"01M75PVRQYZ5M96VNJ0C03WQ2R\",\n      \"project_id\": \"proj-65ef9ecf\",\n      \"name\": \"demo\",\n      \"local_path\": \"/tmp/clone\",\n      \"remote\": \"github.com/acme/demo\"\n    }\n  ]\n}\n"
        );
    }

    #[test]
    fn proto_names_are_snake_case() {
        assert_eq!(proto_name("uptimeSec"), "uptime_sec");
        assert_eq!(proto_name("agentVisible"), "agent_visible");
        assert_eq!(proto_name("id"), "id");
    }

    #[test]
    fn exit_errors_keep_their_code() {
        assert_eq!(fail(CliError::Exit(2)), 2);
    }
}
