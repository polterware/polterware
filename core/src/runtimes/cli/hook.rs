//! Port of `backend/runtimes/cli/hook.go`.
use super::{
    core::{dial, resolve_project_in},
    flags::{FlagError, FlagSet},
    output::{CliError, CliResult},
};
use crate::local_v1;
use crate::platform::coreclient::CALL_TIMEOUT;
use serde::Deserialize;
use std::io::Read;
const USAGE: &str = "usage: polterware hook session-start [--label name] | session-end | lint | context | status | stop";
#[derive(Default, Deserialize)]
struct Input {
    #[serde(rename = "session_id")]
    session_id: String,
    cwd: String,
    #[serde(default)]
    tool_input: ToolInput,
}
#[derive(Default, Deserialize)]
struct ToolInput {
    file_path: String,
}
fn p(e: FlagError) -> CliError {
    CliError::message(e.message())
}
pub fn run_hook(db: &str, args: &[String]) -> CliResult {
    if args.is_empty() {
        return Err(CliError::message(USAGE));
    }
    let mut body = String::new();
    std::io::stdin().read_to_string(&mut body)?;
    let input: Input = if body.trim().is_empty() {
        Input::default()
    } else {
        serde_json::from_str(&body)
            .map_err(|e| CliError::message(format!("hook input is not JSON: {e}")))?
    };
    match args[0].as_str() {
        "session-start" => {
            let mut f = FlagSet::new("hook session-start");
            f.string_flag("label", "claude-code", "AgentRun label for the session");
            f.parse(&args[1..]).map_err(p)?;
            let path = std::env::var("CLAUDE_ENV_FILE").unwrap_or_default();
            if !path.is_empty() && !input.session_id.is_empty() {
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(path)?;
                writeln!(
                    file,
                    "export POLTERWARE_SESSION={}\nexport POLTERWARE_AGENT_LABEL={}",
                    shell_quote(&input.session_id),
                    shell_quote(f.string_value("label"))
                )?;
            }
            Ok(())
        }
        "session-end" => {
            if input.session_id.is_empty() {
                return Ok(());
            }
            let c = dial(db)?;
            let _: local_v1::EndSessionResponse = c.unary(
                "/polterware.local.v1.WorkService/EndSession",
                &local_v1::EndSessionRequest {
                    session_id: input.session_id,
                },
                CALL_TIMEOUT,
            )?;
            Ok(())
        }
        "context" => {
            if input.tool_input.file_path.is_empty() {
                return Ok(());
            }
            let c = dial(db)?;
            let dir = if input.cwd.is_empty() {
                std::env::current_dir()?.to_string_lossy().to_string()
            } else {
                input.cwd
            };
            let Some(project) = resolve_project_in(&c, "", &dir)? else {
                return Ok(());
            };
            let r: local_v1::FileContextResponse = c.unary(
                "/polterware.local.v1.WorkService/FileContext",
                &local_v1::FileContextRequest {
                    project_id: project.id,
                    session_id: input.session_id,
                    file: input.tool_input.file_path,
                },
                CALL_TIMEOUT,
            )?;
            if !r.text.trim().is_empty() {
                emit("PostToolUse", &r.text)
            }
            Ok(())
        }
        "status" | "stop" => {
            let c = dial(db)?;
            let dir = if input.cwd.is_empty() {
                std::env::current_dir()?.to_string_lossy().to_string()
            } else {
                input.cwd
            };
            if let Some(project) = resolve_project_in(&c, "", &dir)? {
                let r: local_v1::QueryPolterResponse = c.unary(
                    "/polterware.local.v1.WorkService/QueryPolter",
                    &local_v1::QueryPolterRequest {
                        project_id: project.id,
                        query: "status".into(),
                        ..Default::default()
                    },
                    CALL_TIMEOUT,
                )?;
                if args[0] == "status" && !r.text.trim().is_empty() {
                    emit("SessionStart", &r.text)
                } else if args[0] == "stop" {
                    emit(
                        "Stop",
                        "Write a handoff before ending: polterware handoff --note \"Goal: … Done: … In progress: … Next: … Touched: …\"",
                    )
                }
            }
            Ok(())
        }
        "lint" => Ok(()),
        _ => Err(CliError::message(format!(
            "unknown hook {:?}: use session-start, session-end, lint, context, status, or stop",
            args[0]
        ))),
    }
}
fn emit(event: &str, text: &str) {
    println!(
        "{{\"hookSpecificOutput\":{{\"hookEventName\":{},\"additionalContext\":{}}}}}",
        serde_json::to_string(event).unwrap_or_default(),
        serde_json::to_string(text).unwrap_or_default()
    )
}
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
