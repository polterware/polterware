//! The `milestone` command: create, change, archive, and order milestones.

use super::{
    core::{dial, resolve_project},
    flags::{FlagError, FlagSet},
    output::{CliError, CliResult},
};
use crate::local_v1;
use crate::platform::coreclient::{CALL_TIMEOUT, Clients};

const USAGE: &str = "usage: polterware milestone ls [--archived] [--project <id>]\n       polterware milestone new \"<name>\" [--goal \"...\"] [--exit \"...\"]... [--target YYYY-MM-DD] [--state planned|active|done] [--project <id>]\n       polterware milestone set <slug|id> [--name \"...\"] [--goal \"...\"] [--exit \"...\"]... [--target YYYY-MM-DD] [--state planned|active|done] [--project <id>]\n       polterware milestone archive|restore <slug|id> [--project <id>]\n       polterware milestone order <slug|id>... [--project <id>]   (every active milestone, first to last)\n       (a card joins one with milestone: <slug> in polterware apply)";
const SERVICE: &str = "/polterware.local.v1.WorkService/";

fn parse_err(err: FlagError) -> CliError {
    CliError::message(err.message())
}

fn flags(name: &'static str) -> FlagSet {
    let mut f = FlagSet::new(name);
    f.string_flag(
        "project",
        "",
        "project id (default: project containing the cwd)",
    );
    f
}

/// Adds the fields `new` and `set` share.
fn field_flags(f: &mut FlagSet) {
    f.string_flag("goal", "", "what the milestone is for");
    f.list_flag("exit", "an exit criterion; repeat for each", false);
    f.string_flag("target", "", "target date, YYYY-MM-DD");
    f.string_flag("state", "", "planned, active, or done");
}

/// Reports whether `--name` (or `-name`, with or without `=value`) is among `args`.
pub(crate) fn given(args: &[String], name: &str) -> bool {
    args.iter().any(|arg| {
        let bare = arg.trim_start_matches('-');
        arg.starts_with('-') && (bare == name || bare.starts_with(&format!("{name}=")))
    })
}

fn change(summary: String) -> Option<local_v1::ChangeEnvelope> {
    Some(local_v1::ChangeEnvelope {
        summary,
        rationale: "Written from the CLI".to_string(),
        ..Default::default()
    })
}

pub fn run_milestone(db: &str, args: &[String]) -> CliResult {
    let Some(sub) = args.first() else {
        return Err(CliError::message(USAGE));
    };
    let rest = &args[1..];
    match sub.as_str() {
        "ls" | "list" => ls(db, rest),
        "new" => new(db, rest),
        "set" => set(db, rest),
        "archive" => archive(db, rest, true),
        "restore" => archive(db, rest, false),
        "order" => order(db, rest),
        _ => Err(CliError::message(USAGE)),
    }
}

fn list(c: &Clients, project: &str) -> Result<Vec<local_v1::Milestone>, CliError> {
    let r: local_v1::ListMilestonesResponse = c.unary(
        &format!("{SERVICE}ListMilestones"),
        &local_v1::ListMilestonesRequest {
            project_id: project.to_string(),
        },
        CALL_TIMEOUT,
    )?;
    Ok(r.milestones)
}

fn find(items: &[local_v1::Milestone], reference: &str) -> Result<local_v1::Milestone, CliError> {
    let needle = reference.trim();
    items
        .iter()
        .find(|m| m.id == needle || m.slug == needle)
        .cloned()
        .ok_or_else(|| {
            CliError::message(format!(
                "unknown milestone {needle:?}: list them with polterware milestone ls"
            ))
        })
}

fn format_list(items: &[local_v1::Milestone], archived: bool) -> String {
    let shown: Vec<&local_v1::Milestone> = items
        .iter()
        .filter(|m| m.archived_at.is_empty() != archived)
        .collect();
    if shown.is_empty() {
        return if archived {
            "no archived milestones\n".to_string()
        } else {
            "no milestones\n".to_string()
        };
    }
    let mut out = String::new();
    for m in shown {
        let mut line = format!("{}  {}  {:?}", m.slug, m.state, m.name);
        if !m.target_date.is_empty() {
            line.push_str(&format!("  target {}", m.target_date));
        }
        out.push_str(&line);
        out.push('\n');
        if !m.goal.is_empty() {
            out.push_str(&format!("  goal: {}\n", m.goal));
        }
        for exit in &m.exit_criteria {
            out.push_str(&format!("  exit: {exit}\n"));
        }
    }
    out
}

fn ls(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("milestone ls");
    f.bool_flag("archived", false, "list only archived milestones");
    f.parse_interspersed(args).map_err(parse_err)?;
    if !f.rest().is_empty() {
        return Err(CliError::message(USAGE));
    }
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    print!(
        "{}",
        format_list(&list(&c, &p.id)?, f.bool_value("archived"))
    );
    Ok(())
}

fn print_saved(verb: &str, m: Option<local_v1::Milestone>) -> CliResult {
    let m = m.ok_or_else(|| CliError::message("the core returned no milestone"))?;
    println!("{verb} milestone {}: {} ({})", m.slug, m.name, m.state);
    Ok(())
}

fn new(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("milestone new");
    field_flags(&mut f);
    f.parse_interspersed(args).map_err(parse_err)?;
    if f.rest().len() != 1 || f.rest()[0].trim().is_empty() {
        return Err(CliError::message(USAGE));
    }
    let name = f.rest()[0].trim().to_string();
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let r: local_v1::CreateMilestoneResponse = c.unary(
        &format!("{SERVICE}CreateMilestone"),
        &local_v1::CreateMilestoneRequest {
            project_id: p.id,
            name: name.clone(),
            goal: f.string_value("goal").to_string(),
            exit_criteria: f.list_value("exit").to_vec(),
            target_date: f.string_value("target").to_string(),
            state: f.string_value("state").to_string(),
            change: change(format!("Add milestone: {name}")),
        },
        CALL_TIMEOUT,
    )?;
    print_saved("+", r.milestone)
}

fn set(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("milestone set");
    field_flags(&mut f);
    f.string_flag("name", "", "the milestone's name");
    f.parse_interspersed(args).map_err(parse_err)?;
    if f.rest().len() != 1 {
        return Err(CliError::message(USAGE));
    }
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let mut m = find(&list(&c, &p.id)?, &f.rest()[0])?;
    // UpdateMilestone replaces every field; keep what the flags leave out.
    if given(args, "name") {
        m.name = f.string_value("name").to_string();
    }
    if given(args, "goal") {
        m.goal = f.string_value("goal").to_string();
    }
    if given(args, "exit") {
        m.exit_criteria = f
            .list_value("exit")
            .iter()
            .filter(|e| !e.trim().is_empty())
            .cloned()
            .collect();
    }
    if given(args, "target") {
        m.target_date = f.string_value("target").to_string();
    }
    if given(args, "state") {
        m.state = f.string_value("state").to_string();
    }
    let r: local_v1::UpdateMilestoneResponse = c.unary(
        &format!("{SERVICE}UpdateMilestone"),
        &local_v1::UpdateMilestoneRequest {
            project_id: p.id,
            id: m.id.clone(),
            name: m.name.clone(),
            goal: m.goal,
            exit_criteria: m.exit_criteria,
            target_date: m.target_date,
            state: m.state,
            change: change(format!("Change milestone: {}", m.name)),
        },
        CALL_TIMEOUT,
    )?;
    print_saved("~", r.milestone)
}

fn archive(db: &str, args: &[String], archived: bool) -> CliResult {
    let mut f = flags(if archived {
        "milestone archive"
    } else {
        "milestone restore"
    });
    f.parse_interspersed(args).map_err(parse_err)?;
    if f.rest().len() != 1 {
        return Err(CliError::message(USAGE));
    }
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let m = find(&list(&c, &p.id)?, &f.rest()[0])?;
    let verb = if archived { "Archive" } else { "Restore" };
    let r: local_v1::ArchiveMilestoneResponse = c.unary(
        &format!("{SERVICE}ArchiveMilestone"),
        &local_v1::ArchiveMilestoneRequest {
            project_id: p.id,
            id: m.id,
            archived,
            change: change(format!("{verb} milestone: {}", m.name)),
        },
        CALL_TIMEOUT,
    )?;
    print_saved(if archived { "archived" } else { "restored" }, r.milestone)
}

fn order(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("milestone order");
    f.parse_interspersed(args).map_err(parse_err)?;
    if f.rest().is_empty() {
        return Err(CliError::message(USAGE));
    }
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let items = list(&c, &p.id)?;
    let ids = f
        .rest()
        .iter()
        .flat_map(|s| s.split(','))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|reference| find(&items, reference).map(|m| m.id))
        .collect::<Result<Vec<_>, _>>()?;
    let r: local_v1::ReorderMilestonesResponse = c.unary(
        &format!("{SERVICE}ReorderMilestones"),
        &local_v1::ReorderMilestonesRequest {
            project_id: p.id,
            milestone_ids: ids,
            change: change("Reorder milestones".to_string()),
        },
        CALL_TIMEOUT,
    )?;
    let slugs: Vec<String> = r.milestones.into_iter().map(|m| m.slug).collect();
    println!("order: {}", slugs.join(", "));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| v.to_string()).collect()
    }

    #[test]
    fn given_sees_a_flag_in_any_spelling() {
        let a = args(&["beta", "--goal", "x", "-state=active"]);
        assert!(given(&a, "goal"));
        assert!(given(&a, "state"));
        assert!(!given(&a, "exit"));
        assert!(!given(&args(&["goal"]), "goal"));
    }

    #[test]
    fn find_by_slug_or_id() {
        let items = vec![local_v1::Milestone {
            id: "m1".into(),
            slug: "beta".into(),
            ..Default::default()
        }];
        assert_eq!(find(&items, "beta").unwrap().id, "m1");
        assert_eq!(find(&items, "m1").unwrap().slug, "beta");
        assert!(find(&items, "gamma").is_err());
    }

    #[test]
    fn list_splits_active_and_archived() {
        let items = vec![
            local_v1::Milestone {
                slug: "beta".into(),
                name: "Beta".into(),
                state: "active".into(),
                goal: "Ship it.".into(),
                exit_criteria: vec!["No P0 bugs".into()],
                target_date: "2026-12-01".into(),
                ..Default::default()
            },
            local_v1::Milestone {
                slug: "old".into(),
                name: "Old".into(),
                state: "done".into(),
                archived_at: "2026-01-01T00:00:00Z".into(),
                ..Default::default()
            },
        ];
        assert_eq!(
            format_list(&items, false),
            "beta  active  \"Beta\"  target 2026-12-01\n  goal: Ship it.\n  exit: No P0 bugs\n"
        );
        assert_eq!(format_list(&items, true), "old  done  \"Old\"\n");
        assert_eq!(format_list(&[], false), "no milestones\n");
    }
}
