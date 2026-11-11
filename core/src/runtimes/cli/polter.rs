//! Polter text commands, through the local core.
use super::{
    core::{dial, resolve_project},
    flags::FlagSet,
    output::{CliError, CliResult},
};
use crate::{
    domains::polter,
    local_v1,
    platform::coreclient::{self, CALL_TIMEOUT},
};
use std::io::Read;

pub(super) fn flags(name: &'static str) -> FlagSet {
    let mut f = FlagSet::new(name);
    f.string_flag(
        "project",
        "",
        "project id (default: project containing the cwd)",
    );
    f
}
pub(super) fn parse(f: &mut FlagSet, args: &[String]) -> CliResult {
    f.parse_interspersed(args)
        .map_err(|e| CliError::message(e.message()))
}
pub(super) fn project(c: &coreclient::Clients, id: &str) -> Result<String, CliError> {
    Ok(resolve_project(c, id)?
        .ok_or_else(|| CliError::message("no project found"))?
        .id)
}
pub(super) fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_default()
}
pub(super) fn label() -> String {
    env("POLTERWARE_AGENT_LABEL")
}
pub(super) fn session() -> String {
    env("POLTERWARE_SESSION")
}
pub(super) fn call<T: serde::Serialize, R: serde::de::DeserializeOwned>(
    c: &coreclient::Clients,
    method: &str,
    req: &T,
) -> Result<R, coreclient::Error> {
    c.unary(
        &format!("polterware.local.v1.WorkService/{method}"),
        req,
        CALL_TIMEOUT,
    )
}
pub(super) fn diagnostics(name: &str, ds: &[local_v1::PolterDiagnostic], lines: bool) {
    for d in ds {
        let hint = if d.severity == "hint" { "hint: " } else { "" };
        if lines && d.line > 0 {
            println!("{name}:{} {hint}{}", d.line, d.message);
        } else {
            println!("{name} {hint}{}", d.message);
        }
    }
}
fn detail_diagnostics(err: &coreclient::Error) -> Option<Vec<local_v1::PolterDiagnostic>> {
    use base64::Engine;
    let coreclient::Error::Connect { details, .. } = err else {
        return None;
    };
    for detail in details {
        if detail.get("type").and_then(|v| v.as_str())
            != Some("polterware.local.v1.ApplyPolterResponse")
        {
            continue;
        }
        if let Some(debug) = detail.get("debug") {
            if let Ok(response) =
                serde_json::from_value::<local_v1::ApplyPolterResponse>(debug.clone())
            {
                if !response.diagnostics.is_empty() {
                    return Some(response.diagnostics);
                }
            }
        }
        let Some(value) = detail.get("value").and_then(|v| v.as_str()) else {
            continue;
        };
        let Ok(bytes) = base64::engine::general_purpose::STANDARD_NO_PAD
            .decode(value)
            .or_else(|_| base64::engine::general_purpose::STANDARD.decode(value))
        else {
            continue;
        };
        if let Some(fields) = wire_fields(&bytes) {
            let diagnostics: Vec<_> = fields
                .into_iter()
                .filter(|(n, _, _)| *n == 4)
                .filter_map(|(_, _, b)| {
                    let mut d = local_v1::PolterDiagnostic::default();
                    for (n, v, b) in wire_fields(b)? {
                        match n {
                            1 => d.line = v as i32,
                            2 => d.severity = String::from_utf8(b.to_vec()).ok()?,
                            3 => d.message = String::from_utf8(b.to_vec()).ok()?,
                            _ => {}
                        }
                    }
                    Some(d)
                })
                .collect();
            if !diagnostics.is_empty() {
                return Some(diagnostics);
            }
        }
    }
    None
}
fn wire_fields(mut bytes: &[u8]) -> Option<Vec<(u64, u64, &[u8])>> {
    fn varint(bytes: &mut &[u8]) -> Option<u64> {
        let mut n = 0u64;
        for shift in (0..70).step_by(7) {
            let (&b, rest) = bytes.split_first()?;
            *bytes = rest;
            if shift == 63 && b > 1 {
                return None;
            }
            n |= ((b & 127) as u64) << shift;
            if b & 128 == 0 {
                return Some(n);
            }
        }
        None
    }
    let mut fields: Vec<(u64, u64, &[u8])> = Vec::new();
    while !bytes.is_empty() {
        let tag = varint(&mut bytes)?;
        match tag & 7 {
            0 => {
                let v = varint(&mut bytes)?;
                fields.push((tag >> 3, v, &[]));
            }
            2 => {
                let n = usize::try_from(varint(&mut bytes)?).ok()?;
                if n > bytes.len() {
                    return None;
                }
                let (data, rest) = bytes.split_at(n);
                bytes = rest;
                fields.push((tag >> 3, 0, data));
            }
            _ => return None,
        }
    }
    Some(fields)
}
pub(super) fn apply(
    c: &coreclient::Clients,
    req: &local_v1::ApplyPolterRequest,
    name: &str,
    lines: bool,
) -> Result<local_v1::ApplyPolterResponse, CliError> {
    match call(c, "ApplyPolter", req) {
        Ok(r) => Ok(r),
        Err(e) => {
            if let Some(ds) = detail_diagnostics(&e) {
                diagnostics(name, &ds, lines);
                return Err(CliError::Exit(1));
            }
            Err(e.into())
        }
    }
}
pub(super) fn query(
    c: &coreclient::Clients,
    project_id: String,
    query: String,
    scope_nodes: Vec<String>,
    scope_exact: bool,
) -> CliResult {
    let r: local_v1::QueryPolterResponse = call(
        c,
        "QueryPolter",
        &local_v1::QueryPolterRequest {
            project_id,
            query,
            scope_nodes,
            scope_exact,
            ..Default::default()
        },
    )?;
    print!("{}", r.text);
    Ok(())
}
fn input(path: &str) -> Result<(String, String), CliError> {
    if path == "-" {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        Ok(("<stdin>".into(), s))
    } else {
        Ok((path.into(), std::fs::read_to_string(path)?))
    }
}
pub fn run_apply(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("apply");
    f.bool_flag("dry-run", false, "print the plan without writing");
    for (n, u) in [
        ("summary", "change summary (overrides frontmatter)"),
        ("rationale", "change rationale (overrides frontmatter)"),
        ("agent-run-id", "optional open AgentRun id"),
    ] {
        f.string_flag(n, "", u);
    }
    parse(&mut f, args)?;
    if f.rest().len() != 1 {
        return Err(CliError::message(
            "usage: polterware apply <file|-> [--dry-run]",
        ));
    }
    let (name, text) = input(&f.rest()[0])?;
    let c = dial(db)?;
    let id = project(&c, f.string_value("project"))?;
    let run = f.string_value("agent-run-id");
    let r = apply(
        &c,
        &local_v1::ApplyPolterRequest {
            project_id: id,
            text,
            dry_run: f.bool_value("dry-run"),
            change: Some(local_v1::ChangeEnvelope {
                summary: f.string_value("summary").into(),
                rationale: f.string_value("rationale").into(),
                agent_run_id: run.into(),
                ..Default::default()
            }),
            session_label: label(),
            session_id: if run.is_empty() {
                session()
            } else {
                String::new()
            },
        },
        &name,
        true,
    )?;
    diagnostics(&name, &r.diagnostics, true);
    if r.ops.is_empty() {
        println!("no changes")
    } else if f.bool_value("dry-run") {
        println!("{}", r.ops.join("\n"))
    } else {
        println!("applied {}", r.summary)
    }
    Ok(())
}
pub fn run_format(args: &[String]) -> CliResult {
    if args.len() > 1 {
        return Err(CliError::message(format!(
            "usage: polterware format [{}]",
            polter::SPEC_TOPIC_NAMES.join("|")
        )));
    }
    print!(
        "{}",
        polter::format_spec(args.first().map_or("", String::as_str)).map_err(CliError::message)?
    );
    Ok(())
}
fn scope(f: &mut FlagSet) {
    f.list_flag("scope", "Structure nodes", true);
    f.bool_flag("exact", false, "match exact scope");
}
pub fn run_query(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("query");
    scope(&mut f);
    parse(&mut f, args)?;
    if f.rest().is_empty() {
        return Err(CliError::message(
            "usage: polterware query \"cards\" | \"card <slug>\" | \"status\" | \"inbox\" | \"handoff <slug>\" | \"brief <target>\" | \"search <terms>\" | \"show <id>\" | \"commits <slug>\" | \"structure\" | \"decisions [state]\" | \"decision <decision>\" | \"feature <slug> [step]\" | \"verify <slug>\" | \"canvas [<name|id>]\"",
        ));
    }
    let c = dial(db)?;
    query(
        &c,
        project(&c, f.string_value("project"))?,
        f.rest().join(" "),
        f.list_value("scope").to_vec(),
        f.bool_value("exact"),
    )
}
pub fn run_status(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("status");
    scope(&mut f);
    parse(&mut f, args)?;
    let c = dial(db)?;
    let r: local_v1::QueryPolterResponse = call(
        &c,
        "QueryPolter",
        &local_v1::QueryPolterRequest {
            project_id: project(&c, f.string_value("project"))?,
            query: "status".into(),
            scope_nodes: f.list_value("scope").to_vec(),
            scope_exact: f.bool_value("exact"),
            ..Default::default()
        },
    )?;
    let mut text = r.text;
    let dir = std::path::Path::new(db)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    for notice in crate::providers::agentsetup::stale_notices(&dir.to_string_lossy()) {
        text = format!("{}\n{}\n", text.trim_end_matches('\n'), notice);
    }
    print!("{text}");
    Ok(())
}
pub fn run_handoff(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("handoff");
    f.string_flag(
        "card",
        "",
        "work card slug (default: the card for this branch, or the only card)",
    );
    f.string_flag(
        "note",
        "",
        "session note (Goal/Done/In progress/Next/Touched/Open question, or free text)",
    );
    parse(&mut f, args)?;
    let c = dial(db)?;
    let l = label();
    let s = session();
    let req = local_v1::WriteHandoffRequest {
        project_id: project(&c, f.string_value("project"))?,
        card_slug: f.string_value("card").into(),
        note: f.string_value("note").into(),
        agent: if l.is_empty() {
            "cli".into()
        } else {
            l.clone()
        },
        session_label: if s.is_empty() { String::new() } else { l },
        session_id: s,
        ..Default::default()
    };
    let r: local_v1::WriteHandoffResponse = match call(&c, "WriteHandoff", &req) {
        Ok(r) => r,
        Err(e) => {
            if let Some(ds) = detail_diagnostics(&e) {
                diagnostics("handoff", &ds, true);
                return Err(CliError::Exit(1));
            }
            return Err(e.into());
        }
    };
    diagnostics("handoff", &r.diagnostics, true);
    if r.ops.is_empty() {
        println!("no changes")
    } else {
        println!("applied {}", r.summary)
    }
    Ok(())
}
pub fn run_lint(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("lint");
    parse(&mut f, args)?;
    if f.rest().len() != 1 {
        return Err(CliError::message("usage: polterware lint <file|dir|->"));
    }
    let mut files = Vec::new();
    fn walk(p: &std::path::Path, out: &mut Vec<String>) -> std::io::Result<()> {
        if p.is_dir() {
            let mut entries = std::fs::read_dir(p)?.collect::<Result<Vec<_>, _>>()?;
            entries.sort_by_key(|e| e.file_name());
            for e in entries {
                if e.file_name() != ".git" && e.file_name() != "node_modules" {
                    walk(&e.path(), out)?;
                }
            }
        } else if p.to_string_lossy().ends_with(".pw.md") {
            out.push(p.to_string_lossy().into_owned());
        }
        Ok(())
    }
    let path = &f.rest()[0];
    if path != "-" && std::path::Path::new(path).is_dir() {
        walk(std::path::Path::new(path), &mut files)?
    } else {
        files.push(path.clone())
    }
    let c = dial(db)?;
    let id = match project(&c, f.string_value("project")) {
        Ok(id) => id,
        Err(e) if !f.string_value("project").is_empty() => return Err(e),
        Err(_) => String::new(),
    };
    let mut failed = false;
    for file in files {
        let (name, text) = input(&file)?;
        let r: local_v1::LintPolterResponse = call(
            &c,
            "LintPolter",
            &local_v1::LintPolterRequest {
                project_id: id.clone(),
                text,
            },
        )?;
        diagnostics(&name, &r.diagnostics, true);
        failed |= r.diagnostics.iter().any(|d| d.severity == "error");
    }
    if failed {
        Err(CliError::Exit(1))
    } else {
        Ok(())
    }
}
pub fn run_task(db: &str, args: &[String]) -> CliResult {
    const U: &str = "usage: polterware task claim|done|release <slug>#T2 [--agent name] [--commit <sha>] [--project <id>]";
    if args.is_empty() || !["claim", "done", "release"].contains(&args[0].as_str()) {
        return Err(CliError::message(U));
    }
    let action = &args[0];
    let mut f = flags("task");
    f.string_flag(
        "agent",
        "",
        "agent label holding the claim (default: $POLTERWARE_AGENT_LABEL)",
    );
    f.string_flag(
        "commit",
        "",
        "with done: the commit that finished the task, shown on the card",
    );
    parse(&mut f, &args[1..])?;
    if f.rest().is_empty() || f.rest().len() > 2 {
        return Err(CliError::message(U));
    }
    let rf = polter::parse_task_ref(&f.rest().join(" ")).map_err(CliError::message)?;
    let holder = if f.string_value("agent").trim().is_empty() {
        label()
    } else {
        f.string_value("agent").trim().into()
    };
    if holder.trim().is_empty() {
        return Err(CliError::message(format!(
            "task {action} needs an agent: pass --agent or set POLTERWARE_AGENT_LABEL"
        )));
    }
    let text = polter::coord_text(action, &rf, holder.trim()).map_err(CliError::message)?;
    let c = dial(db)?;
    let id = project(&c, f.string_value("project"))?;
    let r = apply(
        &c,
        &local_v1::ApplyPolterRequest {
            project_id: id.clone(),
            text,
            session_label: holder,
            session_id: session(),
            ..Default::default()
        },
        "task",
        false,
    )?;
    if r.ops.is_empty() {
        println!("no changes")
    } else {
        println!("{}", r.ops.join("\n"))
    }
    if action == "done" && !f.string_value("commit").trim().is_empty() {
        let r: local_v1::LinkTaskCommitResponse = call(
            &c,
            "LinkTaskCommit",
            &local_v1::LinkTaskCommitRequest {
                project_id: id,
                task: format!("{}#T{}", rf.slug, rf.number),
                sha: f.string_value("commit").trim().into(),
                ..Default::default()
            },
        )?;
        if let Some(commit) = r.commit {
            println!(
                "commit {} linked to {} T{}",
                commit.short_sha, rf.slug, rf.number
            )
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn format_matches_go() {
        assert_eq!(
            polter::format_spec("").unwrap(),
            include_str!("testdata/format-go.txt")
        );
    }
    #[test]
    fn scope_flags_repeat_and_split_commas() {
        let mut f = flags("query");
        scope(&mut f);
        parse(
            &mut f,
            &[
                "--scope".into(),
                "auth, billing".into(),
                "--scope".into(),
                "login".into(),
                "--exact".into(),
            ],
        )
        .unwrap();
        assert_eq!(f.list_value("scope"), &["auth", "billing", "login"]);
        assert!(f.bool_value("exact"));
    }
    #[test]
    fn connect_binary_diagnostics_are_preserved() {
        use base64::Engine;
        let bytes = [0x22, 0x08, 0x08, 0x07, 0x12, 0x01, b'e', 0x1a, 0x01, b'x'];
        let error = coreclient::Error::Connect {
            message: "invalid".into(),
            details: vec![
                serde_json::json!({"type":"polterware.local.v1.ApplyPolterResponse", "value":base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes)}),
            ],
        };
        let fields = detail_diagnostics(&error).unwrap();
        assert_eq!(fields[0].line, 7);
        assert_eq!(fields[0].severity, "e");
        assert_eq!(fields[0].message, "x");
    }
}
