use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use polterware_core::platform::coreclient;
use serde_json::{Value, json};

struct CliHarness {
    _dir: tempfile::TempDir,
    db: std::path::PathBuf,
    home: std::path::PathBuf,
}

impl CliHarness {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        Self {
            db: dir.path().join("data/local.db"),
            home,
            _dir: dir,
        }
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_polterware-rs"));
        command
            .arg("-db")
            .arg(&self.db)
            .args(args)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", self.home.join(".config"))
            .env("POLTERWARE_CORE_IDLE", "1m");
        command
    }

    fn run(&self, stdin: &str, args: &[&str]) -> CommandResult {
        let mut child = self
            .command(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        if !stdin.is_empty() {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(stdin.as_bytes())
                .unwrap();
        }
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let stdout_reader = thread::spawn(move || read_all(stdout));
        let stderr_reader = thread::spawn(move || read_all(stderr));
        let deadline = Instant::now() + Duration::from_secs(60);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("polterware {args:?} timed out");
            }
            thread::sleep(Duration::from_millis(10));
        };
        let stdout = stdout_reader.join().unwrap();
        let stderr = stderr_reader.join().unwrap();
        let output = if status.success() {
            stdout
        } else {
            format!("{stdout}{stderr}")
        };
        CommandResult { output, status }
    }

    fn must_run(&self, stdin: &str, args: &[&str]) -> String {
        let result = self.run(stdin, args);
        assert!(
            result.status.success(),
            "polterware {args:?}: {}\n{}",
            result.status,
            result.output
        );
        result.output
    }
}

impl Drop for CliHarness {
    fn drop(&mut self) {
        let _ = coreclient::stop(&self.db);
    }
}

struct CommandResult {
    output: String,
    status: ExitStatus,
}

fn read_all(mut reader: impl Read) -> String {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).unwrap();
    String::from_utf8_lossy(&bytes).into_owned()
}

const E2E_ROADMAP: &str = "---\npolter: 1\nsummary: Plan the local build\nrationale: The build must be usable from every agent\n---\n\n## local-build: Local build\nstage: plan\n\nShip a build that agents can use on this machine.\n\n- [ ] T1 Wire the MCP bridge\n? Ship unsigned? [blocking] (yes | no)\n";

#[test]
fn test_binary_format_needs_no_core() {
    let harness = CliHarness::new();
    let output = harness.must_run("", &["format"]);
    assert!(
        output.contains("Polter format v1"),
        "format output: {output}"
    );
    assert!(!harness.db.exists(), "format must not create the database");
}

#[test]
fn test_binary_unknown_command_fails() {
    let harness = CliHarness::new();
    let result = harness.run("", &["definitely-not-a-command"]);
    assert!(
        !result.status.success(),
        "an unknown command must exit non-zero: {}",
        result.output
    );
    assert_eq!(
        result.status.code(),
        Some(2),
        "exit = {:?}; want status 2",
        result.status
    );
}

#[test]
fn test_binary_init_apply_query_round_trip() {
    let harness = CliHarness::new();
    let root = tempfile::tempdir().unwrap();
    let root = root.path().to_str().unwrap();
    let output = harness.must_run(
        "",
        &["init", "--id", "proj-e2e", "--name", "E2E", "--path", root],
    );
    assert!(output.contains("proj-e2e"), "init output: {output}");

    let plan = harness.must_run(
        E2E_ROADMAP,
        &["apply", "-", "--project", "proj-e2e", "--dry-run"],
    );
    assert!(plan.contains("local-build"), "dry run plan: {plan}");
    let cards = harness.must_run("", &["query", "--project", "proj-e2e", "cards"]);
    assert!(
        !cards.contains("local-build"),
        "a dry run must not write: {cards}"
    );

    harness.must_run(E2E_ROADMAP, &["apply", "-", "--project", "proj-e2e"]);
    let card = harness.must_run("", &["query", "--project", "proj-e2e", "card local-build"]);
    for want in ["local-build", "T1", "Ship unsigned?"] {
        assert!(card.contains(want), "card is missing {want:?}:\n{card}");
    }
    let status = harness.must_run("", &["status", "--project", "proj-e2e"]);
    assert!(!status.is_empty(), "status printed nothing");
}

#[test]
fn test_binary_apply_rejects_invalid_polter_with_diagnostics() {
    let harness = CliHarness::new();
    let root = tempfile::tempdir().unwrap();
    harness.must_run(
        "",
        &[
            "init",
            "--id",
            "proj-bad",
            "--name",
            "Bad",
            "--path",
            root.path().to_str().unwrap(),
        ],
    );
    let bad = "---\npolter: 1\nsummary: x\nrationale: y\n---\n\n- [ ] orphan task\n";
    let result = harness.run(bad, &["apply", "-", "--project", "proj-bad"]);
    assert!(
        !result.status.success(),
        "invalid Polter must fail: {}",
        result.output
    );
    assert!(
        result.output.contains("<stdin>:7"),
        "want a diagnostic anchored at <stdin>:7, got: {}",
        result.output
    );
}

#[test]
fn test_binary_state_survives_a_core_restart() {
    let harness = CliHarness::new();
    let root = tempfile::tempdir().unwrap();
    harness.must_run(
        "",
        &[
            "init",
            "--id",
            "proj-keep",
            "--name",
            "Keep",
            "--path",
            root.path().to_str().unwrap(),
        ],
    );
    harness.must_run(E2E_ROADMAP, &["apply", "-", "--project", "proj-keep"]);
    coreclient::stop(&harness.db).unwrap();
    let card = harness.must_run("", &["query", "--project", "proj-keep", "card local-build"]);
    assert!(
        card.contains("local-build"),
        "the card was lost across a core restart:\n{card}"
    );
}

struct McpProcess {
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<String>,
}

impl McpProcess {
    fn new(harness: &CliHarness) -> Self {
        let mut child = harness
            .command(&["mcp"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, lines) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        Self {
            child,
            stdin,
            lines,
        }
    }

    fn call(&mut self, message: &str) -> Value {
        self.stdin.write_all(message.as_bytes()).unwrap();
        self.stdin.write_all(b"\n").unwrap();
        let line = self
            .lines
            .recv_timeout(Duration::from_secs(30))
            .expect("no reply to MCP request");
        serde_json::from_str(&line).unwrap_or_else(|_| panic!("not JSON-RPC: {line:?}"))
    }
}

impl Drop for McpProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn test_binary_mcp_bridge_serves_stdio() {
    let harness = CliHarness::new();
    let root = tempfile::tempdir().unwrap();
    harness.must_run(
        "",
        &[
            "init",
            "--id",
            "proj-mcp",
            "--name",
            "MCP",
            "--path",
            root.path().to_str().unwrap(),
        ],
    );
    let mut process = McpProcess::new(&harness);

    let init = process.call(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","clientInfo":{"name":"e2e-client","version":"1"}}}"#);
    assert!(
        init.get("error").is_none() && init["result"].to_string().contains("polterware-mcp"),
        "initialize = {init}"
    );
    process
        .stdin
        .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n")
        .unwrap();
    let tools = process.call(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#);
    assert!(
        tools.get("error").is_none() && tools["result"].to_string().contains("polter.apply"),
        "tools/list = {tools}"
    );
    let format = process.call(r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"polter.format","arguments":{}}}"#);
    assert!(
        format.get("error").is_none() && format["result"].to_string().contains("Polter format v1"),
        "polter.format = {format}"
    );

    let apply = json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"polter.apply","arguments":{"project_id":"proj-mcp","text":E2E_ROADMAP}}});
    let reply = process.call(&apply.to_string());
    assert!(
        reply.get("error").is_none() && !reply["result"].to_string().contains("\"isError\":true"),
        "polter.apply over MCP = {reply}"
    );
    let card = harness.must_run("", &["query", "--project", "proj-mcp", "card local-build"]);
    assert!(
        card.contains("local-build"),
        "MCP write is not visible to the CLI:\n{card}"
    );
}
