//! Argument dispatch, usage, and exit codes.
//!
//! `polterware [-db path] <command> ...`: the top-level `-db` flag comes first
//! (parsing stops at the first positional argument), then the command reads
//! the database it was given. Only `serve` opens the store (Decision 0007);
//! every other command is a client of the core and starts it when none runs.

use super::canvas;
use super::conflict;
use super::context;
use super::db::{abs, default_db};
use super::deploys;
use super::export;
use super::feature;
use super::flags::{FlagError, FlagSet, go_quote};
use super::hook;
use super::inbox;
use super::init;
use super::mcpbridge;
use super::moved::{moved_command, run_legacy_dispatch, run_moved};
use super::output::{CliResult, fail, usage, usage_text};
use super::polter;
use super::repo;
use super::scope;
use super::serve;
use super::setup;
use super::structure;

/// The binary's entry point: parses the arguments and answers the exit code.
pub fn main() -> i32 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    run(&args)
}

/// Dispatches one command line and returns its exit code.
pub fn run(argv: &[String]) -> i32 {
    let mut flags = FlagSet::new("polterware");
    flags.set_usage(usage_text());
    flags.string_flag("db", &default_db(), "sqlite path");
    match flags.parse(argv) {
        Ok(()) => {}
        // Go's top-level flag set exits 0 on -h/--help and 2 on a bad flag,
        // after printing the usage.
        Err(FlagError::Help) => return 0,
        Err(FlagError::Invalid(_)) => return 2,
    }
    // Go's flag package stops at the first non-flag argument; everything from
    // there is the command and its arguments.
    let mut db_path = flags.string_value("db").to_string();
    if let Ok(absolute) = abs(&db_path) {
        db_path = absolute;
    }
    let args = flags.rest().to_vec();
    let Some(command) = args.first() else {
        usage();
        return 2;
    };
    let rest = &args[1..];

    let result: CliResult = match command.as_str() {
        "serve" => serve::run_serve(&db_path, rest),
        "mcp" => mcpbridge::run_mcp(&db_path),
        "init" => init::run_init(&db_path, rest),
        "repo" => repo::run_repo(&db_path, rest),
        "brief" => context::run_brief(&db_path, rest),
        "search" | "context" => context::run_search(&db_path, rest),
        "structure" => structure::run_structure(&db_path, rest),
        "canvas" => canvas::run_canvas(&db_path, rest),
        "export" => export::run_export(&db_path, rest),
        "apply" => polter::run_apply(&db_path, rest),
        "lint" => polter::run_lint(&db_path, rest),
        "format" => polter::run_format(rest),
        "query" => polter::run_query(&db_path, rest),
        "status" => polter::run_status(&db_path, rest),
        "handoff" => polter::run_handoff(&db_path, rest),
        "task" => polter::run_task(&db_path, rest),
        "feature" => feature::run_feature(&db_path, rest),
        "verify" => feature::run_verify(&db_path, rest),
        "inbox" => inbox::run_inbox(&db_path, rest),
        "conflict" => conflict::run_conflict(&db_path, rest),
        "deploys" => deploys::run_deploys(&db_path, rest),
        "scope" => scope::run_scope(&db_path, rest),
        "setup" => setup::run_setup(&db_path, rest),
        "hooks" => setup::run_hooks(&db_path, rest),
        "hook" => hook::run_hook(&db_path, rest),
        "dispatch" => run_legacy_dispatch(rest),
        "help" | "-h" | "--help" => {
            usage();
            Ok(())
        }
        _ => {
            if let Some(sub) = moved_command(command) {
                run_moved(sub, rest)
            } else {
                eprintln!("unknown command {}", go_quote(command));
                usage();
                return 2;
            }
        }
    };
    match result {
        Ok(()) => 0,
        Err(err) => fail(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn the_usage_is_ten_thousand_bytes_of_go_text() {
        assert_eq!(usage_text().len(), 3890);
        assert_eq!(usage_text().matches('\n').count(), 33);
    }

    #[test]
    fn help_commands_answer_success_and_unknown_commands_two() {
        assert_eq!(run(&["help".to_string()]), 0);
        assert_eq!(run(&["--help".to_string()]), 0);
        assert_eq!(run(&["bogus-command".to_string()]), 2);
    }

    #[test]
    fn no_command_prints_the_usage_and_answers_two() {
        assert_eq!(run(&[]), 2);
    }
}
