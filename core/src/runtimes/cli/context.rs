//! Brief and search queries.
use super::{
    output::{CliError, CliResult},
    scope::{add_scope_flags, project_flags, query},
};
pub fn run_brief(db: &str, args: &[String]) -> CliResult {
    run(db, args, true)
}
pub fn run_search(db: &str, args: &[String]) -> CliResult {
    run(db, args, false)
}
fn run(db: &str, args: &[String], brief: bool) -> CliResult {
    let kind = if brief { "brief" } else { "search" };
    let mut f = project_flags(kind);
    f.string_flag("budget", "0", "token budget (default 1200)");
    add_scope_flags(&mut f);
    f.parse_interspersed(args)
        .map_err(|e| CliError::message(e.message()))?;
    let budget: i64 = f.string_value("budget").parse().map_err(|_| {
        CliError::message(format!(
            "invalid value \"{}\" for flag -budget: parse error",
            f.string_value("budget")
        ))
    })?;
    if f.rest().is_empty() {
        return Err(CliError::message(if brief {
            "usage: polterware brief <card slug|Structure path|file> [--budget N] [--project <id>]"
        } else {
            "usage: polterware search \"<terms>\" [--budget N] [--scope <node>] [--project <id>]"
        }));
    }
    let mut target = f.rest().join(" ");
    if brief
        && (target.contains('/') || target.contains('.'))
        && std::path::Path::new(&target).exists()
        && let Ok(cwd) = std::env::current_dir()
    {
        target = cwd.join(&target).to_string_lossy().into_owned();
    }
    let mut q = format!("{kind} {target}");
    if budget > 0 {
        q.push_str(&format!(" budget:{budget}"));
    }
    query(
        db,
        f.string_value("project"),
        q,
        f.list_value("scope").to_vec(),
        f.bool_value("exact"),
    )
}
