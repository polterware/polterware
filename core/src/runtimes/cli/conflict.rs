//! Conflict commands.
use super::{
    core::dial,
    output::{CliError, CliResult},
    polter::{apply, call, flags, label, parse, project, query, session},
};
use crate::local_v1;
const U: &str = "usage: polterware conflict raise <slug> \"<request>\" --rule <slug> [--card <slug>] [--option \"<text>\"]... [--project <id>]\n       polterware conflict decide <slug> \"<answer>\" [--project <id>]\n       polterware conflict wait <slug> [--timeout 30m] [--project <id>]\n       polterware conflict withdraw <slug> [--project <id>]\n       polterware conflict list [--project <id>]";
pub fn run_conflict(db: &str, args: &[String]) -> CliResult {
    if args.is_empty() {
        return Err(CliError::message(U));
    }
    let verb = &args[0];
    let mut f = flags("conflict");
    for (n, u) in [
        ("rule", "the Rule the request goes against (slug or id)"),
        ("card", "the card the request is about"),
    ] {
        f.string_flag(n, "", u)
    }
    f.duration_flag("timeout", 1800_000_000_000, "how long wait blocks");
    f.list_flag("option", "an answer a person may pick (repeat it)", false);
    parse(&mut f, &args[1..])?;
    let p = f.rest();
    let n = match verb.as_str() {
        "raise" | "decide" => 2,
        "wait" | "withdraw" => 1,
        "list" => 0,
        _ => return Err(CliError::message(U)),
    };
    if p.len() != n {
        return Err(CliError::message(U));
    }
    let c = dial(db)?;
    let id = project(&c, f.string_value("project"))?;
    if verb == "list" {
        return query(&c, id, "conflicts".into(), vec![], false);
    }
    if verb == "wait" {
        let timeout = f.duration_value("timeout");
        let start = std::time::Instant::now();
        loop {
            let r: local_v1::QueryPolterResponse = call(
                &c,
                "QueryPolter",
                &local_v1::QueryPolterRequest {
                    project_id: id.clone(),
                    query: format!("conflict {}", p[0]),
                    ..Default::default()
                },
            )?;
            if !r.text.contains("\nstate: open\n") {
                print!("{}", r.text);
                return Ok(());
            }
            if start.elapsed().as_nanos() >= timeout.max(0) as u128 {
                print!("{}", r.text);
                return Err(CliError::message(format!(
                    "conflict {} is still open after {}",
                    p[0],
                    super::flags::format_go_duration(std::time::Duration::from_nanos(
                        timeout.max(0) as u64
                    ))
                )));
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }
    if verb == "decide" {
        let r: local_v1::ResolveInboxItemResponse = call(
            &c,
            "ResolveInboxItem",
            &local_v1::ResolveInboxItemRequest {
                project_id: id,
                item_id: format!("conflict:{}", p[0]),
                action: "decide".into(),
                answer: p[1].clone(),
                session_label: label(),
                ..Default::default()
            },
        )?;
        println!("{}", r.ops.join("\n"));
        return Ok(());
    }
    let mut text = String::from("---\npolter: 1\n");
    if verb == "raise" {
        if f.string_value("rule").is_empty() {
            return Err(CliError::message(
                "conflict raise needs --rule: the law the request goes against",
            ));
        }
        text.push_str(&format!("summary: Raise conflict {}\nrationale: The request goes against the project's law, so a person decides it\n---\n\n#### conflict {}: {}\n",p[0],p[0],p[1]));
        for key in ["rule", "card"] {
            let value = f.string_value(key).trim();
            if !value.is_empty() {
                text.push_str(&format!("{key}: {value}\n"))
            }
        }
        if !f.list_value("option").is_empty() {
            text.push_str(&format!(
                "options: {}\n",
                f.list_value("option").join(" | ")
            ))
        }
    } else {
        text.push_str(&format!("summary: Withdraw conflict {}\nrationale: The request no longer needs a decision\n---\n\n#### conflict {}\nstatus: withdrawn\n",p[0],p[0]))
    }
    let r = apply(
        &c,
        &local_v1::ApplyPolterRequest {
            project_id: id,
            text,
            session_label: label(),
            session_id: session(),
            ..Default::default()
        },
        "conflict",
        false,
    )?;
    println!("{}", r.ops.join("\n"));
    if verb == "raise" {
        println!(
            "raised: a person decides it in the Inbox; read the answer with polterware conflict wait {}",
            p[0]
        )
    }
    Ok(())
}
