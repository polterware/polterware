//! Port of `backend/compositions/wedge/structure_history.go`.

#[allow(unused_imports)]
use super::*;

use std::fmt::Write as _;
use std::sync::LazyLock;

use regex::Regex;

use crate::domains::structure;
use crate::providers::git;
use crate::providers::sqlite::{self, Store};

static COMMIT_SHA: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9a-fA-F]{7,40}$").expect("the commit sha pattern"));

/// Replays the Structure as it was at `point`: a change batch id, an RFC3339
/// time, a YYYY-MM-DD day, or a commit sha (resolved to its committer time).
/// Rows carry no derived status, feature counts, those are live-only.
pub fn structure_at(
    store: &Store,
    project_id: &str,
    point: &str,
) -> sqlite::Result<Vec<structure::Row>> {
    let ops = store.list_structure_change_ops(project_id)?;
    let point = resolve_commit_point(store, project_id, &ops, point)?;
    let cut = structure::cut(&ops, &point).map_err(structure_error)?;
    Ok(structure::historical(&structure::replay(
        project_id,
        &ops[..cut],
    )))
}

/// Turns a commit sha into its committer time. Anything else, including a batch
/// id, passes through unchanged.
fn resolve_commit_point(
    store: &Store,
    project_id: &str,
    ops: &[structure::HistoryOp],
    point: &str,
) -> sqlite::Result<String> {
    let point = point.trim();
    if !COMMIT_SHA.is_match(point) {
        return Ok(point.to_string());
    }
    for op in ops {
        if op.batch_id == point {
            return Ok(point.to_string());
        }
    }
    let checkouts = project_checkouts(store, project_id)?;
    for co in &checkouts {
        if let Some(committed) = git::commit_time(&co.root, point) {
            return Ok(sqlite::format_rfc3339(committed));
        }
    }
    Err(sqlite::Error::message(format!(
        "unknown point {point:?}: not a change batch, and not a commit in the project's repositories"
    )))
}

/// Lists the change batches that touched the Structure, newest first.
pub fn structure_history(
    store: &Store,
    project_id: &str,
) -> sqlite::Result<Vec<structure::HistoryEntry>> {
    let ops = store.list_structure_change_ops(project_id)?;
    let mut entries = structure::history(&ops);
    entries.reverse();
    Ok(entries)
}

/// Renders the batches for `structure history`.
pub(crate) fn encode_structure_history(entries: &[structure::HistoryEntry]) -> String {
    if entries.is_empty() {
        return "No Structure changes recorded.\n".to_string();
    }
    let mut out = String::new();
    for entry in entries {
        let _ = write!(
            out,
            "{}  {}  {} ops",
            entry.batch_id,
            sqlite::format_rfc3339(entry.at),
            entry.node_ops
        );
        if !entry.summary.is_empty() {
            let _ = write!(out, "  {}", entry.summary);
        }
        out.push('\n');
    }
    out
}

fn structure_error(error: structure::Error) -> sqlite::Error {
    sqlite::Error::message(error.to_string())
}

/// One repository cloned on this machine.
struct GitCheckout {
    #[allow(dead_code)]
    name: String,
    root: String,
}

// TODO(dedupe): private copy of `repos.go`'s `projectCheckouts` and `gitCheckout`;
// the sibling module exports the same once the port lands.
fn project_checkouts(store: &Store, project_id: &str) -> sqlite::Result<Vec<GitCheckout>> {
    let repos = store.list_repositories(project_id)?;
    let mut out = Vec::new();
    for repo in repos {
        let path = expand_path(&repo.local_path);
        if !path.is_empty() {
            out.push(GitCheckout {
                name: repo.name,
                root: path,
            });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::sqlite::test_support::{TempStore, save_project};
    use time::OffsetDateTime;

    #[test]
    fn an_empty_structure_has_no_history() {
        let TempStore { store, .. } = TempStore::new();
        save_project(&store, "p", OffsetDateTime::UNIX_EPOCH);
        let entries = structure_history(&store, "p").expect("history");
        assert!(entries.is_empty(), "entries: {entries:?}");
        assert_eq!(
            encode_structure_history(&entries),
            "No Structure changes recorded.\n"
        );
        let rows = structure_at(&store, "p", "").expect("structure at");
        assert!(rows.is_empty(), "rows: {rows:?}");
    }

    #[test]
    fn an_unknown_point_is_refused() {
        let TempStore { store, .. } = TempStore::new();
        save_project(&store, "p", OffsetDateTime::UNIX_EPOCH);
        let err = structure_at(&store, "p", "deadbeef").unwrap_err();
        assert_eq!(
            err.to_string(),
            "unknown point \"deadbeef\": not a change batch, and not a commit in the project's repositories"
        );
    }
}
