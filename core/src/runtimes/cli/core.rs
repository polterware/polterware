//! Port of `backend/runtimes/cli/core.go`: the CLI's client of the core.
//!
//! `dial` returns clients for the core that owns the database, starting it when
//! none is running (Decision 0007: only the core opens the store). The rest
//! resolves the project a command works in and converts the line-numbered
//! diagnostics a core attaches to a refused write.

use std::env;

use crate::domains::polter;
use crate::local_v1;
use crate::platform::coreclient::{self, CALL_TIMEOUT};

use super::output::CliError;

const RESOLVE_PROJECT: &str = "/polterware.local.v1.ProjectService/ResolveProject";

/// Returns clients for the core that owns `db_path`, starting it when needed.
pub fn dial(db_path: &str) -> Result<coreclient::Clients, CliError> {
    Ok(coreclient::connect(db_path, "cli")?)
}

/// Returns the project with `id`, or the one containing the working directory.
pub fn resolve_project(
    clients: &coreclient::Clients,
    id: &str,
) -> Result<Option<local_v1::Project>, CliError> {
    let cwd = env::current_dir()?;
    resolve_project_in(clients, id, &cwd.to_string_lossy())
}

/// Returns the project with `id`, or the one containing `dir`.
pub fn resolve_project_in(
    clients: &coreclient::Clients,
    id: &str,
    dir: &str,
) -> Result<Option<local_v1::Project>, CliError> {
    let response: local_v1::ResolveProjectResponse = clients.unary(
        RESOLVE_PROJECT,
        &local_v1::ResolveProjectRequest {
            id: id.to_string(),
            dir: dir.to_string(),
        },
        CALL_TIMEOUT,
    )?;
    Ok(response.project)
}

/// Converts the core's diagnostics into the Polter package's.
pub fn diagnostics_from(items: &[local_v1::PolterDiagnostic]) -> Vec<polter::Diagnostic> {
    items
        .iter()
        .map(|item| polter::Diagnostic {
            line: item.line,
            severity: item.severity.clone(),
            message: item.message.clone(),
        })
        .collect()
}

/// Extracts the line-numbered diagnostics the core attaches to an invalid
/// `ApplyPolter` call.
///
/// Go reads them out of the Connect error's details (an `ApplyPolterResponse`).
/// `coreclient` answers with the message alone and keeps no details, so until
/// it carries them this reads as Go does for an error without diagnostics.
pub fn polter_diagnostics(_err: &coreclient::Error) -> Option<Vec<polter::Diagnostic>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_keep_the_line_severity_and_message() {
        let items = vec![local_v1::PolterDiagnostic {
            line: 7,
            severity: polter::SEVERITY_HINT.to_string(),
            message: "rename me".to_string(),
        }];
        let diagnostics = diagnostics_from(&items);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 7);
        assert_eq!(diagnostics[0].severity, "hint");
        assert_eq!(diagnostics[0].message, "rename me");
    }
}
