//! The generated AGENTS.md block of every checkout.
//!
//! Writes the generated AGENTS.md block of every checkout. The store is the
//! source of truth: files are replaced, never read back into SQLite.

use std::path::Path;

use crate::domains::{projection, structure, work};
use crate::providers::sqlite;

#[allow(unused_imports)]
use super::*;

/// Projection file actions.
pub const PROJECTION_WRITE: &str = "write";
pub const PROJECTION_UNCHANGED: &str = "unchanged";
pub const PROJECTION_REMOVE: &str = "remove";
pub const PROJECTION_SKIP: &str = "skip";

/// The errors `export.go` returns, one per Go `fmt.Errorf` site plus the store
/// and the filesystem.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error(transparent)]
    Store(#[from] sqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
}

impl ExportError {
    fn message(text: impl Into<String>) -> ExportError {
        ExportError::Message(text.into())
    }
}

pub type ExportResult<T> = std::result::Result<T, ExportError>;

/// One file export wrote, kept, removed, or skipped. `path` is relative to the
/// checkout; `repo` is empty for the project root.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectionFile {
    pub repo: String,
    pub path: String,
    pub action: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Checkout {
    name: String,
    dir: String,
}

/// Writes the generated AGENTS.md block of every checkout.
pub fn export_projections(
    store: &sqlite::Store,
    project_id: &str,
    dry_run: bool,
) -> ExportResult<Vec<ProjectionFile>> {
    let input = load_projections(store, project_id)?;
    let mut out = Vec::new();
    for checkout in &input.checkouts {
        if !is_dir(&checkout.dir) {
            out.push(ProjectionFile {
                repo: checkout.name.clone(),
                path: ".".to_string(),
                action: PROJECTION_SKIP.to_string(),
                reason: "checkout directory not found".to_string(),
            });
            continue;
        }
        out.push(export_agents_block(
            checkout,
            &input.block(&checkout.name),
            dry_run,
        )?);
    }
    Ok(out)
}

/// Rewrites only the generated AGENTS.md block of each checkout, and only
/// where it changed. A checkout whose directory is missing is skipped.
pub fn sync_agents_blocks(
    store: &sqlite::Store,
    project_id: &str,
) -> ExportResult<Vec<ProjectionFile>> {
    let input = load_projections(store, project_id)?;
    let mut out = Vec::new();
    for checkout in &input.checkouts {
        if !is_dir(&checkout.dir) {
            out.push(ProjectionFile {
                repo: checkout.name.clone(),
                path: ".".to_string(),
                action: PROJECTION_SKIP.to_string(),
                reason: "checkout directory not found".to_string(),
            });
            continue;
        }
        out.push(export_agents_block(
            checkout,
            &input.block(&checkout.name),
            false,
        )?);
    }
    Ok(out)
}

struct Projections {
    checkouts: Vec<Checkout>,
    nodes: Vec<structure::Node>,
    rules: Vec<work::Rule>,
}

impl Projections {
    fn block(&self, checkout: &str) -> String {
        projection::block(checkout, &self.nodes, &self.rules)
    }
}

fn load_projections(store: &sqlite::Store, project_id: &str) -> ExportResult<Projections> {
    if store.get_project(project_id).is_err() {
        return Err(ExportError::message(format!(
            "unknown project {}",
            crate::domains::polter::go_quote(project_id)
        )));
    }
    let snapshot = store.load_work_snapshot(project_id)?;
    let repos = project_checkouts(store, project_id).map_err(store_error)?;
    let mut input = Projections {
        checkouts: Vec::new(),
        nodes: with_default_repo(&snapshot.nodes, &repos),
        rules: snapshot.rules.clone(),
    };
    for repo in &repos {
        input.checkouts.push(Checkout {
            name: repo.name.clone(),
            dir: repo.root.clone(),
        });
    }
    Ok(input)
}

/// Maps a repository read error onto the export error, which only carries the
/// store's own failures.
fn store_error(err: ReposError) -> ExportError {
    match err {
        ReposError::Store(err) => ExportError::Store(err),
        other => ExportError::message(other.to_string()),
    }
}

fn export_agents_block(
    checkout: &Checkout,
    block: &str,
    dry_run: bool,
) -> ExportResult<ProjectionFile> {
    let target = Path::new(&checkout.dir).join(projection::AGENTS_FILE);
    let existing = match std::fs::read_to_string(&target) {
        Ok(existing) => existing,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(ExportError::Io(err)),
    };
    let content = match projection::replace_block(&existing, block) {
        Ok(content) => content,
        Err(err) => {
            return Ok(ProjectionFile {
                repo: checkout.name.clone(),
                path: projection::AGENTS_FILE.to_string(),
                action: PROJECTION_SKIP.to_string(),
                reason: err.to_string(),
            });
        }
    };
    write_projection(
        &checkout.name,
        &target,
        projection::AGENTS_FILE,
        &content,
        dry_run,
    )
}

fn write_projection(
    repo: &str,
    target: &Path,
    rel: &str,
    content: &str,
    dry_run: bool,
) -> ExportResult<ProjectionFile> {
    let mut file = ProjectionFile {
        repo: repo.to_string(),
        path: rel.to_string(),
        action: PROJECTION_WRITE.to_string(),
        reason: String::new(),
    };
    if let Ok(current) = std::fs::read(target)
        && current == content.as_bytes()
    {
        file.action = PROJECTION_UNCHANGED.to_string();
        return Ok(file);
    }
    if dry_run {
        return Ok(file);
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(target, content.as_bytes())?;
    Ok(file)
}

fn is_dir(path: &str) -> bool {
    std::fs::metadata(path).is_ok_and(|info| info.is_dir())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::sqlite::test_support::TempStore;

    use super::super::work::apply_polter;

    /// Opens a project whose only repository is cloned at `root`.
    fn init_with_repo(
        store: &sqlite::Store,
        name: &str,
        root: &str,
    ) -> crate::domains::projects::Project {
        let project = init_project(
            store,
            InitInput {
                name: name.to_string(),
                ..InitInput::default()
            },
        )
        .expect("project");
        ensure_repository(store, &project.id, root).expect("repository");
        project
    }

    #[test]
    fn export_writes_the_agents_block_only() {
        let holder = TempStore::new();
        let store = &holder.store;
        let root = tempfile::tempdir().expect("tempdir");
        let root_path = root.path().to_string_lossy().into_owned();
        let project = init_with_repo(store, "Export", &root_path);
        let text = format!(
            "---\npolter: 1\nsummary: Shop graph\nrationale: Export test\n---\n# shop: Shop\nlevel: domain\n\n! Money is integer cents\n\n# shop/web: Web\nlevel: component · under: shop · dir: {root_path}\n"
        );
        apply_polter(
            store,
            ApplyInput {
                project_id: project.id.clone(),
                text,
                summary: "Shop graph".into(),
                rationale: "Export test".into(),
                auto_start_label: "export-test".into(),
                ..ApplyInput::default()
            },
        )
        .expect("apply");

        let files = export_projections(store, &project.id, false).expect("export");
        assert!(
            files
                .iter()
                .any(|file| file.path == "AGENTS.md" && file.action == PROJECTION_WRITE),
            "files = {files:?}"
        );
        assert!(
            files
                .iter()
                .all(|file| !file.path.starts_with("docs/decision")),
            "export must not write decision files: {files:?}"
        );
        let agents = std::fs::read_to_string(root.path().join("AGENTS.md")).expect("agents");
        assert!(agents.contains("- Money is integer cents"));
        assert!(!agents.contains("Decisions:"));
    }
}
