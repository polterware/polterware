//! Decision lifecycle commands.
use super::{
    core::dial,
    output::{CliError, CliResult},
    polter::{apply, flags, label, parse, project, query, session},
};
use crate::{domains::polter as domain, local_v1};
const U: &str = "usage: polterware decision list [proposed|active|challenged|superseded]\n       polterware decision show <decision>\n       polterware decision propose \"<title>\" [--trigger \"...\"] [--body \"...\"]\n       polterware decision activate <decision> [--reason \"...\"]\n       polterware decision challenge <decision> --reason \"...\"\n       polterware decision supersede <decision> (--by <decision> | --title \"<new title>\") [--reason \"...\"]\n       polterware decision trigger <decision> \"<review condition>\"";
pub fn run_decision(db: &str, args: &[String]) -> CliResult {
    if args.is_empty() {
        return Err(CliError::message(U));
    }
    let action = &args[0];
    let mut f = flags("decision");
    for (n, u) in [
        ("reason", "why the decision moves"),
        ("trigger", "review trigger"),
        ("body", "Decision text, plain (no # headings)"),
        ("by", "the decision that supersedes"),
        ("title", "title of a new superseding decision"),
        ("agent", "agent label (default: $POLTERWARE_AGENT_LABEL)"),
    ] {
        f.string_flag(n, "", u);
    }
    parse(&mut f, &args[1..])?;
    let p = f.rest();
    if action == "list" {
        if p.len() > 1
            || p.first().is_some_and(|s| {
                !["proposed", "active", "challenged", "superseded"].contains(&s.as_str())
            })
        {
            return Err(CliError::message(U));
        }
        let c = dial(db)?;
        return query(
            &c,
            project(&c, f.string_value("project"))?,
            format!("decisions {}", p.join(" ")).trim().into(),
            vec![],
            false,
        );
    }
    if action == "show" {
        if p.len() != 1 {
            return Err(CliError::message(U));
        }
        let c = dial(db)?;
        return query(
            &c,
            project(&c, f.string_value("project"))?,
            format!("decision {}", p[0]),
            vec![],
            false,
        );
    }
    if p.len() != if action == "trigger" { 2 } else { 1 } {
        return Err(CliError::message(U));
    }
    let reason = f.string_value("reason");
    let fallback = |s: &str| {
        if reason.trim().is_empty() {
            s.to_string()
        } else {
            reason.to_string()
        }
    };
    let mut i = domain::DecisionInput::default();
    let (summary, rationale) = match action.as_str() {
        "propose" => {
            i.reference = "new".into();
            i.title = p[0].clone();
            i.trigger = f.string_value("trigger").into();
            i.body = f.string_value("body").into();
            (
                format!("Propose decision {}", p[0]),
                fallback("A new decision is proposed for review"),
            )
        }
        "activate" => {
            i.reference = p[0].clone();
            i.status = "active".into();
            i.reason = reason.into();
            (
                format!("Activate decision {}", p[0]),
                fallback("The decision is accepted"),
            )
        }
        "challenge" => {
            if reason.trim().is_empty() {
                return Err(CliError::message("decision challenge needs --reason"));
            }
            i.reference = p[0].clone();
            i.status = "challenged".into();
            i.reason = reason.into();
            (format!("Challenge decision {}", p[0]), reason.into())
        }
        "supersede" => {
            let by = f.string_value("by");
            let title = f.string_value("title");
            if by.is_empty() == title.is_empty() {
                return Err(CliError::message(
                    "decision supersede needs exactly one of --by <decision> or --title \"<new title>\"",
                ));
            }
            i.reference = if by.is_empty() {
                "new".into()
            } else {
                by.into()
            };
            i.title = title.into();
            i.supersedes = p[0].clone();
            i.trigger = f.string_value("trigger").into();
            i.body = f.string_value("body").into();
            i.reason = reason.into();
            (
                format!("Supersede decision {}", p[0]),
                fallback("A newer decision replaces it"),
            )
        }
        "trigger" => {
            i.reference = p[0].clone();
            i.trigger = p[1].clone();
            (
                format!("Set review trigger on {}", p[0]),
                "Record when the decision must be reviewed".into(),
            )
        }
        _ => return Err(CliError::message(U)),
    };
    let c = dial(db)?;
    let holder = if f.string_value("agent").trim().is_empty() {
        label()
    } else {
        f.string_value("agent").trim().into()
    };
    let r = apply(
        &c,
        &local_v1::ApplyPolterRequest {
            project_id: project(&c, f.string_value("project"))?,
            text: domain::compose_decision(&i, &summary, &rationale),
            session_label: holder.trim().into(),
            session_id: session(),
            ..Default::default()
        },
        "decision",
        false,
    )?;
    if r.ops.is_empty() {
        println!("no changes")
    } else {
        println!("{}", r.ops.join("\n"))
    }
    Ok(())
}
