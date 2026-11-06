//! The local MCP helper and the MCP clients that point at it, as the Electron Desktop's
//! `mcp-supervisor.ts` keeps them: an optional `polterware mcp` this app starts and stops (its
//! stdin held open so the stdio server stays parked), the command and Cursor config a client
//! pastes, and `polterware setup mcp list|add|remove`, which registers this binary with the
//! clients this computer has. A client registers the binary itself, so setting one up needs
//! the built CLI; without it the helper runs through `go run`, as the Electron Desktop's does.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::client;

/// How long `setup mcp` may take.
const SETUP_TIMEOUT: Duration = Duration::from_secs(30);
/// How long a stopped helper has before it is killed.
const STOP_GRACE: Duration = Duration::from_secs(2);
/// The pause after spawning, so a helper that dies at once is caught.
const SETTLE: Duration = Duration::from_millis(50);

/// The helper as the MCP page shows it (`McpServerStatus`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct McpStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub db_path: String,
    pub command: String,
    pub cursor_config: String,
}

/// One MCP client and whether it points at this Polterware (`McpClientStatus`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct McpClient {
    pub id: String,
    pub name: String,
    /// `not_installed`, `not_configured`, `configured`, `configured_other`, or `unknown`.
    pub status: String,
    pub error: String,
}

impl McpClient {
    /// `CLIENT_STATE`'s label.
    pub fn label(&self) -> &'static str {
        match self.status.as_str() {
            "configured" => "Set up",
            "configured_other" => "Points elsewhere",
            "not_configured" => "Not set up",
            "not_installed" => "Not installed",
            _ => "Unknown",
        }
    }
}

/// How the helper starts: the built CLI, else `go run` from this checkout.
struct Launch {
    program: PathBuf,
    args: Vec<String>,
    cwd: Option<PathBuf>,
    display: String,
}

/// This checkout's root, where `go run ./backend/runtimes/cli` builds the CLI.
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

fn launch(db: &Path) -> Launch {
    let db_text = db.display().to_string();
    let quoted = serde_json::to_string(&db_text).unwrap_or_default();
    match client::cli_binary() {
        Some(binary) => Launch {
            display: format!("{} -db {quoted} mcp", binary.display()),
            program: binary,
            args: vec!["-db".into(), db_text, "mcp".into()],
            cwd: None,
        },
        None => Launch {
            program: "go".into(),
            args: vec![
                "run".into(),
                "./backend/runtimes/cli".into(),
                "-db".into(),
                db_text,
                "mcp".into(),
            ],
            cwd: Some(repository_root()),
            display: format!("go run ./backend/runtimes/cli -db {quoted} mcp"),
        },
    }
}

/// `cursorConfigSnippet`: the `mcpServers` entry a Cursor config takes.
fn cursor_config(db: &Path) -> String {
    let db = db.display().to_string();
    let server = match client::cli_binary() {
        Some(binary) => serde_json::json!({
            "command": binary.display().to_string(),
            "args": ["-db", db, "mcp"],
        }),
        None => serde_json::json!({
            "command": "go",
            "args": ["run", "./backend/runtimes/cli", "-db", db, "mcp"],
            "cwd": repository_root().display().to_string(),
        }),
    };
    let config = serde_json::json!({ "mcpServers": { "polterware": server } });
    format!(
        "{}\n",
        serde_json::to_string_pretty(&config).unwrap_or_default()
    )
}

/// The helper's state: `child`, when it still runs.
pub fn status(child: Option<&mut Child>, db: &Path) -> McpStatus {
    let pid = child.and_then(|child| match child.try_wait() {
        Ok(None) => Some(child.id()),
        _ => None,
    });
    McpStatus {
        running: pid.is_some(),
        pid,
        db_path: db.display().to_string(),
        command: launch(db).display,
        cursor_config: cursor_config(db),
    }
}

/// Starts the helper, failing when it exits at once.
pub fn start(db: &Path) -> Result<Child, String> {
    let launch = launch(db);
    let mut command = Command::new(&launch.program);
    command
        .args(&launch.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(cwd) = &launch.cwd {
        command.current_dir(cwd);
    }
    let mut child = command
        .spawn()
        .map_err(|error| format!("MCP helper failed to start: {error}"))?;
    std::thread::sleep(SETTLE);
    if let Ok(Some(_)) = child.try_wait() {
        return Err(
            "MCP helper exited immediately. Check the core binary and database path.".into(),
        );
    }
    Ok(child)
}

/// Stops the helper: SIGTERM, then SIGKILL after two seconds.
pub fn stop(mut child: Child) {
    if let Ok(Some(_)) = child.try_wait() {
        return;
    }
    // SAFETY: a signal to the helper this app started.
    unsafe {
        libc::kill(child.id() as i32, libc::SIGTERM);
    }
    let deadline = Instant::now() + STOP_GRACE;
    while Instant::now() < deadline {
        if let Ok(Some(_)) = child.try_wait() {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// `polterware -db <db> setup mcp <args>`'s output.
fn run_setup(db: &Path, args: &[&str]) -> Result<String, String> {
    let binary = client::cli_binary().ok_or_else(|| {
        "Build the polterware CLI first (pnpm build:cli) to set up MCP clients.".to_string()
    })?;
    let mut child = Command::new(binary)
        .arg("-db")
        .arg(db)
        .args(["setup", "mcp"])
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let deadline = Instant::now() + SETUP_TIMEOUT;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("polterware setup mcp timed out.".into());
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    let mut stdout = String::new();
    let mut stderr = String::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_string(&mut stdout);
    }
    if let Some(mut err) = child.stderr.take() {
        let _ = err.read_to_string(&mut stderr);
    }
    if !status.success() {
        let message = stderr.trim();
        return Err(if message.is_empty() {
            format!("polterware setup mcp failed ({status})")
        } else {
            message.to_string()
        });
    }
    Ok(stdout)
}

/// `setup mcp list --json`'s clients.
pub fn list_clients(db: &Path) -> Result<Vec<McpClient>, String> {
    parse_clients(&run_setup(db, &["list", "--json"])?)
}

/// Adds or removes this Polterware in client `id`, then lists them again.
pub fn set_client(db: &Path, id: &str, enabled: bool) -> Result<Vec<McpClient>, String> {
    run_setup(db, &[if enabled { "add" } else { "remove" }, id])?;
    list_clients(db)
}

fn parse_clients(output: &str) -> Result<Vec<McpClient>, String> {
    #[derive(Deserialize)]
    struct Listed {
        #[serde(default)]
        clients: Option<Vec<McpClient>>,
    }
    let listed: Listed = serde_json::from_str(output).map_err(|error| error.to_string())?;
    Ok(listed
        .clients
        .unwrap_or_default()
        .into_iter()
        .filter(|client| !client.id.is_empty())
        .map(|mut client| {
            if client.name.is_empty() {
                client.name = client.id.clone();
            }
            client
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listed_clients_keep_their_state_and_name() {
        let clients = parse_clients(
            r#"{"clients":[{"id":"cursor","name":"Cursor","status":"configured"},{"id":"zed","status":"not_installed"},{"name":"nameless"}]}"#,
        )
        .unwrap();
        assert_eq!(clients.len(), 2);
        assert_eq!(clients[0].label(), "Set up");
        assert_eq!(clients[1].name, "zed");
        assert_eq!(clients[1].label(), "Not installed");
        assert!(parse_clients(r#"{"clients":null}"#).unwrap().is_empty());
    }

    #[test]
    fn the_cursor_config_names_the_database() {
        let config = cursor_config(Path::new("/tmp/a b/local.db"));
        let parsed: serde_json::Value = serde_json::from_str(&config).unwrap();
        let args = &parsed["mcpServers"]["polterware"]["args"];
        assert!(
            args.as_array()
                .unwrap()
                .iter()
                .any(|arg| arg == "/tmp/a b/local.db")
        );
        assert!(config.ends_with("}\n"));
    }
}
