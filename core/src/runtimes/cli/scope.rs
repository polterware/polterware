//! Entity scope commands and shared scope flags.
use super::{
    core::{dial, resolve_project},
    flags::FlagSet,
    output::{CliError, CliResult},
};
use crate::{local_v1, platform::coreclient::CALL_TIMEOUT};
const USAGE: &str = "usage: polterware scope <decision|fact|rule|item> <id> [<node path>...] [--project <id>]\n       (the nodes replace its contexts, separated by spaces or commas; none clears\n       them (the whole project); item is a derived Inbox item key)";
pub(crate) fn add_scope_flags(f: &mut FlagSet) {
    f.list_flag(
        "scope",
        "Structure node path, slug, or id; repeat or separate with commas (default: whole project)",
        true,
    );
    f.bool_flag(
        "exact",
        false,
        "with --scope, leave out each node's ancestors",
    );
}
pub(crate) fn project_flags(name: &'static str) -> FlagSet {
    let mut f = FlagSet::new(name);
    f.string_flag(
        "project",
        "",
        "project id (default: project containing the cwd)",
    );
    f
}
pub(crate) fn query(
    db: &str,
    project: &str,
    query: String,
    nodes: Vec<String>,
    exact: bool,
) -> CliResult {
    let c = dial(db)?;
    let p = resolve_project(&c, project)?.ok_or_else(|| CliError::message("no project found"))?;
    let r: local_v1::QueryPolterResponse = c.unary(
        "/polterware.local.v1.WorkService/QueryPolter",
        &local_v1::QueryPolterRequest {
            project_id: p.id,
            query,
            scope_nodes: nodes,
            scope_exact: exact,
            ..Default::default()
        },
        CALL_TIMEOUT,
    )?;
    print!("{}", r.text);
    Ok(())
}
pub fn run_scope(db: &str, args: &[String]) -> CliResult {
    let mut f = project_flags("scope");
    f.parse_interspersed(args)
        .map_err(|e| CliError::message(e.message()))?;
    let p = f.rest();
    if p.len() < 2 {
        return Err(CliError::message(USAGE));
    }
    let nodes: Vec<String> = p[2..]
        .iter()
        .flat_map(|s| s.split(','))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    let c = dial(db)?;
    let project = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let _: local_v1::SetScopeResponse = c.unary(
        "/polterware.local.v1.WorkService/SetScope",
        &local_v1::SetScopeRequest {
            project_id: project.id,
            kind: p[0].clone(),
            id: p[1].clone(),
            nodes: nodes.clone(),
        },
        CALL_TIMEOUT,
    )?;
    if nodes.is_empty() {
        println!("{} {}: scope cleared (whole project)", p[0], p[1]);
    } else {
        println!("{} {}: scope {}", p[0], p[1], nodes.join(", "));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scope_flag_repeat_and_comma() {
        let mut f = FlagSet::new("t");
        add_scope_flags(&mut f);
        f.parse(&["--scope", "auth, billing", "--scope", "login", "--exact"].map(String::from))
            .unwrap();
        assert_eq!(f.list_value("scope").join("|"), "auth|billing|login");
        assert!(f.bool_value("exact"));
    }
}
