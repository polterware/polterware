//! Port of `backend/compositions/wedge/export.go`.
//!
//! Writes Decision files into `docs/decision/` of each governing checkout and
//! the generated AGENTS.md block of every checkout. The store is the source of
//! truth: files are replaced, never read back into SQLite, and a file under
//! `docs/decision/` that Polterware did not write is never overwritten.

use std::collections::HashMap;
use std::path::Path;

use crate::domains::{decision, projection, structure, work};
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

/// Writes Decision files into `docs/decision/` of each governing checkout and
/// the generated AGENTS.md block of every checkout.
pub fn export_projections(
    store: &sqlite::Store,
    project_id: &str,
    dry_run: bool,
) -> ExportResult<Vec<ProjectionFile>> {
    let input = load_projections(store, project_id)?;
    let governed = projection::govern(&input.nodes, &input.decisions, &input.checkout_names());
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
        let decisions = governed.get(&checkout.name).map_or(&[][..], Vec::as_slice);
        out.extend(export_decisions(checkout, decisions, dry_run)?);
        out.extend(remove_legacy_projections(checkout, dry_run)?);
        out.push(export_agents_block(
            checkout,
            &input.block(&checkout.name),
            dry_run,
        )?);
    }
    Ok(out)
}

/// Rewrites only the generated AGENTS.md block of each checkout, and only
/// where it changed; Decision files stay on export. A checkout whose directory
/// is missing is skipped.
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
    /// Every Decision, each projected as a file so history stays readable;
    /// `binding` leaves out superseded ones, which agents do not follow.
    decisions: Vec<projection::Decision>,
    binding: Vec<projection::Decision>,
    governed: HashMap<String, Vec<projection::Decision>>,
    rules: Vec<work::Rule>,
}

impl Projections {
    fn block(&self, checkout: &str) -> String {
        projection::block(
            checkout,
            &self.nodes,
            self.governed.get(checkout).map_or(&[][..], Vec::as_slice),
            &self.binding,
            &self.rules,
        )
    }

    fn checkout_names(&self) -> Vec<String> {
        self.checkouts.iter().map(|co| co.name.clone()).collect()
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
    let contexts: HashMap<String, Vec<String>> = snapshot
        .decisions
        .iter()
        .map(|item| (item.id.clone(), item.scopes.clone()))
        .collect();
    let items = store.list_knowledge(project_id, "decision")?;
    let repos = project_checkouts(store, project_id).map_err(store_error)?;
    let mut input = Projections {
        checkouts: Vec::new(),
        nodes: with_default_repo(&snapshot.nodes, &repos),
        decisions: Vec::new(),
        binding: Vec::new(),
        governed: HashMap::new(),
        rules: snapshot.rules.clone(),
    };
    for item in items {
        let scopes = contexts.get(&item.id).cloned().unwrap_or_default();
        let record = projection::Decision {
            id: item.id.clone(),
            title: item.title.clone(),
            body: item.body.clone(),
            status: decision::normalize(&item.status).to_string(),
            principle: scopes.is_empty(),
            contexts: scopes,
        };
        if record.status != decision::STATE_SUPERSEDED {
            input.binding.push(record.clone());
        }
        input.decisions.push(record);
    }
    for repo in &repos {
        input.checkouts.push(Checkout {
            name: repo.name.clone(),
            dir: repo.root.clone(),
        });
    }
    input.governed = projection::govern(&input.nodes, &input.binding, &input.checkout_names());
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

fn export_decisions(
    checkout: &Checkout,
    decisions: &[projection::Decision],
    dry_run: bool,
) -> ExportResult<Vec<ProjectionFile>> {
    let dir = Path::new(&checkout.dir).join(projection::DECISION_DIR);
    // Existing files: ours by Decision id, hand-written ones by number.
    let mut ours: HashMap<String, String> = HashMap::new();
    let mut hand_written: HashMap<String, String> = HashMap::new();
    for entry in read_dir_sorted(&dir)? {
        let name = entry.0;
        let number = match projection::decision_file_number(&name) {
            Some(number) if !entry.1 => number,
            _ => continue,
        };
        let body = std::fs::read_to_string(dir.join(&name))?;
        if let Some(id) = projection::decision_marker(&body) {
            ours.insert(id, name);
            continue;
        }
        hand_written.insert(number, name);
    }

    let rel = |name: &str| format!("{}/{}", projection::DECISION_DIR, name);
    let mut out = Vec::new();
    let mut exported: Vec<String> = Vec::new();
    for record in decisions {
        let Some((name, content)) = projection::decision_file(record) else {
            out.push(ProjectionFile {
                repo: checkout.name.clone(),
                path: projection::DECISION_DIR.to_string(),
                action: PROJECTION_SKIP.to_string(),
                reason: format!(
                    "{} has no Decision number in its title",
                    crate::domains::polter::go_quote(&record.title)
                ),
            });
            continue;
        };
        let number = projection::decision_file_number(&name).unwrap_or_default();
        if let Some(existing) = hand_written.get(&number) {
            out.push(ProjectionFile {
                repo: checkout.name.clone(),
                path: rel(existing),
                action: PROJECTION_SKIP.to_string(),
                reason: "not written by polterware export; left as is".to_string(),
            });
            continue;
        }
        exported.push(record.id.clone());
        if let Some(previous) = ours.get(&record.id)
            && previous != &name
        {
            if !dry_run {
                std::fs::remove_file(dir.join(previous))?;
            }
            out.push(ProjectionFile {
                repo: checkout.name.clone(),
                path: rel(previous),
                action: PROJECTION_REMOVE.to_string(),
                reason: format!("renamed to {name}"),
            });
        }
        out.push(write_projection(
            &checkout.name,
            &dir.join(&name),
            &rel(&name),
            &content,
            dry_run,
        )?);
    }

    // A projection whose Decision no longer governs this checkout is removed.
    let mut stale: Vec<String> = ours
        .keys()
        .filter(|id| !exported.contains(*id))
        .cloned()
        .collect();
    stale.sort();
    for id in stale {
        let name = ours[&id].clone();
        if !dry_run {
            std::fs::remove_file(dir.join(&name))?;
        }
        out.push(ProjectionFile {
            repo: checkout.name.clone(),
            path: rel(&name),
            action: PROJECTION_REMOVE.to_string(),
            reason: "no longer governs this checkout".to_string(),
        });
    }
    Ok(out)
}

/// Deletes the files export wrote under the legacy directory before projections
/// moved to `docs/decision/`, and the directory when nothing else is left in
/// it. Hand-written files there stay.
fn remove_legacy_projections(
    checkout: &Checkout,
    dry_run: bool,
) -> ExportResult<Vec<ProjectionFile>> {
    let dir = Path::new(&checkout.dir).join(projection::LEGACY_DECISION_DIR);
    let entries = read_dir_sorted(&dir)?;
    let mut out = Vec::new();
    let mut kept = 0;
    for (name, is_dir) in entries {
        if is_dir {
            kept += 1;
            continue;
        }
        let body = std::fs::read_to_string(dir.join(&name))?;
        if !projection::legacy_decision_marker(&body) {
            kept += 1;
            continue;
        }
        if !dry_run {
            std::fs::remove_file(dir.join(&name))?;
        }
        out.push(ProjectionFile {
            repo: checkout.name.clone(),
            path: format!("{}/{}", projection::LEGACY_DECISION_DIR, name),
            action: PROJECTION_REMOVE.to_string(),
            reason: format!("moved to {}", projection::DECISION_DIR),
        });
    }
    if kept == 0 && !dry_run {
        match std::fs::remove_dir(&dir) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(ExportError::Io(err)),
        }
    }
    Ok(out)
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

/// Reads a directory's entries sorted by name, the way Go's `os.ReadDir` does.
/// A missing directory is empty.
fn read_dir_sorted(dir: &Path) -> ExportResult<Vec<(String, bool)>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(ExportError::Io(err)),
    };
    let mut out = Vec::new();
    for entry in entries {
        let entry = entry?;
        let is_dir = entry.file_type()?.is_dir();
        out.push((entry.file_name().to_string_lossy().into_owned(), is_dir));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::knowledge;
    use crate::providers::sqlite::test_support::TempStore;

    use super::super::work::{ApplyInput, apply_polter};

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

    const EXPORT_GRAPH: &str = r#"---
polter: 1
summary: Shop graph
rationale: Export test
---
# shop: Shop
level: domain

! Money is integer cents

# shop/web: Web
level: component · under: shop · dir: {root} · decisions: Decision 0001

# shop/auth: Auth
level: component · repo: api · path: src/auth · decisions: Decision 0002

! Tokens expire in 15 minutes
"#;

    fn has_projection(files: &[ProjectionFile], repo: &str, path: &str, action: &str) -> bool {
        files
            .iter()
            .any(|file| file.repo == repo && file.path == path && file.action == action)
    }

    fn write_file(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(path, content).expect("write");
    }

    fn read_file(path: &Path) -> String {
        std::fs::read_to_string(path).expect("read")
    }

    fn exists(path: &Path) -> bool {
        path.exists()
    }

    #[test]
    fn export_writes_decisions_and_the_agents_block() {
        let holder = TempStore::new();
        let root_dir = tempfile::tempdir().expect("root");
        let api_dir = tempfile::tempdir().expect("api");
        let root = root_dir.path().join("shop");
        let api = api_dir.path().join("api");
        for dir in [&root, &api] {
            std::fs::create_dir_all(dir).expect("checkout");
        }
        let project = init_with_repo(&holder.store, "Shop", &root.to_string_lossy());

        ensure_repository(&holder.store, &project.id, &api.to_string_lossy())
            .expect("second repository");
        let now = time::OffsetDateTime::now_utc();
        for item in [
            knowledge::Item {
                id: "decision-cents".to_string(),
                title: "Decision 0001: Integer cents".to_string(),
                body: "## Decision\n\nStore cents.".to_string(),
                status: "accepted".to_string(),
                ..knowledge::Item::default()
            },
            knowledge::Item {
                id: "decision-tokens".to_string(),
                title: "Decision 0002: Short tokens".to_string(),
                body: "## Decision\n\nShort tokens.".to_string(),
                ..knowledge::Item::default()
            },
            knowledge::Item {
                id: "decision-hand".to_string(),
                title: "Decision 0003: Hand written".to_string(),
                body: "From the store.".to_string(),
                ..knowledge::Item::default()
            },
        ] {
            let mut item = item;
            item.project_id = project.id.clone();
            item.kind = "decision".to_string();
            item.created_at = now;
            item.updated_at = now;
            holder.store.save_knowledge(item).expect("knowledge");
        }
        let text = EXPORT_GRAPH.replace("{root}", &root.to_string_lossy());
        apply_polter(
            &holder.store,
            ApplyInput {
                project_id: project.id.clone(),
                text,
                auto_start_label: "cli".to_string(),
                ..ApplyInput::default()
            },
        )
        .expect("apply");

        // A hand-written Decision with the same number and surrounding
        // AGENTS.md text stay.
        let hand_decision = root
            .join("docs")
            .join("decision")
            .join("0003-hand-written.md");
        write_file(
            &hand_decision,
            "# Decision 0003: Hand written\n\nOriginal.\n",
        );
        let agents = root.join("AGENTS.md");
        write_file(
            &agents,
            &format!(
                "# Agents\n\n{}\nstale\n{}\n\n## Tail\nKeep me.\n",
                projection::BLOCK_BEGIN,
                projection::BLOCK_END
            ),
        );

        // Projections export wrote under the legacy directory are removed;
        // hand-written files there stay, and an emptied directory goes.
        let legacy_root = root.join(projection::LEGACY_DECISION_DIR);
        let legacy_api = api.join(projection::LEGACY_DECISION_DIR);
        write_file(
            &legacy_root.join("0001-integer-cents.md"),
            "<!-- polterware:verdict decision-cents generated by polterware export -->\n# Old\n",
        );
        write_file(&legacy_root.join("notes.md"), "# Mine\n");
        write_file(
            &legacy_api.join("0002-short-tokens.md"),
            "<!-- polterware:verdict decision-tokens generated by polterware export -->\n",
        );

        let dry = export_projections(&holder.store, &project.id, true).expect("dry run");
        assert!(
            !dry.is_empty()
                && !exists(&api.join("AGENTS.md"))
                && exists(&legacy_api.join("0002-short-tokens.md")),
            "dry run wrote files: {dry:?}"
        );

        let files = export_projections(&holder.store, &project.id, false).expect("export");
        assert_eq!(
            read_file(&hand_decision),
            "# Decision 0003: Hand written\n\nOriginal.\n",
            "hand-written Decision changed"
        );
        assert!(
            has_projection(
                &files,
                "shop",
                "docs/decision/0003-hand-written.md",
                PROJECTION_SKIP
            ),
            "hand-written Decision should be skipped: {files:?}"
        );
        assert!(
            read_file(
                &root
                    .join("docs")
                    .join("decision")
                    .join("0001-integer-cents.md")
            )
            .contains("Store cents."),
            "root Decision missing"
        );
        assert!(
            read_file(
                &api.join("docs")
                    .join("decision")
                    .join("0002-short-tokens.md")
            )
            .contains("Short tokens."),
            "api Decision missing"
        );
        assert!(
            !exists(
                &root
                    .join("docs")
                    .join("decision")
                    .join("0002-short-tokens.md")
            ),
            "Decision 0002 governs api only"
        );
        assert!(
            !exists(&legacy_root.join("0001-integer-cents.md"))
                && exists(&legacy_root.join("notes.md"))
                && !exists(&legacy_api),
            "legacy projections: {files:?}"
        );
        assert!(
            has_projection(
                &files,
                "api",
                &format!("{}/0002-short-tokens.md", projection::LEGACY_DECISION_DIR),
                PROJECTION_REMOVE
            ),
            "legacy removal not reported: {files:?}"
        );

        let root_agents = read_file(&agents);
        assert!(
            !root_agents.contains("stale")
                && root_agents.starts_with(&format!("# Agents\n\n{}", projection::BLOCK_BEGIN))
                && root_agents
                    .ends_with(&format!("{}\n\n## Tail\nKeep me.\n", projection::BLOCK_END)),
            "root AGENTS.md:\n{root_agents}"
        );
        let api_agents = read_file(&api.join("AGENTS.md"));
        assert!(
            api_agents.contains("Tokens expire in 15 minutes (from shop/auth)")
                && api_agents.contains("Money is integer cents (from shop)"),
            "api AGENTS.md:\n{api_agents}"
        );

        // Edits in the checkout do not flow back; re-export replaces them.
        write_file(
            &agents,
            &root_agents.replacen("Money is integer cents", "Money is floats", 1),
        );
        let projected = root
            .join("docs")
            .join("decision")
            .join("0001-integer-cents.md");
        write_file(
            &projected,
            &format!("{}\nLocal edit.\n", read_file(&projected)),
        );
        export_projections(&holder.store, &project.id, false).expect("re-export");
        assert_eq!(
            read_file(&agents),
            root_agents,
            "re-export must replace checkout edits"
        );
        assert!(
            !read_file(&projected).contains("Local edit."),
            "re-export must replace checkout edits"
        );
        let again = export_projections(&holder.store, &project.id, false).expect("export");
        for file in &again {
            assert_ne!(
                file.action, PROJECTION_WRITE,
                "second export should change nothing: {file:?}"
            );
        }
    }

    #[test]
    fn an_unknown_project_names_itself() {
        let holder = TempStore::new();
        let err = export_projections(&holder.store, "proj-missing", true).expect_err("unknown");
        assert_eq!(err.to_string(), "unknown project \"proj-missing\"");
    }
}
