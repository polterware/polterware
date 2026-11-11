//! Project tree query.
use super::{
    flags::go_quote,
    output::{CliError, CliResult},
    scope::{project_flags, query},
};
pub fn run_structure(db: &str, args: &[String]) -> CliResult {
    let mut f = project_flags("structure");
    f.string_flag("node", "", "node id or path");
    f.string_flag(
        "at",
        "",
        "point in time: a change batch id, an RFC3339 time, a YYYY-MM-DD day, or a commit sha",
    );
    f.parse(args).map_err(|e| CliError::message(e.message()))?;
    let history = f.rest().len() == 1 && f.rest()[0] == "history";
    if !f.rest().is_empty() && !history {
        return Err(CliError::message(format!(
            "unknown argument {}: use structure [--node <path>] [--at <point>] or structure history",
            go_quote(&f.rest()[0])
        )));
    }
    if history && (!f.string_value("node").is_empty() || !f.string_value("at").is_empty()) {
        return Err(CliError::message(
            "structure history takes no --node or --at",
        ));
    }
    let mut q = "structure".to_string();
    if history {
        q.push_str(" history");
    } else {
        if !f.string_value("node").is_empty() {
            q.push(' ');
            q.push_str(f.string_value("node"));
        }
        if !f.string_value("at").is_empty() {
            q.push_str(" at ");
            q.push_str(f.string_value("at"));
        }
    }
    query(db, f.string_value("project"), q, vec![], false)
}
