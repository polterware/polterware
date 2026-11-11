//! Port of `backend/compositions/wedge/work.go`.

#[allow(unused_imports)]
use super::*;

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use serde_json::Value;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;

use crate::domains::agentrun;
use crate::domains::canvas;
use crate::domains::decision;
use crate::domains::inbox;
use crate::domains::knowledge;
use crate::domains::polter;
use crate::domains::projects;
use crate::domains::structure;
use crate::domains::work;
use crate::platform::ids;
use crate::providers::cli;
use crate::providers::git as gitpkg;
use crate::providers::sqlite::{self, Store};

/// The errors this package returns, one per Go sentinel plus the store and
/// domain errors its reads pass through.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The text has error diagnostics; nothing was written.
    #[error("polter text has errors")]
    PolterInvalid,
    /// No project id was given and none contains the directory.
    #[error("no project for this directory: pass --project <id> or run polterware init")]
    NoProject,
    /// `agentrun::Error::ChangeIncomplete` plus the frontmatter hint
    /// [`apply_polter`] adds.
    #[error(
        "change step requires summary and rationale: add them to the frontmatter or pass --summary and --rationale"
    )]
    ChangeIncompleteHint,
    /// `agentrun::Error::ChangeIncomplete` plus the frontmatter hint
    /// [`apply_in_session`] adds.
    #[error(
        "change step requires summary and rationale: add summary and rationale to the frontmatter"
    )]
    ChangeIncompleteFrontmatter,
    #[error(transparent)]
    Sqlite(#[from] sqlite::Error),
    #[error(transparent)]
    AgentRun(#[from] agentrun::Error),
    #[error(transparent)]
    Structure(#[from] structure::Error),
    #[error("{0}")]
    Message(String),
}

/// The package's own result: the signed default lets callers write
/// `Result<T>`, keeping Go's one-error shape without re-exporting an alias that
/// would collide with a sibling file's.
type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    /// Reports whether this is [`Error::PolterInvalid`].
    pub fn is_polter_invalid(&self) -> bool {
        matches!(self, Error::PolterInvalid)
    }
}

/// An apply that failed, with the result the caller still reports: the
/// diagnostics of invalid text, or the envelope resolved before the write.
#[derive(Debug)]
pub struct ApplyFailure {
    /// The result the caller reports either way, boxed to keep the `Err`
    /// variant small.
    pub result: Box<ApplyResult>,
    pub error: Error,
}

impl ApplyFailure {
    /// Reports whether the text was refused for its error diagnostics.
    pub fn is_polter_invalid(&self) -> bool {
        self.error.is_polter_invalid()
    }
}

impl std::fmt::Display for ApplyFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl std::error::Error for ApplyFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Resolves the project with `id`, or, when `id` is empty, the project whose
/// root contains `dir` (the deepest root wins).
pub fn resolve_project(store: &Store, id: &str, dir: &str) -> Result<projects::Project> {
    let id = id.trim();
    if !id.is_empty() {
        return store
            .get_project(id)
            .map_err(|_| Error::Message(format!("unknown project {}", polter::go_quote(id))));
    }
    let dir = absolute_path(dir)?;
    let items = store.list_projects(false)?;
    // The project whose repository clone holds dir most deeply wins.
    let mut best: Option<projects::Project> = None;
    let mut best_len: isize = -1;
    for item in items {
        let checkouts = project_checkouts(store, &item.id)?;
        for co in &checkouts {
            let root = clean_path(&co.root);
            let Some(relative) = relative_path(&root, &dir) else {
                continue;
            };
            if relative == ".." || relative.starts_with("../") {
                continue;
            }
            if root.len() as isize > best_len {
                best = Some(item.clone());
                best_len = root.len() as isize;
            }
        }
    }
    best.ok_or(Error::NoProject)
}

/// Loads what reference-level lint and hints check against.
pub fn polter_references(store: &Store, project_id: &str) -> Result<polter::References> {
    let snapshot = store.load_work_snapshot(project_id)?;
    polter_references_from_snapshot(store, &snapshot)
}

fn polter_references_from_snapshot(
    store: &Store,
    snapshot: &work::Snapshot,
) -> Result<polter::References> {
    let decisions = store.list_knowledge(&snapshot.project_id, "decision")?;
    let mut refs = polter::References {
        cards: snapshot.cards.clone(),
        nodes: snapshot.nodes.clone(),
        repos: snapshot.repos.clone(),
        environments: snapshot.environments.clone(),
        ..polter::References::default()
    };
    for item in &decisions {
        refs.decisions.push(polter::DecisionRef {
            id: item.id.clone(),
            title: item.title.clone(),
            status: item.status.clone(),
        });
    }
    let view = project_git(store, &snapshot.project_id);
    refs.git = Some(view);
    Ok(refs)
}

/// Reads the project's first repository clone (by name). None returns an empty
/// view; callers still answer.
pub fn project_git(store: &Store, project_id: &str) -> polter::GitView {
    let checkouts = project_checkouts(store, project_id).unwrap_or_default();
    let Some(first) = checkouts.first() else {
        return polter::GitView::default();
    };
    let obs = gitpkg::observe(&first.root);
    polter::GitView {
        available: obs.available,
        branch: obs.branch,
        files: obs.files,
        delivered: obs.delivered,
    }
}

/// One `polterware apply` / `polter.apply` call.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ApplyInput {
    pub project_id: String,
    pub text: String,
    pub dry_run: bool,
    /// `summary` and `rationale` override the text's frontmatter.
    pub summary: String,
    pub rationale: String,
    pub agent_run_id: String,
    /// Opens a run for this write when `agent_run_id` is empty.
    pub auto_start_label: String,
    /// `actor_kind` and `tool` default to an agent write through `polter.apply`;
    /// the Desktop passes a human actor and its own tool name.
    pub actor_kind: String,
    pub tool: String,
    /// The session agent label that claim, done, and release lines act for.
    /// Empty rejects those lines.
    pub agent: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ApplyResult {
    pub project_id: String,
    pub ops: Vec<work::Op>,
    pub diagnostics: Vec<polter::Diagnostic>,
    pub applied: bool,
    pub agent_run_id: String,
    pub batch_id: String,
    /// `summary` and `rationale` are the resolved change envelope (flags over
    /// frontmatter), filled even on a dry run.
    pub summary: String,
    pub rationale: String,
}

/// Decodes, lints, plans, and (unless `dry_run`) applies Polter text as one
/// audited write. On [`Error::PolterInvalid`] the failure carries the
/// diagnostics.
pub fn apply_polter(
    store: &Store,
    in_: ApplyInput,
) -> std::result::Result<ApplyResult, ApplyFailure> {
    let mut result = ApplyResult {
        project_id: in_.project_id.clone(),
        ..ApplyResult::default()
    };
    let fail = |result: &ApplyResult, error: Error| ApplyFailure {
        result: Box::new(result.clone()),
        error,
    };
    let (text, proposed) = resolve_node_dirs(store, &in_.project_id, &in_.text, in_.dry_run)
        .map_err(|error| fail(&result, error))?;
    let mut snapshot = store
        .load_work_snapshot(&in_.project_id)
        .map_err(|error| fail(&result, error.into()))?;
    snapshot.repos.extend(proposed);
    let references =
        polter_references_from_snapshot(store, &snapshot).map_err(|error| fail(&result, error))?;
    let (doc, diags) = polter::lint(&text, Some(&references));
    let (ops, plan_diags) = polter::plan_as(&doc, &snapshot, &in_.agent);
    result.diagnostics = merge_diagnostics(&diags, &plan_diags);
    if polter::has_errors(&result.diagnostics) {
        return Err(fail(&result, Error::PolterInvalid));
    }
    result.ops = ops;
    result.summary = first_non_empty(&[&in_.summary, &doc.summary]);
    result.rationale = first_non_empty(&[&in_.rationale, &doc.rationale]);
    if in_.dry_run || result.ops.is_empty() {
        return Ok(result);
    }

    let summary = result.summary.clone();
    let rationale = result.rationale.clone();
    let actor_kind = first_non_empty(&[&in_.actor_kind, agentrun::ACTOR_KIND_AGENT]);
    if agentrun::validate_change_as(&actor_kind, &summary, &rationale).is_err() {
        return Err(fail(&result, Error::ChangeIncompleteHint));
    }
    let tool = first_non_empty(&[&in_.tool, "polter.apply"]);
    let batch_id = ids::new_ulid();
    result.batch_id = batch_id.clone();
    // A fact records who wrote it, like every other field the plan resolved.
    let fact_actor = first_non_empty(&[&in_.agent, &actor_kind]);
    for op in &mut result.ops {
        if op.entity == work::ENTITY_FACT {
            op.fields
                .insert("actor".to_string(), Value::String(fact_actor.clone()));
        }
    }
    let write_ops = result.ops.clone();
    let project_id = in_.project_id.clone();
    let written = execute_audited_write(
        store,
        ChangeEnvelope {
            summary,
            rationale,
            agent_run_id: in_.agent_run_id.clone(),
            project_id: project_id.clone(),
            paths: Vec::new(),
            auto_start_label: in_.auto_start_label.clone(),
            batch_id: batch_id.clone(),
        },
        &actor_kind,
        &tool,
        |run| {
            let card_ids = store.apply_work_ops(&project_id, &run.id, &batch_id, &write_ops)?;
            Ok((card_ids, write_ops))
        },
    )
    .map_err(|error| fail(&result, error.into()))?;
    result.applied = true;
    result.agent_run_id = written.agent_run_id;
    if let Err(error) = prune_repositories(store, &in_.project_id, &result.ops) {
        return Err(fail(&result, error));
    }
    Ok(result)
}

/// Answers a v1 query ("cards", "card <slug>", "status", "inbox",
/// "handoff <slug>", "oracle", "structure", "decisions [state]",
/// "decision <decision>", "feature <slug>", "verify <target>", "canvas",
/// "canvas <name|id>") in text.
pub fn query_polter(store: &Store, project_id: &str, text: &str) -> Result<String> {
    query_polter_scoped(store, project_id, text, &structure::MultiScope::default())
}

/// Answers a query over the cards, Decisions and Inbox items inside scope. An
/// inactive scope is the whole project. Several nodes are a union.
pub fn query_polter_scoped(
    store: &Store,
    project_id: &str,
    text: &str,
    scope: &structure::MultiScope,
) -> Result<String> {
    let query = polter::parse_query(text).map_err(Error::Message)?;
    let snapshot = store.load_work_snapshot(project_id)?;
    let snapshot = super::scope::scope_snapshot(store, snapshot, scope)?;
    match query.kind.as_str() {
        polter::QUERY_CARD => {
            let Some(card) = snapshot.card(&query.slug) else {
                return Err(Error::Message(format!(
                    "unknown card {}: list cards with polterware query cards",
                    polter::go_quote(&query.slug)
                )));
            };
            Ok(polter::encode_card(card))
        }
        polter::QUERY_HANDOFF => query_handoff(store, &snapshot, &query.slug),
        polter::QUERY_STATUS => Ok(query_status(store, &snapshot, scope)),
        polter::QUERY_INBOX => {
            let result = super::scope::inbox_scoped(store, &snapshot, scope)?;
            Ok(inbox::list(&result))
        }
        polter::QUERY_BRIEF => query_brief(store, &snapshot, scope, &query.slug, query.budget),
        polter::QUERY_SEARCH => query_search(store, &snapshot, scope, &query.slug, query.budget),
        polter::QUERY_SHOW => query_show(store, &snapshot, &query.slug),
        polter::QUERY_COMMITS => query_commits(store, project_id, &query.slug),
        polter::QUERY_CONFLICTS => Ok(polter::encode_conflicts(&snapshot)),
        polter::QUERY_DEPLOYS => {
            deploys_text(store, project_id, &query.slug, OffsetDateTime::now_utc())
        }
        polter::QUERY_CONFLICT => {
            let Some(conflict) = snapshot.conflict(&query.slug) else {
                return Err(Error::Message(format!(
                    "unknown conflict {}: list them with polterware query conflicts",
                    polter::go_quote(&query.slug)
                )));
            };
            Ok(polter::encode_conflict(conflict, &snapshot))
        }
        polter::QUERY_DECISIONS => Ok(polter::encode_decisions(&snapshot.decisions, &query.slug)),
        polter::QUERY_DECISION => {
            let Some(target) = decision::find(&snapshot.decisions, &query.slug) else {
                return Err(Error::Message(format!(
                    "unknown decision {}: list decisions with polterware decision list",
                    polter::go_quote(&query.slug)
                )));
            };
            Ok(polter::encode_decision(
                &target,
                &snapshot.decisions,
                &polter::nodes_citing(&snapshot.nodes, &target),
            ))
        }
        polter::QUERY_FEATURE | polter::QUERY_VERIFY => query_feature(store, &snapshot, &query),
        polter::QUERY_CANVAS => query_canvas(store, project_id, &query.slug, scope),
        polter::QUERY_MILESTONES => Ok(polter::encode_milestones(&snapshot)),
        polter::QUERY_ROADMAP => Ok(polter::encode_roadmap(&snapshot)),
        polter::QUERY_STRUCTURE => {
            let project = store.get_project(project_id)?;
            if query.history {
                let entries = structure_history(store, project_id)?;
                return Ok(super::structure_history::encode_structure_history(&entries));
            }
            if !query.at.is_empty() {
                let rows = structure_at(store, project_id, &query.at)?;
                return Ok(format!(
                    "Structure at {} (structure only: status and features are live)\n{}",
                    query.at,
                    polter::encode_structure(&project.name, &rows, &[], &[], &query.slug)
                ));
            }
            Ok(polter::encode_structure(
                &project.name,
                &structure_rows(store, &snapshot),
                &snapshot.cards,
                &snapshot.rules,
                &query.slug,
            ))
        }
        _ => Ok(polter::encode_list(&snapshot)),
    }
}

fn query_handoff(store: &Store, snapshot: &work::Snapshot, slug: &str) -> Result<String> {
    let Some(card) = snapshot.card(slug) else {
        return Err(Error::Message(format!(
            "unknown card {}: list cards with polterware query cards",
            polter::go_quote(slug)
        )));
    };
    let view = project_git(store, &snapshot.project_id);
    let mut handoff = card.latest_handoff(&view.branch);
    if handoff.is_none() || view.branch.is_empty() || view.branch == "HEAD" {
        handoff = card.latest_handoff_any();
    }
    let Some(handoff) = handoff else {
        return Err(Error::Message(format!(
            "no handoff for {}: write one with polterware handoff --card {slug}",
            polter::go_quote(slug)
        )));
    };
    Ok(polter::encode_handoff(card, handoff))
}

fn query_status(store: &Store, snapshot: &work::Snapshot, scope: &structure::MultiScope) -> String {
    // The Inbox is a best-effort addition: a read that fails leaves status
    // without its inbox lines rather than failing the session start.
    let mut inbox_lines = String::new();
    if let Ok(result) = super::scope::inbox_scoped(store, snapshot, scope) {
        inbox_lines = inbox::status_lines(&result);
    }
    inbox_lines.push_str(&polter::encode_status_conflicts(
        snapshot,
        &sqlite::format_rfc3339(OffsetDateTime::now_utc() - time::Duration::hours(24)),
    ));
    let view = project_git(store, &snapshot.project_id);
    let card = match select_work_card(snapshot, "", &view.branch) {
        Ok(card) => card,
        Err((slugs, _)) => {
            return polter::encode_status_unresolved(&view.branch, &slugs)
                + &polter::encode_status_claims("", snapshot)
                + &inbox_lines;
        }
    };
    let current = if !view.branch.is_empty() && view.branch != "HEAD" {
        card.latest_handoff(&view.branch).cloned()
    } else {
        None
    };
    let stage = polter::display_stage(&card.stage, view.delivered);
    polter::encode_status(&card, &view.branch, stage, current.as_ref())
        + &polter::encode_status_claims(&card.slug, snapshot)
        + &inbox_lines
}

/// Keeps lint output, then plan diagnostics that lint did not already report
/// (`check_graph` runs in both).
fn merge_diagnostics(
    lint: &[polter::Diagnostic],
    plan: &[polter::Diagnostic],
) -> Vec<polter::Diagnostic> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<polter::Diagnostic> = Vec::with_capacity(lint.len() + plan.len());
    for diag in lint.iter().chain(plan) {
        let key = format!("{}\u{0}{}\u{0}{}", diag.line, diag.severity, diag.message);
        if !seen.insert(key) {
            continue;
        }
        out.push(diag.clone());
    }
    out
}

/// The first value that is not blank, trimmed; empty when every value is.
fn first_non_empty(values: &[&str]) -> String {
    for value in values {
        if !value.trim().is_empty() {
            return value.trim().to_string();
        }
    }
    String::new()
}

/// The first value that is not blank; unlike [`first_non_empty`] it is not
/// trimmed, which is how `deploys.go`'s `firstNonEmptyText` reads.
fn first_non_empty_text(values: &[&str]) -> String {
    for value in values {
        if !value.trim().is_empty() {
            return value.to_string();
        }
    }
    String::new()
}

// ---------------------------------------------------------------------------
// TODO(dedupe): private copies of helpers the sibling files of this package own
// (`repos.go`, `brief.go`, `show.go`, `commits.go`, `feature.go`,
// `canvas_query.go`, `deploys.go`, `handoff.go`). They exist so this file
// compiles next to those files' ports and are deleted once the copies merge.
// ---------------------------------------------------------------------------

/// One repository cloned on this machine.
struct GitCheckout {
    name: String,
    root: String,
}

/// Lists the project's repositories that have a clone here.
fn project_checkouts(store: &Store, project_id: &str) -> Result<Vec<GitCheckout>> {
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

static REPO_NAME_UNSAFE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^a-z0-9]+").expect("the repository name pattern"));

/// The GitHub-level identity of a clone's origin (github.com/owner/repo), empty
/// when it has no GitHub remote.
fn repository_remote(dir: &str) -> String {
    match gitpkg::github_repo(dir) {
        Some(owner_repo) => format!("github.com/{owner_repo}"),
        None => String::new(),
    }
}

/// Proposes a repository name: the GitHub repository's, else the folder's, as a
/// lowercase slug.
fn repo_name_for(remote: &str, dir: &str) -> String {
    let mut base = path_base(dir);
    if !remote.is_empty() {
        base = remote[remote.rfind('/').map_or(0, |i| i + 1)..].to_string();
    }
    let mut name = REPO_NAME_UNSAFE
        .replace_all(&base.to_lowercase(), "-")
        .trim_matches('-')
        .to_string();
    if name.len() > 32 {
        name = name[..32].trim_matches('-').to_string();
    }
    if name.is_empty() {
        name = "repo".to_string();
    }
    name
}

/// Returns the project's repository for a local folder: the one cloned there,
/// else the one with the same GitHub remote (its clone path becomes `dir`),
/// else a new one named after the remote or the folder.
fn ensure_repository(store: &Store, project_id: &str, dir: &str) -> Result<projects::Repository> {
    let abs = absolute_dir(dir)?;
    let existing = store.list_repositories(project_id)?;
    for repo in &existing {
        if !repo.local_path.is_empty() && same_path(&expand_path(&repo.local_path), &abs) {
            return Ok(repo.clone());
        }
    }
    let remote = repository_remote(&abs);
    for repo in &existing {
        if !remote.is_empty() && repo.remote == remote {
            let mut repo = repo.clone();
            repo.local_path = abs.clone();
            store.save_repository(repo.clone())?;
            return Ok(repo);
        }
    }
    let base = repo_name_for(&remote, &abs);
    let taken: HashSet<&str> = existing.iter().map(|repo| repo.name.as_str()).collect();
    let mut name = base.clone();
    let mut i = 2;
    while taken.contains(name.as_str()) {
        let stripped = base
            .strip_suffix(&format!("-{}", i - 1))
            .unwrap_or(&base)
            .to_string();
        name = format!("{stripped}-{i}");
        i += 1;
    }
    let repo = projects::new_repository(&ids::new_ulid(), project_id, &name, &remote, &abs)
        .map_err(|err| Error::Message(err.to_string()))?;
    store.save_repository(repo.clone())?;
    Ok(repo)
}

fn absolute_dir(path: &str) -> Result<String> {
    let expanded = expand_path(path);
    if expanded.is_empty() {
        return Err(Error::Message("repository folder is empty".to_string()));
    }
    let abs = absolute_path(&expanded)?;
    match std::fs::metadata(&abs) {
        Ok(info) if !info.is_dir() => Err(Error::Message(format!(
            "repository folder {abs} is not a directory"
        ))),
        Ok(_) => Ok(abs),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Err(Error::Message(format!(
            "repository folder {abs} does not exist"
        ))),
        Err(err) => Err(Error::Message(err.to_string())),
    }
}

fn same_path(a: &str, b: &str) -> bool {
    clean_path(a) == clean_path(b)
}

/// Rewrites each node's `dir: <folder>` line to `repo: <name>`, the repository
/// cloned there (created when the project has none). A dry run creates nothing
/// and returns the names it would create.
fn resolve_node_dirs(
    store: &Store,
    project_id: &str,
    text: &str,
    dry_run: bool,
) -> Result<(String, Vec<String>)> {
    let (doc, _) = polter::decode(text);
    let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
    let mut proposed: Vec<String> = Vec::new();
    for node in &doc.nodes {
        for entry in &node.meta {
            if entry.key != "dir" || entry.line < 1 || entry.line as usize > lines.len() {
                continue;
            }
            let name = if dry_run {
                let abs = absolute_dir(&entry.value)?;
                let mut name = repo_name_for(&repository_remote(&abs), &abs);
                for repo in store.list_repositories(project_id)? {
                    if same_path(&expand_path(&repo.local_path), &abs) {
                        name = repo.name;
                    }
                }
                proposed.push(name.clone());
                name
            } else {
                ensure_repository(store, project_id, &entry.value)?.name
            };
            let index = (entry.line - 1) as usize;
            let line = lines[index].clone();
            if let Some(start) = line.find("dir:") {
                let rest = &line[start + "dir:".len()..];
                let end = rest.find(" · ").unwrap_or(rest.len());
                lines[index] = format!("{}repo: {}{}", &line[..start], name, &rest[end..]);
            }
        }
    }
    Ok((lines.join("\n"), proposed))
}

/// Removes the repositories a write unlinked from its last node, so a
/// repository lives only while a component uses it.
fn prune_repositories(store: &Store, project_id: &str, ops: &[work::Op]) -> Result<()> {
    for op in ops {
        if op.entity != work::ENTITY_NODE {
            continue;
        }
        if let Some(name) = op.before.get("repo").and_then(Value::as_str)
            && !name.is_empty()
        {
            store.delete_unused_repository(project_id, name)?;
        }
    }
    Ok(())
}

/// Returns nodes where a node naming no repository belongs to the project's
/// only one, so a one-repository project needs no `repo:` lines.
fn with_default_repo(nodes: &[structure::Node], checkouts: &[GitCheckout]) -> Vec<structure::Node> {
    if checkouts.len() != 1 {
        return nodes.to_vec();
    }
    nodes
        .iter()
        .map(|node| {
            let mut node = node.clone();
            if node.repo.is_empty() {
                node.repo = checkouts[0].name.clone();
            }
            node
        })
        .collect()
}

/// Finds the Structure node whose checkout holds a file: the node with the
/// longest `path:` prefix in the file's checkout (the project root, or a
/// registered repository). The file is absolute, relative to the project root,
/// or written as `repo:path`. `None` when no node claims it.
fn node_for_path(
    store: &Store,
    snapshot: &work::Snapshot,
    file: &str,
) -> Result<Option<structure::Node>> {
    let file = file.trim();
    if file.is_empty() {
        return Ok(None);
    }
    let repos = project_checkouts(store, &snapshot.project_id)?;
    let mut repo = String::new();
    let mut rel;
    if Path::new(file).is_absolute() {
        // The deepest checkout that holds the file wins: a repo may sit inside
        // another.
        let mut best: isize = -1;
        let mut best_rel = String::new();
        for item in &repos {
            if item.root.is_empty() {
                continue;
            }
            let Some(inside) = relative_path(&clean_path(&item.root), &clean_path(file)) else {
                continue;
            };
            if inside == ".." || inside.starts_with("../") || item.root.len() as isize <= best {
                continue;
            }
            best = item.root.len() as isize;
            repo = item.name.clone();
            best_rel = inside;
        }
        if best < 0 {
            return Ok(None);
        }
        rel = best_rel;
    } else {
        rel = file.to_string();
        if let Some((name, rest)) = file.split_once(':')
            && repos.iter().any(|item| item.name == name)
        {
            repo = name.to_string();
            rel = rest.to_string();
        }
    }
    let rel = slash_path(&clean_path(&rel));
    let mut found = structure::Node::default();
    let mut ok = false;
    for node in with_default_repo(&snapshot.nodes, &repos) {
        // A relative path with no `repo:` prefix may sit in any repository.
        if node.status == structure::NODE_REMOVED
            || node.folder.is_empty()
            || (!repo.is_empty() && node.repo != repo)
        {
            continue;
        }
        let folder = slash_path(&clean_path(&node.folder))
            .trim_matches('/')
            .to_string();
        if rel != folder && !rel.starts_with(&format!("{folder}/")) {
            continue;
        }
        if !ok || folder.len() > found.folder.trim_matches('/').len() {
            found = node;
            ok = true;
        }
    }
    Ok(if ok { Some(found) } else { None })
}

/// Loads the in-scope docs and agent-visible canvases next to the snapshot.
/// Canvas nodes linked to Structure carry the node's title.
fn brief_input(
    store: &Store,
    snapshot: &work::Snapshot,
    scope: &structure::MultiScope,
) -> Result<polter::BriefInput> {
    let resolved = store.load_scopes(&snapshot.project_id, scope)?;
    let items = store.list_knowledge(&snapshot.project_id, knowledge::KIND_DOC)?;
    let scopes = store.scope_map(&snapshot.project_id, sqlite::SCOPE_KNOWLEDGE)?;
    let mut in_ = polter::BriefInput {
        snapshot: snapshot.clone(),
        ..polter::BriefInput::default()
    };
    for item in items {
        let ids = scopes.get(&item.id).cloned().unwrap_or_default();
        if !resolved.matches_any(&ids) {
            continue;
        }
        in_.docs.push(polter::Doc {
            id: item.id,
            title: item.title,
            doc_type: item.doc_type,
            scope_node_ids: ids,
            body: item.body,
        });
    }
    for mut item in store.list_canvases(&snapshot.project_id)? {
        if !item.agent_visible || !resolved.matches(&item.scope_node_id) {
            continue;
        }
        item.graph = canvas::with_structure(&item.graph, &snapshot.nodes);
        in_.canvases.push(item);
    }
    Ok(in_)
}

/// Renders the brief of a card, a Structure node, or a file. A target that is
/// neither a card nor a node is read as a file path and briefed as the node
/// whose checkout holds it.
fn query_brief(
    store: &Store,
    snapshot: &work::Snapshot,
    scope: &structure::MultiScope,
    target: &str,
    budget: i32,
) -> Result<String> {
    let in_ = brief_input(store, snapshot, scope)?;
    match polter::brief(&in_, target, budget) {
        Ok(text) => Ok(text),
        Err(polter::BriefError::UnknownTarget(_)) => {
            let Some(node) = node_for_path(store, snapshot, target)? else {
                return Err(Error::Message(
                    polter::BriefError::UnknownTarget(trimmed(target)).to_string(),
                ));
            };
            let text = polter::brief(&in_, &node.path, budget)
                .map_err(|err| Error::Message(err.to_string()))?;
            Ok(format!(
                "file {} is under {}\n{text}",
                target.trim(),
                node.path
            ))
        }
    }
}

fn query_search(
    store: &Store,
    snapshot: &work::Snapshot,
    scope: &structure::MultiScope,
    terms: &str,
    budget: i32,
) -> Result<String> {
    let in_ = brief_input(store, snapshot, scope)?;
    Ok(polter::search(&in_, terms, budget))
}

static RUN_DATE: LazyLock<&[time::format_description::FormatItem<'static>]> =
    LazyLock::new(|| format_description!("[year]-[month]-[day] [hour]:[minute]"));
static STEP_TIME: LazyLock<&[time::format_description::FormatItem<'static>]> =
    LazyLock::new(|| format_description!("[hour]:[minute]"));

/// Renders one record by id as text: a card (by slug), a Decision (by ref or
/// id), a fact or Rule (by slug or id), a doc with its body, a Canon entity, or
/// an AgentRun with its steps.
fn query_show(store: &Store, snapshot: &work::Snapshot, id: &str) -> Result<String> {
    let id = id.trim();
    if let Some(card) = snapshot.card(id) {
        return Ok(polter::encode_card(card));
    }
    if let Some(target) = decision::find(&snapshot.decisions, id) {
        return Ok(polter::encode_decision(
            &target,
            &snapshot.decisions,
            &polter::nodes_citing(&snapshot.nodes, &target),
        ));
    }
    for fact in &snapshot.facts {
        if fact.fact.id != id && (fact.slug.is_empty() || fact.slug != id) {
            continue;
        }
        let mut out = format!(
            "fact {} [{}] {}\n",
            fact.fact.id, fact.fact.truth_class, fact.fact.statement
        );
        if !fact.slug.is_empty() {
            out.push_str(&format!("slug: {}\n", fact.slug));
        }
        if !fact.scopes.is_empty() {
            out.push_str(&format!("scope: {}\n", fact.scopes.join(", ")));
        }
        out.push_str(&format!(
            "source: {} · actor: {}\n",
            fact.fact.source, fact.fact.actor
        ));
        return Ok(out);
    }
    for rule in &snapshot.rules {
        if rule.id != id && (rule.slug.is_empty() || rule.slug != id) {
            continue;
        }
        let text = format!("rule {} {}\n", rule.handle(), rule.statement);
        if !rule.scopes.is_empty() {
            return Ok(format!("{text}scope: {}\n", rule.scopes.join(", ")));
        }
        return Ok(format!("{text}scope: whole project\n"));
    }
    if let Ok(item) = store.get_knowledge(id)
        && item.project_id == snapshot.project_id
        && !item.archived()
    {
        let scopes = store.scope_map(&snapshot.project_id, sqlite::SCOPE_KNOWLEDGE)?;
        let mut text = format!("doc {} {}\n", item.id, polter::go_quote(&item.title));
        let mut paths: Vec<String> = Vec::new();
        let contexts = scopes.get(&item.id).cloned().unwrap_or_default();
        for node in &snapshot.nodes {
            if contexts.contains(&node.id) {
                paths.push(node.path.clone());
            }
        }
        if !paths.is_empty() {
            paths.sort();
            text.push_str(&format!("scope: {}\n", paths.join(", ")));
        }
        if !item.tags.is_empty() {
            text.push_str(&format!("tags: {}\n", item.tags.join(", ")));
        }
        return Ok(format!("{text}\n{}\n", item.body.trim_end_matches('\n')));
    }
    if let Ok(entity) = store.get_canon_entity(id)
        && entity.project_id == snapshot.project_id
    {
        return Ok(format!(
            "entity {} [{}] {}\nref: {}/{} · truth: {} · source: {}\n",
            entity.id,
            entity.kind,
            entity.title,
            entity.ref_kind,
            entity.ref_id,
            entity.truth_class,
            entity.source
        ));
    }
    if let Ok(record) = store.get_agent_run_record(id)
        && record.run.project_id == snapshot.project_id
    {
        let run = &record.run;
        let started = run.created_at.format(&*RUN_DATE).unwrap_or_default();
        let mut out = format!(
            "run {} [{}] {}\nagent: {} · actor: {} · started {started}\n",
            run.id, run.status, run.task, run.agent_label, run.actor_kind
        );
        for step in &record.steps {
            let at = step.created_at.format(&*STEP_TIME).unwrap_or_default();
            out.push_str(&format!("- {at} {}: {}\n", step.kind, step.summary));
        }
        return Ok(out);
    }
    Err(Error::Message(format!(
        "unknown id {}: show takes a card slug, a Decision ref, a fact, rule, doc, entity, or run id (search <terms> finds them)",
        polter::go_quote(id)
    )))
}

/// One commit of a card's task, for the card page.
#[derive(Debug, Clone, PartialEq)]
struct CardCommit {
    task_number: i32,
    repo: String,
    sha: String,
    subject: String,
    url: String,
    committed_at: OffsetDateTime,
    /// True for a commit found by its subject (`slug#T2`), with no stored link.
    cited: bool,
}

impl Default for CardCommit {
    fn default() -> Self {
        Self {
            task_number: 0,
            repo: String::new(),
            sha: String::new(),
            subject: String::new(),
            url: String::new(),
            committed_at: OffsetDateTime::UNIX_EPOCH,
            cited: false,
        }
    }
}

impl CardCommit {
    /// The first seven characters of the sha.
    fn short_sha(&self) -> &str {
        if self.sha.len() > 7 {
            &self.sha[..7]
        } else {
            &self.sha
        }
    }
}

/// Lists a card's commits, newest first: the ones linked to its tasks, then the
/// ones whose subject cites a task (`slug#T2`) in a checkout's recent history.
fn card_commits(store: &Store, project_id: &str, slug: &str) -> Result<Vec<CardCommit>> {
    let snapshot = store.load_work_snapshot(project_id)?;
    let Some(card) = snapshot.card(slug) else {
        return Err(Error::Message(format!(
            "commit input: unknown card {}",
            polter::go_quote(slug)
        )));
    };
    let mut numbers: HashMap<String, i32> = HashMap::new();
    for task in &card.tasks {
        numbers.insert(task.id.clone(), task.number);
    }
    let links = store.list_commit_links(project_id)?;
    let mut seen: HashSet<String> = HashSet::new();
    let key = |number: i32, sha: &str| format!("{number}:{sha}");
    let mut out: Vec<CardCommit> = Vec::new();
    for link in &links {
        let Some(number) = numbers.get(&link.task_id).copied() else {
            continue;
        };
        seen.insert(key(number, &link.sha));
        out.push(CardCommit {
            task_number: number,
            repo: link.repo.clone(),
            sha: link.sha.clone(),
            subject: link.subject.clone(),
            url: link.url.clone(),
            committed_at: link.committed_at,
            cited: false,
        });
    }
    let checkouts = project_checkouts(store, project_id)?;
    for co in &checkouts {
        let meta = gitpkg::read_metadata(&co.root);
        if !meta.available {
            continue;
        }
        let owner_repo = gitpkg::github_repo(&co.root);
        for commit in &meta.commits {
            for task in polter::match_explicit_tasks(&commit.subject, std::slice::from_ref(card)) {
                let Some(number) = numbers.get(&task.id).copied() else {
                    continue;
                };
                if seen.contains(&key(number, &commit.sha)) {
                    continue;
                }
                seen.insert(key(number, &commit.sha));
                let mut found = CardCommit {
                    task_number: number,
                    repo: co.name.clone(),
                    sha: commit.sha.clone(),
                    subject: commit.subject.clone(),
                    cited: true,
                    ..CardCommit::default()
                };
                if let Some(owner_repo) = &owner_repo {
                    found.url = gitpkg::github_commit_url(owner_repo, &commit.sha);
                }
                if let Some(when) = gitpkg::commit_time(&co.root, &commit.sha) {
                    found.committed_at = when;
                }
                out.push(found);
            }
        }
    }
    out.sort_by(|a, b| b.committed_at.cmp(&a.committed_at));
    Ok(out)
}

/// Lists a card's commits, one line each: task, short sha, subject, and the
/// GitHub URL when there is one.
fn query_commits(store: &Store, project_id: &str, slug: &str) -> Result<String> {
    let commits = card_commits(store, project_id, slug)?;
    if commits.is_empty() {
        return Ok(format!(
            "no commits for {slug}: cite `{slug}#T2` in a commit subject, or run polterware task done {slug}#T2 --commit <sha>\n"
        ));
    }
    let mut out = format!("{} commits for {slug}\n", commits.len());
    for commit in &commits {
        let mut line = format!(
            "- T{} {} {}",
            commit.task_number,
            commit.short_sha(),
            commit.subject
        );
        if !commit.url.is_empty() {
            line.push(' ');
            line.push_str(&commit.url);
        }
        out.push_str(&line);
        out.push('\n');
    }
    Ok(out)
}

/// The Shape impact and flow checks of one card.
fn review_feature(
    store: &Store,
    snapshot: &work::Snapshot,
    slug: &str,
) -> Result<polter::FeatureReview> {
    let references = polter_references_from_snapshot(store, snapshot)?;
    polter::review_feature(
        snapshot,
        slug,
        &references.decisions,
        &question_routes(store, snapshot),
    )
    .map_err(Error::Message)
}

/// Maps open question ids to where they wait: the Inbox, deferred, or delegated
/// to agents. A failed Inbox read leaves every question in the Inbox.
fn question_routes(store: &Store, snapshot: &work::Snapshot) -> HashMap<String, String> {
    let mut routes: HashMap<String, String> = HashMap::new();
    let Ok(result) = super::scope::list_inbox(store, snapshot) else {
        return routes;
    };
    const PREFIX: &str = "decision:";
    for item in &result.items {
        if let Some(id) = item.key.strip_prefix(PREFIX)
            && item.state == inbox::STATE_DEFERRED
        {
            routes.insert(id.to_string(), polter::ROUTE_DEFERRED.to_string());
        }
    }
    for routed in &result.routed {
        if let Some(id) = routed.item.key.strip_prefix(PREFIX)
            && routed.action == inbox::ACTION_DELEGATE
        {
            routes.insert(id.to_string(), polter::ROUTE_DELEGATED.to_string());
        }
    }
    routes
}

/// Reads whether a card or one task (`slug#T2`) is finished. It never writes
/// and never moves the stored stage.
fn verify_feature(store: &Store, project_id: &str, target: &str) -> Result<polter::VerifyResult> {
    let mut slug = target.trim().to_string();
    let mut number = 0;
    if slug.contains('#') || slug.contains(' ') {
        let reference = polter::parse_task_ref(&slug).map_err(Error::Message)?;
        slug = reference.slug;
        number = reference.number;
    }
    let snapshot = store.load_work_snapshot(project_id)?;
    let review = review_feature(store, &snapshot, &slug)?;
    let mut contract: Vec<polter::ReviewInvariant> = review
        .card
        .invariants
        .iter()
        .filter(|invariant| invariant.state == work::INVARIANT_ACTIVE)
        .map(|invariant| polter::ReviewInvariant {
            text: invariant.text.clone(),
            from: "card".to_string(),
        })
        .collect();
    contract.extend(review.inherited.iter().cloned());
    polter::verify_feature(&polter::VerifyInput {
        card: review.card,
        task: number,
        delivered: project_git(store, project_id).delivered,
        contract,
    })
    .map_err(Error::Message)
}

/// Answers "feature <slug> [step]" and "verify <target>".
fn query_feature(
    store: &Store,
    snapshot: &work::Snapshot,
    query: &polter::Query,
) -> Result<String> {
    if query.kind == polter::QUERY_VERIFY {
        let result = verify_feature(store, &snapshot.project_id, &query.slug)?;
        return Ok(polter::encode_verify(&result));
    }
    let review = review_feature(store, snapshot, &query.slug)?;
    let mut step = query.step.as_str();
    if step.is_empty() {
        step = polter::FEATURE_STEP_SHAPE;
    }
    if !polter::valid_feature_step(step) {
        return Err(Error::Message(format!(
            "unknown step {}: use shape, contract, or plan",
            polter::go_quote(step)
        )));
    }
    Ok(polter::encode_feature_review(&review, step))
}

/// Lists agent-visible canvases, or returns one of them as a Mermaid flowchart.
/// A canvas with `agent_visible` false is omitted from the list and answers as
/// unknown, including when the caller names its id.
fn query_canvas(
    store: &Store,
    project_id: &str,
    reference: &str,
    scope: &structure::MultiScope,
) -> Result<String> {
    let items = store.list_canvases(project_id)?;
    let resolved = store.load_scopes(project_id, scope)?;
    let visible: Vec<canvas::Canvas> = items
        .into_iter()
        .filter(|item| item.agent_visible && resolved.matches(&item.scope_node_id))
        .collect();
    let nodes = store.structure_nodes(project_id)?;
    let reference = reference.trim();
    if reference.is_empty() {
        return Ok(format_agent_canvases(&visible, &nodes));
    }
    let item = find_agent_canvas(&visible, reference)?;
    Ok(canvas::export_mermaid(
        &canvas::with_structure(&item.graph, &nodes),
        &item.layout_json,
    ))
}

fn format_agent_canvases(items: &[canvas::Canvas], nodes: &[structure::Node]) -> String {
    if items.is_empty() {
        return "no canvases\n".to_string();
    }
    let paths: HashMap<&str, &str> = nodes
        .iter()
        .map(|node| (node.id.as_str(), node.path.as_str()))
        .collect();
    let mut out = String::from("id  scope  name\n");
    for item in items {
        let scope = if item.scope_node_id.is_empty() {
            "-".to_string()
        } else {
            paths
                .get(item.scope_node_id.as_str())
                .map(|path| (*path).to_string())
                .unwrap_or_else(|| item.scope_node_id.clone())
        };
        out.push_str(&format!("{}  {}  {}\n", item.id, scope, item.name));
    }
    out
}

fn find_agent_canvas(items: &[canvas::Canvas], reference: &str) -> Result<canvas::Canvas> {
    let mut by_id: Option<&canvas::Canvas> = None;
    let mut by_name: Vec<&canvas::Canvas> = Vec::new();
    for item in items {
        if item.id == reference {
            by_id = Some(item);
        }
        if item.name == reference {
            by_name.push(item);
        }
    }
    if let Some(item) = by_id {
        return Ok(item.clone());
    }
    match by_name.len() {
        1 => Ok(by_name[0].clone()),
        0 => Err(Error::Message(format!(
            "unknown canvas {}: list canvases with polterware query canvas",
            polter::go_quote(reference)
        ))),
        n => Err(Error::Message(format!(
            "canvas name {} matches {n} canvases: use an id",
            polter::go_quote(reference)
        ))),
    }
}

/// Lists a project's deploy links, or one node's, one per line: where it runs,
/// the platform's target, the latest check and its age, and the links.
fn deploys_text(
    store: &Store,
    project_id: &str,
    node_reference: &str,
    now: OffsetDateTime,
) -> Result<String> {
    let node_id = deploy_node_id(store, project_id, node_reference)?;
    let links = store.list_node_deploy_links(project_id)?;
    let mut out = String::new();
    for link in &links {
        if !node_id.is_empty() && link.node_id != node_id {
            continue;
        }
        out.push_str(&deploy_line(link, now));
        out.push('\n');
    }
    if out.is_empty() {
        return Ok(
            "no deploys: link one with deploys: env=provider:resource on a component\n".to_string(),
        );
    }
    Ok(out)
}

/// Resolves a node id or Structure path; empty stays empty.
fn deploy_node_id(store: &Store, project_id: &str, reference: &str) -> Result<String> {
    let reference = reference.trim().trim_matches('/').to_string();
    if reference.is_empty() {
        return Ok(String::new());
    }
    for node in store.structure_nodes(project_id)? {
        if node.status != structure::NODE_REMOVED
            && (node.id == reference || node.path == reference)
        {
            return Ok(node.id);
        }
    }
    Err(Error::Message(format!(
        "unknown Structure node {:?}: list them with polterware structure",
        reference
    )))
}

/// One link as agents read it:
/// `web production: vercel polterware-web@prj_1, failed 10m ago (error), <dashboard>`.
fn deploy_line(link: &inbox::DeployLink, now: OffsetDateTime) -> String {
    let env = first_non_empty_text(&[&link.environment, &link.environment_id]);
    let mut target = link.resource.clone();
    if !link.target_id.is_empty() && link.target_id != link.resource {
        target.push('@');
        target.push_str(&link.target_id);
    }
    let mut line = format!("{} {}: {} {}", link.node_path, env, link.provider, target);
    if link.state.is_empty() {
        line.push_str(", not checked");
    } else {
        line.push_str(", ");
        line.push_str(&link.state);
        if let Ok(at) = OffsetDateTime::parse(&link.checked_at, &Rfc3339) {
            line.push(' ');
            line.push_str(&ago(now - at));
            line.push_str(" ago");
        }
        if !link.summary.is_empty() {
            line.push_str(" (");
            line.push_str(&link.summary);
            line.push(')');
        }
    }
    if !link.url.is_empty() {
        line.push_str(", ");
        line.push_str(&link.url);
    }
    if link.state == cli::STATE_FAILED && !link.state_url.is_empty() && link.state_url != link.url {
        line.push_str(", ");
        line.push_str(&link.state_url);
    }
    line
}

fn ago(d: time::Duration) -> String {
    if d < time::Duration::minutes(1) {
        "<1m".to_string()
    } else if d < time::Duration::hours(1) {
        format!("{}m", d.whole_minutes())
    } else if d < time::Duration::hours(48) {
        format!("{}h", d.whole_hours())
    } else {
        format!("{}d", d.whole_hours() / 24)
    }
}

/// Picks the card a handoff or status command is about. `slug` wins. Otherwise
/// the only active card that already has a handoff on `branch`, otherwise the
/// only active card. The error carries the active cards when the choice is
/// ambiguous.
fn select_work_card(
    snapshot: &work::Snapshot,
    slug: &str,
    branch: &str,
) -> std::result::Result<work::Card, (Vec<String>, Error)> {
    let active: Vec<&work::Card> = snapshot
        .cards
        .iter()
        .filter(|card| card.status == work::CARD_ACTIVE)
        .collect();
    let slugs: Vec<String> = active.iter().map(|card| card.slug.clone()).collect();
    let slug = slug.trim();
    if !slug.is_empty() {
        let Some(card) = snapshot.card(slug) else {
            return Err((
                slugs,
                Error::Message(format!(
                    "unknown card {}: list cards with polterware query cards",
                    polter::go_quote(slug)
                )),
            ));
        };
        if card.status != work::CARD_ACTIVE {
            return Err((
                slugs,
                Error::Message(format!(
                    "unknown card {}: list cards with polterware query cards",
                    polter::go_quote(slug)
                )),
            ));
        }
        return Ok(card.clone());
    }
    if !branch.is_empty() && branch != "HEAD" {
        let on_branch: Vec<&&work::Card> = active
            .iter()
            .filter(|card| card.latest_handoff(branch).is_some())
            .collect();
        if on_branch.len() == 1 {
            return Ok((*on_branch[0]).clone());
        }
    }
    if active.len() == 1 {
        return Ok(active[0].clone());
    }
    if active.is_empty() {
        return Err((
            Vec::new(),
            Error::Message("no active card: create one with polterware apply".to_string()),
        ));
    }
    Err((
        slugs.clone(),
        Error::Message(format!("pass --card <slug>: {}", slugs.join(", "))),
    ))
}

/// The absolute, cleaned form of a path; a relative one resolves against the
/// working directory, as Go's `filepath.Abs` does.
fn absolute_path(path: &str) -> Result<String> {
    let path = path.trim();
    if Path::new(path).is_absolute() {
        return Ok(clean_path(path));
    }
    let cwd = std::env::current_dir().map_err(|err| Error::Message(err.to_string()))?;
    Ok(clean_path(&cwd.join(path).to_string_lossy()))
}

/// The last element of a path (`filepath.Base`).
fn path_base(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return "/".to_string();
    }
    trimmed.rsplit('/').next().unwrap_or(trimmed).to_string()
}

/// A lexical `filepath.Clean`: no empty or `.` parts, and `..` resolved.
fn clean_path(path: &str) -> String {
    let absolute = path.starts_with('/');
    let mut out: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => match out.last() {
                Some(last) if *last != ".." => {
                    out.pop();
                }
                _ => {
                    if !absolute {
                        out.push("..");
                    }
                }
            },
            other => out.push(other),
        }
    }
    let joined = out.join("/");
    if absolute {
        format!("/{joined}")
    } else if joined.is_empty() {
        ".".to_string()
    } else {
        joined
    }
}

/// The relative path of `target` from `root`, as `filepath.Rel` spells it.
fn relative_path(root: &str, target: &str) -> Option<String> {
    let root_parts: Vec<&str> = root.split('/').filter(|part| !part.is_empty()).collect();
    let target_parts: Vec<&str> = target.split('/').filter(|part| !part.is_empty()).collect();
    if root.starts_with('/') != target.starts_with('/') {
        return None;
    }
    let mut i = 0;
    while i < root_parts.len() && i < target_parts.len() && root_parts[i] == target_parts[i] {
        i += 1;
    }
    let mut parts: Vec<String> = Vec::new();
    for _ in i..root_parts.len() {
        parts.push("..".to_string());
    }
    for part in &target_parts[i..] {
        parts.push((*part).to_string());
    }
    if parts.is_empty() {
        Some(".".to_string())
    } else {
        Some(parts.join("/"))
    }
}

/// A path with forward slashes (`filepath.ToSlash`).
fn slash_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn trimmed(value: &str) -> String {
    value.trim().to_string()
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    /// The roadmap every apply test starts from (`work_test.go`'s `roadmap`).
    pub(crate) const ROADMAP: &str = r#"---
polter: 1
summary: Add git import card
rationale: Plan slice 5
---

## git-metadata-import: Git metadata import
stage: plan

Import HEAD, branch and recent commits as evidence.

- [ ] Commit evidence upsert
- [ ] Stamp revision on observed facts
? Include merge commits? [blocking] (yes | no)
"#;

    /// The project `InitProject` registers for a name and no folder: a project
    /// row, its settings, and its default local environment.
    ///
    /// TODO(dedupe): private copy of `init.go`'s `InitProject`; the sibling
    /// module exports the same once the port lands.
    pub(crate) fn init_test_project(store: &Store, name: &str) -> projects::Project {
        let project = projects::new_project(&ids::new_ulid(), name).expect("project");
        store.save_project(project.clone()).expect("save project");
        if store.get_project_settings(&project.id).is_err() {
            store
                .save_project_settings(projects::Settings {
                    project_id: project.id.clone(),
                    ..projects::Settings::default()
                })
                .expect("save settings");
        }
        let envs = store.list_environments(&project.id).expect("environments");
        if envs.is_empty() {
            let mut env = projects::new_environment(
                &format!("{}-local", project.id),
                &project.id,
                "local",
                "local",
            )
            .expect("environment");
            env.is_default = true;
            env.protection = "relaxed".to_string();
            env.description = "local environment".to_string();
            store
                .save_environment(env.clone())
                .expect("save environment");
            store
                .save_project_settings(projects::Settings {
                    project_id: project.id.clone(),
                    default_environment_id: env.id,
                    ..projects::Settings::default()
                })
                .expect("save settings");
        }
        project
    }

    /// Links a local folder as the project's repository, as Go's test helper
    /// `initWithRepo` does through `EnsureRepository`.
    pub(crate) fn add_repository(
        store: &Store,
        project_id: &str,
        root: &str,
    ) -> projects::Repository {
        let repo = projects::new_repository(&ids::new_ulid(), project_id, "demo", "", root)
            .expect("repository");
        store
            .save_repository(repo.clone())
            .expect("save repository");
        repo
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{ROADMAP, add_repository, init_test_project};
    use super::*;
    use crate::providers::sqlite::test_support::TempStore;

    const CLAIM_ROADMAP: &str = r#"---
polter: 1
summary: Add claim cards
rationale: Exercise slice 6
---

## git-import: Git import
stage: build

- [ ] T1 Evidence upsert `backend/import/*`
- [ ] T2 Docs `docs/*`

## canon-core: Canon core
stage: build

- [ ] T1 Core `backend/*`
"#;

    fn temp_root() -> tempfile::TempDir {
        tempfile::tempdir().expect("temp dir")
    }

    fn apply(store: &Store, project_id: &str, text: &str) -> ApplyResult {
        apply_polter(
            store,
            ApplyInput {
                project_id: project_id.to_string(),
                text: text.to_string(),
                auto_start_label: "cli".to_string(),
                ..ApplyInput::default()
            },
        )
        .expect("apply")
    }

    fn apply_with_agent(store: &Store, project_id: &str, text: &str, agent: &str) -> ApplyResult {
        apply_polter(
            store,
            ApplyInput {
                project_id: project_id.to_string(),
                text: text.to_string(),
                auto_start_label: "cli".to_string(),
                agent: agent.to_string(),
                ..ApplyInput::default()
            },
        )
        .expect("apply")
    }

    #[test]
    fn apply_polter_end_to_end() {
        let TempStore { store, .. } = TempStore::new();
        let root = temp_root();
        let root_path = root.path().to_string_lossy().into_owned();
        let project = init_test_project(&store, "Demo");
        add_repository(&store, &project.id, &root_path);

        let resolved = resolve_project(&store, "", &format!("{root_path}/sub")).expect("resolve");
        assert_eq!(resolved.id, project.id);

        let dry = apply_polter(
            &store,
            ApplyInput {
                project_id: project.id.clone(),
                text: ROADMAP.to_string(),
                dry_run: true,
                ..ApplyInput::default()
            },
        )
        .expect("dry run");
        assert!(!dry.applied, "dry run applied");
        assert_eq!(dry.ops.len(), 4, "dry run ops: {:?}", dry.ops);

        let result = apply(&store, &project.id, ROADMAP);
        assert!(result.applied, "apply: {result:?}");
        assert!(!result.agent_run_id.is_empty(), "apply: {result:?}");
        let record = store
            .get_agent_run_record(&result.agent_run_id)
            .expect("run record");
        assert_eq!(record.run.agent_label, "cli");
        assert_eq!(record.run.status, "completed");
        assert_eq!(record.steps.len(), 1, "run steps: {:?}", record.steps);
        assert!(
            record.steps[0].payload_json.contains(&result.batch_id),
            "change step should carry batch id: {}",
            record.steps[0].payload_json
        );
        let op_rows: i64 = store
            .db
            .query_row(
                "SELECT COUNT(1) FROM change_ops WHERE batch_id=?",
                rusqlite::params![result.batch_id],
                |row| row.get(0),
            )
            .expect("change_ops rows");
        assert_eq!(op_rows, 4, "change_ops rows");

        let again = apply(&store, &project.id, ROADMAP);
        assert!(!again.applied, "re-apply should be a no-op");
        assert_eq!(again.ops.len(), 0);

        let list = query_polter(&store, &project.id, "cards").expect("cards");
        assert!(
            list.contains("git-metadata-import  plan  0/2 tasks  1 open?"),
            "list = {list:?}"
        );
        let card = query_polter(&store, &project.id, "card git-metadata-import").expect("card");
        assert!(
            card.contains("- [ ] T2 Stamp revision on observed facts"),
            "card = {card:?}"
        );
    }

    #[test]
    fn apply_polter_rejects_invalid_text() {
        let TempStore { store, .. } = TempStore::new();
        let project = init_test_project(&store, "Demo");
        let failure = apply_polter(
            &store,
            ApplyInput {
                project_id: project.id.clone(),
                text: "## a: A\n".to_string(),
                auto_start_label: "cli".to_string(),
                ..ApplyInput::default()
            },
        )
        .unwrap_err();
        assert!(failure.is_polter_invalid(), "err = {failure}");
        assert!(!failure.result.diagnostics.is_empty());

        let missing = ROADMAP.replace(
            "summary: Add git import card\nrationale: Plan slice 5\n",
            "",
        );
        let failure = apply_polter(
            &store,
            ApplyInput {
                project_id: project.id.clone(),
                text: missing,
                auto_start_label: "cli".to_string(),
                ..ApplyInput::default()
            },
        )
        .unwrap_err();
        assert_eq!(
            failure.to_string(),
            "change step requires summary and rationale: add them to the frontmatter or pass --summary and --rationale"
        );
    }

    #[test]
    fn apply_polter_needs_a_reason_only_from_agents() {
        let TempStore { store, .. } = TempStore::new();
        let project = init_test_project(&store, "Demo");
        let text = "---\npolter: 1\n---\n\n## login: Sign people in\nstage: shape\n";

        let failure = apply_polter(
            &store,
            ApplyInput {
                project_id: project.id.clone(),
                text: text.to_string(),
                summary: "Create card: login".to_string(),
                auto_start_label: "cli".to_string(),
                ..ApplyInput::default()
            },
        )
        .unwrap_err();
        assert_eq!(
            failure.to_string(),
            "change step requires summary and rationale: add them to the frontmatter or pass --summary and --rationale",
            "an agent write needs a reason"
        );

        let result = apply_polter(
            &store,
            ApplyInput {
                project_id: project.id.clone(),
                text: text.to_string(),
                summary: "Create card: login".to_string(),
                actor_kind: agentrun::ACTOR_KIND_HUMAN.to_string(),
                tool: "desktop.work.apply".to_string(),
                ..ApplyInput::default()
            },
        )
        .expect("a person's write needs no reason");
        assert!(result.applied, "a person's write: {result:?}");

        let failure = apply_polter(
            &store,
            ApplyInput {
                project_id: project.id.clone(),
                text: "---\npolter: 1\n---\n\n## login\nstage: plan\n".to_string(),
                actor_kind: agentrun::ACTOR_KIND_HUMAN.to_string(),
                tool: "desktop.work.apply".to_string(),
                ..ApplyInput::default()
            },
        )
        .unwrap_err();
        assert_eq!(
            failure.to_string(),
            "change step requires summary and rationale: add them to the frontmatter or pass --summary and --rationale",
            "a person's write still needs a summary"
        );
    }

    #[test]
    fn polter_fact_apply_round_trip_and_audit() {
        let TempStore { store, .. } = TempStore::new();
        let project = init_test_project(&store, "Facts");
        let header = "---\npolter: 1\nsummary: Promote canvas item\nrationale: promoted from canvas Sketch\n---\n";
        let text = format!(
            "{header}# core: Core\nlevel: domain\n#### local-core: One local core owns SQLite.\nkind: declared\nscope: core\n"
        );
        let apply_as_codex = |text: &str, dry: bool| {
            apply_polter(
                &store,
                ApplyInput {
                    project_id: project.id.clone(),
                    text: text.to_string(),
                    dry_run: dry,
                    auto_start_label: "codex".to_string(),
                    agent: "codex".to_string(),
                    ..ApplyInput::default()
                },
            )
        };
        let dry = apply_as_codex(&text, true).expect("dry");
        assert!(!dry.applied, "dry: {dry:?}");
        assert_eq!(dry.ops.len(), 2, "dry ops: {:?}", dry.ops);
        let facts = store.list_canon_facts(&project.id, "").expect("facts");
        assert!(facts.is_empty(), "dry wrote: {facts:?}");

        let result = apply_as_codex(&text, false).expect("apply");
        assert!(result.applied, "apply: {result:?}");
        let id = result.ops[1].entity_id.clone();
        let fact = store.get_canon_fact(&id).expect("fact");
        assert_eq!(fact.statement, "One local core owns SQLite.");
        assert_eq!(fact.truth_class, "declared");
        assert_eq!(fact.actor, "codex");

        let snapshot = store.load_work_snapshot(&project.id).expect("snapshot");
        assert_eq!(snapshot.facts[0].slug, "local-core");
        assert_eq!(snapshot.facts[0].scopes, vec!["core".to_string()]);

        let (op, batch, actor): (String, String, String) = store
            .db
            .query_row(
                "SELECT op,batch_id,agent_run_id FROM change_ops WHERE entity='fact' AND entity_id=?",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("audit row");
        assert_eq!(op, "create");
        assert_eq!(batch, result.batch_id);
        assert_eq!(actor, result.agent_run_id);

        let record = store
            .get_agent_run_record(&result.agent_run_id)
            .expect("run record");
        assert_eq!(record.steps.len(), 1);
        assert_eq!(record.steps[0].kind, agentrun::KIND_CHANGE);
        assert!(record.steps[0].payload_json.contains(&result.batch_id));

        let again = apply_as_codex(&text, false).expect("idempotence");
        assert!(!again.applied, "idempotence: {again:?}");
        assert_eq!(again.ops.len(), 0);

        for reference in ["local-core", id.as_str()] {
            let updated = apply_as_codex(
                &format!("{header}#### {reference}: Updated {reference}\nkind: inferred\n"),
                false,
            )
            .expect("update");
            assert!(updated.applied, "update: {updated:?}");
            assert_eq!(updated.ops.len(), 1, "update ops: {:?}", updated.ops);
            assert_eq!(updated.ops[0].entity_id, id);
        }
        let cleared = apply_as_codex(
            &format!("{header}#### {id}\nkind: verified\nscope:\n"),
            false,
        )
        .expect("clear");
        assert!(cleared.applied, "clear: {cleared:?}");
        let fact = store.get_canon_fact(&id).expect("verified fact");
        assert_eq!(fact.truth_class, "verified");
        let scope = store
            .scope_map(&project.id, sqlite::SCOPE_FACT)
            .expect("scope");
        assert!(
            scope.get(&id).is_none_or(|nodes| nodes.is_empty()),
            "clear scope: {scope:?}"
        );

        // One invalid block rejects the whole document.
        let bad =
            format!("{header}#### another: Another fact\n#### bad: Rejected\nkind: invalid\n");
        let failure = apply_as_codex(&bad, false).unwrap_err();
        assert!(failure.is_polter_invalid(), "reject: {failure}");
        assert!(polter::has_errors(&failure.result.diagnostics));
        let facts = store.list_canon_facts(&project.id, "").expect("facts");
        assert_eq!(facts.len(), 1, "partial apply: {facts:?}");
    }

    #[test]
    fn structure_kind_and_tech_persist() {
        let TempStore { store, .. } = TempStore::new();
        let project = init_test_project(&store, "Demo");
        apply(
            &store,
            &project.id,
            r#"---
polter: 1
summary: Add the database
rationale: Record what the store is built with
---

# app: App
level: domain

# app/db: Database
level: component
under: app
kind: database
tech: Postgres
"#,
        );
        let db = node_by_path(&store, &project.id, "app/db");
        assert_eq!(db.kind, "database");
        assert_eq!(db.tech, "Postgres");

        let result = apply(
            &store,
            &project.id,
            r#"---
polter: 1
summary: Move to SQLite
rationale: The store is local now
---

# app/db: Database
tech: SQLite
"#,
        );
        assert!(result.applied, "ops: {:?}", result.ops);
        assert_eq!(result.ops.len(), 1, "ops: {:?}", result.ops);
        let db = node_by_path(&store, &project.id, "app/db");
        assert_eq!(db.kind, "database");
        assert_eq!(db.tech, "SQLite");
    }

    #[test]
    fn claim_end_to_end() {
        let TempStore { store, .. } = TempStore::new();
        let project = init_test_project(&store, "Demo");
        apply_with_agent(&store, &project.id, CLAIM_ROADMAP, "");

        let claim = |action: &str, reference: &str, holder: &str| {
            let parsed = polter::parse_task_ref(reference).expect("task ref");
            polter::coord_text(action, &parsed, holder).expect("coord text")
        };
        let claimed = apply_with_agent(
            &store,
            &project.id,
            &claim(polter::COORD_CLAIM, "git-import#T1", "codex"),
            "codex",
        );
        assert!(claimed.applied, "claim: {claimed:?}");
        assert_eq!(claimed.ops.len(), 1, "claim ops: {:?}", claimed.ops);
        let entity: String = store
            .db
            .query_row(
                "SELECT entity FROM change_ops WHERE batch_id=?",
                rusqlite::params![claimed.batch_id],
                |row| row.get(0),
            )
            .expect("change_ops entity");
        assert_eq!(entity, "claim");

        // `backend/*` on another card overlaps codex's `backend/import/*`.
        let failure = apply_polter(
            &store,
            ApplyInput {
                project_id: project.id.clone(),
                text: claim(polter::COORD_CLAIM, "canon-core T1", "claude"),
                auto_start_label: "cli".to_string(),
                agent: "claude".to_string(),
                ..ApplyInput::default()
            },
        )
        .unwrap_err();
        assert!(failure.is_polter_invalid(), "overlap: {failure}");
        assert!(
            failure.result.diagnostics[0]
                .message
                .contains("git-import T1 held by codex"),
            "overlap diags: {:?}",
            failure.result.diagnostics
        );
        // `docs/*` does not overlap.
        apply_with_agent(
            &store,
            &project.id,
            &claim(polter::COORD_CLAIM, "git-import#T2", "claude"),
            "claude",
        );

        let snapshot = store.load_work_snapshot(&project.id).expect("snapshot");
        let card = snapshot.card("git-import").expect("card");
        let held = card.tasks[0].claim.as_ref().expect("stored claim");
        assert_eq!(held.holder, "codex");
        assert_eq!(held.scope, "backend/import/*");
        let encoded = polter::encode_card(card);
        assert!(
            !encoded.contains("claim:"),
            "card read should not return claims:\n{encoded}"
        );
        let claims = polter::encode_status_claims("git-import", &snapshot);
        assert!(
            claims.contains("claim: T1 codex (backend/import/*)"),
            "status claims = {claims:?}"
        );
        assert!(
            claims.contains("claim: T2 claude (docs/*)"),
            "status claims = {claims:?}"
        );

        // Done by the holder marks the task done and frees the scope.
        apply_with_agent(
            &store,
            &project.id,
            &claim(polter::COORD_DONE, "git-import#T1", "codex"),
            "codex",
        );
        apply_with_agent(
            &store,
            &project.id,
            &claim(polter::COORD_CLAIM, "canon-core#T1", "claude"),
            "claude",
        );
        // Release by the holder; a second release is a no-op.
        apply_with_agent(
            &store,
            &project.id,
            &claim(polter::COORD_RELEASE, "git-import#T2", "claude"),
            "claude",
        );
        let again = apply_with_agent(
            &store,
            &project.id,
            &claim(polter::COORD_RELEASE, "git-import#T2", "claude"),
            "claude",
        );
        assert!(!again.applied, "second release should be a no-op");
        let rows: i64 = store
            .db
            .query_row(
                "SELECT COUNT(1) FROM work_claims WHERE project_id=?",
                rusqlite::params![project.id],
                |row| row.get(0),
            )
            .expect("work_claims rows");
        assert_eq!(rows, 1);
    }

    /// The node at `path`, as Go's `nodeByPath` helper reads it.
    fn node_by_path(store: &Store, project_id: &str, path: &str) -> structure::Node {
        let snapshot = store.load_work_snapshot(project_id).expect("snapshot");
        snapshot
            .nodes
            .iter()
            .find(|node| node.path == path)
            .cloned()
            .unwrap_or_else(|| panic!("missing node {path}"))
    }
}
