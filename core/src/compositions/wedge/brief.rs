#[allow(unused_imports)] // The package-wide glob; every name this file uses is explicit.
use super::*;

use std::path::Path;

use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::domains::structure;
use crate::domains::{canvas, knowledge, polter, work};
use crate::providers::sqlite as storepkg;

pub use helpers::{ApplyInput, ApplyResult};

/// Renders the brief of a card, a Structure node, or a file. A target that is
/// neither a card nor a node is read as a file path and briefed as the node
/// whose checkout holds it.
#[allow(dead_code)] // Called by the work.go query router, ported separately.
pub(super) fn query_brief(
    store: &storepkg::Store,
    snapshot: &work::Snapshot,
    scope: &structure::MultiScope,
    target: &str,
    budget: i32,
) -> anyhow::Result<String> {
    let input = brief_input(store, snapshot, scope)?;
    match polter::brief(&input, target, budget) {
        Ok(text) => Ok(text),
        Err(err @ polter::BriefError::UnknownTarget(_)) => {
            let err: anyhow::Error = err.into();
            let (node, ok) = node_for_path(store, snapshot, target)?;
            if !ok {
                return Err(err);
            }
            let text = polter::brief(&input, &node.path, budget)?;
            Ok(format!(
                "file {} is under {}\n{}",
                target.trim(),
                node.path,
                text
            ))
        }
    }
}

#[allow(dead_code)] // Called by the work.go query router, ported separately.
pub(super) fn query_search(
    store: &storepkg::Store,
    snapshot: &work::Snapshot,
    scope: &structure::MultiScope,
    terms: &str,
    budget: i32,
) -> anyhow::Result<String> {
    let input = brief_input(store, snapshot, scope)?;
    Ok(polter::search(&input, terms, budget))
}

/// Loads the in-scope docs and agent-visible canvases next to the snapshot.
/// Canvas nodes linked to Structure carry the node's title.
pub(super) fn brief_input(
    store: &storepkg::Store,
    snapshot: &work::Snapshot,
    scope: &structure::MultiScope,
) -> anyhow::Result<polter::BriefInput> {
    let resolved = store.load_scopes(&snapshot.project_id, scope)?;
    let items = store.list_knowledge(&snapshot.project_id, knowledge::KIND_DOC)?;
    let scopes = store.scope_map(&snapshot.project_id, storepkg::SCOPE_KNOWLEDGE)?;
    let mut input = polter::BriefInput {
        snapshot: snapshot.clone(),
        ..polter::BriefInput::default()
    };
    for item in items {
        let node_ids = scopes.get(&item.id).cloned().unwrap_or_default();
        if !resolved.matches_any(&node_ids) {
            continue;
        }
        input.docs.push(polter::Doc {
            id: item.id,
            title: item.title,
            doc_type: item.doc_type,
            scope_node_ids: node_ids,
            body: item.body,
        });
    }
    let canvases = store.list_canvases(&snapshot.project_id)?;
    for item in canvases {
        if !item.agent_visible || !resolved.matches(&item.scope_node_id) {
            continue;
        }
        let mut item = item;
        item.graph = canvas::with_structure(&item.graph, &snapshot.nodes);
        input.canvases.push(item);
    }
    Ok(input)
}

/// Finds the Structure node whose checkout holds a file: the node with the
/// longest `path:` prefix in the file's checkout (the project root, or a
/// registered repository). The file is absolute, relative to the project root,
/// or written as `repo:path`. `ok` is false when no node claims it.
pub fn node_for_path(
    store: &storepkg::Store,
    snapshot: &work::Snapshot,
    file: &str,
) -> anyhow::Result<(structure::Node, bool)> {
    let file = file.trim();
    if file.is_empty() {
        return Ok((structure::Node::default(), false));
    }
    let repos = helpers::project_checkouts(store, &snapshot.project_id)?;
    let mut repo = String::new();
    let mut rel = String::new();
    if Path::new(file).is_absolute() {
        // The deepest checkout that holds the file wins: a repo may sit inside
        // another.
        let mut best: isize = -1;
        for item in &repos {
            if item.root.is_empty() {
                continue;
            }
            let Some(inside) = go_rel(&item.root, file) else {
                continue;
            };
            if inside == ".." || inside.starts_with("../") || item.root.len() as isize <= best {
                continue;
            }
            best = item.root.len() as isize;
            repo = item.name.clone();
            rel = inside;
        }
        if best < 0 {
            return Ok((structure::Node::default(), false));
        }
    } else {
        rel = file.to_string();
        if let Some((name, rest)) = file.split_once(':') {
            for item in &repos {
                if item.name == name {
                    repo = name.to_string();
                    rel = rest.to_string();
                }
            }
        }
    }
    let rel = helpers::clean_path(&rel);
    let mut found = structure::Node::default();
    let mut ok = false;
    for node in helpers::with_default_repo(&snapshot.nodes, &repos) {
        // A relative path with no repo: prefix may sit in any repository.
        if node.status == structure::NODE_REMOVED
            || node.folder.is_empty()
            || (!repo.is_empty() && node.repo != repo)
        {
            continue;
        }
        let folder = helpers::clean_path(&node.folder)
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
    Ok((found, ok))
}

/// Go's `filepath.Rel(parent, child)` for the cases a checkout check reads: the
/// relation when child sits under parent, and `..`-prefixed output when it does
/// not.
fn go_rel(parent: &str, child: &str) -> Option<String> {
    let parent = helpers::clean_path(parent);
    let child = helpers::clean_path(child);
    if parent == child {
        return Some(".".to_string());
    }
    let parent_parts: Vec<&str> = parent.split('/').filter(|part| !part.is_empty()).collect();
    let child_parts: Vec<&str> = child.split('/').filter(|part| !part.is_empty()).collect();
    let common = parent_parts
        .iter()
        .zip(child_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();
    if common < parent_parts.len() {
        let mut parts: Vec<String> = Vec::new();
        for _ in common..parent_parts.len() {
            parts.push("..".to_string());
        }
        parts.extend(child_parts[common..].iter().map(|part| part.to_string()));
        if parts.is_empty() {
            return Some(".".to_string());
        }
        return Some(parts.join("/"));
    }
    let rest: Vec<&str> = child_parts[common..].to_vec();
    if rest.is_empty() {
        return Some(".".to_string());
    }
    Some(rest.join("/"))
}

/// Bounds the brief the context hook adds per node.
pub const FILE_CONTEXT_BUDGET: i32 = 400;

/// Returns the brief of the Structure node that holds `file`, for the context
/// hook. A session gets each node's brief once, and again only when it changed;
/// `session_id` "" always gets it. No node holding the file returns empty text.
pub fn file_context(
    store: &storepkg::Store,
    project_id: &str,
    session_id: &str,
    file: &str,
) -> anyhow::Result<(String, String)> {
    let snapshot = store.load_work_snapshot(project_id)?;
    let (node, ok) = node_for_path(store, &snapshot, file)?;
    if !ok {
        return Ok((String::new(), String::new()));
    }
    let input = brief_input(store, &snapshot, &structure::MultiScope::default())?;
    let body = polter::brief(&input, &node.path, FILE_CONTEXT_BUDGET)?;
    if !session_id.is_empty() {
        let sum = Sha256::digest(body.as_bytes());
        let fresh = store.mark_hook_context_seen(
            session_id,
            project_id,
            &node.id,
            &hex::encode(&sum[..8]),
            OffsetDateTime::now_utc(),
        )?;
        if !fresh {
            return Ok((String::new(), node.path.clone()));
        }
    }
    Ok((
        format!(
            "Polterware: {} is under {}. Its brief, sent once per session:\n{}",
            file.trim(),
            node.path,
            body
        ),
        node.path,
    ))
}

/// Private ports of the helpers other files of this package own (`repos.go`,
/// `work.go`, `path.go`, `init.go`, `audited.go`), so this run builds and tests
/// on its own. TODO(dedupe): delete once those files land; each item keeps the
/// Go name and signature.
pub(super) mod helpers {
    use std::collections::HashSet;
    use std::path::Path;
    use std::sync::LazyLock;

    use regex::Regex;
    use time::OffsetDateTime;

    use crate::domains::polter::{self, Diagnostic, GitView, References};
    use crate::domains::{agentrun, knowledge, projects, structure, work};
    use crate::platform::ids;
    use crate::providers::{git, sqlite};

    /// Go's `ErrPolterInvalid`, which `work.go` owns.
    pub const ERR_POLTER_INVALID: &str = "polter text has errors";

    /// Go's `ExpandPath`: folds a leading `~` into the user home directory.
    pub fn expand_path(path: &str) -> String {
        let path = path.trim();
        if path.is_empty() {
            return String::new();
        }
        if path == "~" {
            return std::env::var("HOME").unwrap_or_default();
        }
        let rest = path
            .strip_prefix("~/")
            .or_else(|| path.strip_prefix("~\\"))
            .map(str::to_string);
        if let Some(rest) = rest {
            return match std::env::var("HOME") {
                Ok(home) if !home.is_empty() => {
                    Path::new(&home).join(rest).to_string_lossy().into_owned()
                }
                _ => path.to_string(),
            };
        }
        path.to_string()
    }

    /// Go's `filepath.Clean` for a slash path: folds `//`, `.`, and `..`.
    pub fn clean_path(path: &str) -> String {
        let absolute = path.starts_with('/');
        let mut parts: Vec<&str> = Vec::new();
        for part in path.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    if parts.last().is_some_and(|last| *last != "..") {
                        parts.pop();
                        continue;
                    }
                    if !absolute {
                        parts.push("..");
                    }
                }
                other => parts.push(other),
            }
        }
        let joined = parts.join("/");
        if absolute {
            format!("/{joined}")
        } else if joined.is_empty() {
            ".".to_string()
        } else {
            joined
        }
    }

    /// One repository cloned on this machine.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct GitCheckout {
        pub name: String,
        pub root: String,
    }

    /// The project's repositories that have a clone here.
    pub fn project_checkouts(
        store: &sqlite::Store,
        project_id: &str,
    ) -> anyhow::Result<Vec<GitCheckout>> {
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

    /// The GitHub-level identity of a clone's origin (github.com/owner/repo),
    /// empty when it has no GitHub remote.
    pub fn repository_remote(dir: &str) -> String {
        match git::github_repo(dir) {
            Some(owner_repo) => format!("github.com/{owner_repo}"),
            None => String::new(),
        }
    }

    static REPO_NAME_UNSAFE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"[^a-z0-9]+").expect("the repository name pattern"));

    /// Proposes a repository name: the GitHub repository's, else the folder's,
    /// as a lowercase slug.
    pub fn repo_name_for(remote: &str, dir: &str) -> String {
        let mut base = Path::new(dir)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !remote.is_empty() {
            base = remote.rsplit('/').next().unwrap_or(remote).to_string();
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

    /// Returns the project's repository for a local folder: the one cloned
    /// there, else the one with the same GitHub remote (its clone path becomes
    /// dir), else a new one named after the remote or the folder.
    pub fn ensure_repository(
        store: &sqlite::Store,
        project_id: &str,
        dir: &str,
    ) -> anyhow::Result<projects::Repository> {
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
        let mut name = base.clone();
        let mut i: u32 = 2;
        while existing.iter().any(|repo| repo.name == name) {
            let suffix = format!("-{}", i - 1);
            let stem = base.strip_suffix(&suffix).unwrap_or(&base);
            name = format!("{stem}-{i}");
            i += 1;
        }
        let repo = projects::new_repository(&ids::new_ulid(), project_id, &name, &remote, &abs)?;
        store.save_repository(repo.clone())?;
        Ok(repo)
    }

    /// The absolute form of a folder that must exist.
    pub fn absolute_dir(path: &str) -> anyhow::Result<String> {
        let expanded = expand_path(path);
        if expanded.is_empty() {
            return Err(anyhow::anyhow!("repository folder is empty"));
        }
        let abs = std::path::absolute(&expanded)
            .map_err(|err| anyhow::anyhow!("{err}"))?
            .to_string_lossy()
            .into_owned();
        let abs = clean_path(&abs);
        match std::fs::metadata(&abs) {
            Ok(info) if info.is_dir() => Ok(abs),
            Ok(_) => Err(anyhow::anyhow!(
                "repository folder {abs} is not a directory"
            )),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Err(anyhow::anyhow!("repository folder {abs} does not exist"))
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn same_path(a: &str, b: &str) -> bool {
        clean_path(a) == clean_path(b)
    }

    /// Rewrites each node's `dir: <folder>` line to `repo: <name>`, the
    /// repository cloned there (created when the project has none). A dry run
    /// creates nothing and returns the names it would create.
    pub fn resolve_node_dirs(
        store: &sqlite::Store,
        project_id: &str,
        text: &str,
        dry_run: bool,
    ) -> anyhow::Result<(String, Vec<String>)> {
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
                if let Some(i) = line.find("dir:") {
                    let rest = &line[i + "dir:".len()..];
                    let end = rest.find(" · ").unwrap_or(rest.len());
                    lines[index] = format!("{}repo: {name}{}", &line[..i], &rest[end..]);
                }
            }
        }
        Ok((lines.join("\n"), proposed))
    }

    /// Removes the repositories a write unlinked from its last node, so a
    /// repository lives only while a component uses it.
    pub fn prune_repositories(
        store: &sqlite::Store,
        project_id: &str,
        ops: &[work::Op],
    ) -> anyhow::Result<()> {
        for op in ops {
            if op.entity != work::ENTITY_NODE {
                continue;
            }
            if let Some(serde_json::Value::String(name)) = op.before.get("repo")
                && !name.is_empty()
            {
                store.delete_unused_repository(project_id, name)?;
            }
        }
        Ok(())
    }

    /// Returns nodes where a node naming no repository belongs to the project's
    /// only one, so a one-repository project needs no `repo:` lines.
    pub fn with_default_repo(
        nodes: &[structure::Node],
        checkouts: &[GitCheckout],
    ) -> Vec<structure::Node> {
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

    /// Reads the project's first repository clone (by name). None returns an
    /// empty view; callers still answer.
    pub fn project_git(store: &sqlite::Store, project_id: &str) -> GitView {
        let Ok(checkouts) = project_checkouts(store, project_id) else {
            return GitView::default();
        };
        let Some(first) = checkouts.first() else {
            return GitView::default();
        };
        let obs = git::observe(&first.root);
        GitView {
            available: obs.available,
            branch: obs.branch,
            files: obs.files,
            delivered: obs.delivered,
        }
    }

    /// Loads what reference-level lint and hints check against.
    pub fn polter_references(
        store: &sqlite::Store,
        snapshot: &work::Snapshot,
    ) -> anyhow::Result<References> {
        let decisions = store.list_knowledge(&snapshot.project_id, knowledge::KIND_DECISION)?;
        let mut refs = References {
            cards: snapshot.cards.clone(),
            nodes: snapshot.nodes.clone(),
            repos: snapshot.repos.clone(),
            environments: snapshot.environments.clone(),
            ..References::default()
        };
        for decision in decisions {
            refs.decisions.push(polter::DecisionRef {
                id: decision.id,
                title: decision.title,
                status: decision.status,
            });
        }
        refs.git = Some(project_git(store, &snapshot.project_id));
        Ok(refs)
    }

    /// Keeps lint output, then plan diagnostics that lint did not already
    /// report (the graph check runs in both).
    pub fn merge_diagnostics(lint: Vec<Diagnostic>, plan: Vec<Diagnostic>) -> Vec<Diagnostic> {
        let mut seen: HashSet<(i32, String, String)> = HashSet::new();
        let mut out: Vec<Diagnostic> = Vec::with_capacity(lint.len() + plan.len());
        for diag in lint.into_iter().chain(plan) {
            let key = (diag.line, diag.severity.clone(), diag.message.clone());
            if seen.insert(key) {
                out.push(diag);
            }
        }
        out
    }

    /// The first trimmed value that is not empty.
    pub fn first_non_empty(values: &[&str]) -> String {
        for value in values {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        String::new()
    }

    /// One `polterware apply` / `polter.apply` call.
    /// TODO(dedupe): `work.go` owns `ApplyInput`.
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
        /// `actor_kind` and `tool` default to an agent write through
        /// "polter.apply"; the Desktop passes a human actor and its own tool.
        pub actor_kind: String,
        pub tool: String,
        /// The session agent label that claim, done, and release lines act for.
        pub agent: String,
    }

    /// TODO(dedupe): `work.go` owns `ApplyResult`.
    #[derive(Debug, Clone, Default, PartialEq)]
    pub struct ApplyResult {
        pub project_id: String,
        pub ops: Vec<work::Op>,
        pub diagnostics: Vec<Diagnostic>,
        pub applied: bool,
        pub agent_run_id: String,
        pub batch_id: String,
        /// The resolved change envelope (flags over frontmatter), filled even on
        /// a dry run.
        pub summary: String,
        pub rationale: String,
    }

    /// The justified mutation envelope shared by MCP and Desktop.
    /// TODO(dedupe): `audited.go` owns it.
    #[derive(Debug, Clone, Default, PartialEq)]
    pub struct ChangeEnvelope {
        pub summary: String,
        pub rationale: String,
        pub agent_run_id: String,
        pub project_id: String,
        pub paths: Vec<String>,
        /// Lets an agent write without an open run: the core opens and
        /// completes a run with this label (the CLI uses "cli").
        pub auto_start_label: String,
        /// Links the change step to its `change_ops` rows.
        pub batch_id: String,
    }

    /// What a successful audited mutation answers.
    /// TODO(dedupe): `audited.go` owns it.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct AuditedWriteResult {
        pub agent_run_id: String,
        pub subject_ids: Vec<String>,
    }

    /// Validates the open run (or starts one), runs the mutation, appends a
    /// change step, and completes auto-started runs.
    /// TODO(dedupe): `audited.go` owns it.
    pub fn execute_audited_write<F>(
        store: &sqlite::Store,
        env: &ChangeEnvelope,
        actor_kind: &str,
        tool: &str,
        mutate: F,
    ) -> anyhow::Result<AuditedWriteResult>
    where
        F: FnOnce(&agentrun::Run) -> anyhow::Result<Vec<String>>,
    {
        let kind = agentrun::normalize_actor_kind(actor_kind)?;
        agentrun::validate_change_as(kind, &env.summary, &env.rationale)?;
        if env.project_id.is_empty() {
            return Err(agentrun::Error::EmptyProject.into());
        }

        let now = OffsetDateTime::now_utc();
        let mut run_id = env.agent_run_id.clone();
        let mut auto_complete = false;

        if run_id.is_empty() {
            let mut label = "Desktop".to_string();
            let auto_label = env.auto_start_label.trim();
            if !auto_label.is_empty() {
                label = auto_label.to_string();
            } else if kind != agentrun::ACTOR_KIND_HUMAN {
                return Err(agentrun::Error::MissingAgentRunId.into());
            }
            let run =
                agentrun::start_with_actor(&env.project_id, &env.summary, &label, kind, Some(now))?;
            store.create_agent_run(run.clone())?;
            run_id = run.id;
            auto_complete = true;
        }

        let run = store.get_agent_run(&run_id)?;
        agentrun::can_append(&run)?;
        agentrun::match_project(&run, &env.project_id)?;
        if !run.actor_kind.is_empty() && run.actor_kind != kind {
            return Err(anyhow::anyhow!(
                "invalid actor kind: run is {}, write is {}",
                run.actor_kind,
                kind
            ));
        }

        let subject_ids = mutate(&run)?;

        store
            .append_agent_run_step_if_open(&run_id, |open| {
                agentrun::match_project(open, &env.project_id)?;
                agentrun::new_batch_change_step(
                    open.id.as_str(),
                    &env.summary,
                    &env.rationale,
                    tool,
                    &env.batch_id,
                    &subject_ids,
                    &env.paths,
                    Some(now),
                )
            })
            .map_err(|err| anyhow::anyhow!("mutation saved but change step failed: {err}"))?;

        if auto_complete {
            let done = agentrun::complete(run, Some(now))
                .map_err(|err| anyhow::anyhow!("mutation saved but complete failed: {err}"))?;
            store
                .update_agent_run(done)
                .map_err(|err| anyhow::anyhow!("mutation saved but complete failed: {err}"))?;
        }

        Ok(AuditedWriteResult {
            agent_run_id: run_id,
            subject_ids,
        })
    }

    /// Decodes, lints, plans, and (unless a dry run) applies Polter text as one
    /// audited write. On [`ERR_POLTER_INVALID`] the result carries the
    /// diagnostics.
    /// TODO(dedupe): `work.go` owns `ApplyPolter`.
    pub fn apply_polter(
        store: &sqlite::Store,
        input: &ApplyInput,
    ) -> (ApplyResult, anyhow::Result<()>) {
        let mut result = ApplyResult {
            project_id: input.project_id.clone(),
            ..ApplyResult::default()
        };
        let outcome = apply_polter_inner(store, input, &mut result);
        (result, outcome)
    }

    fn apply_polter_inner(
        store: &sqlite::Store,
        input: &ApplyInput,
        result: &mut ApplyResult,
    ) -> anyhow::Result<()> {
        let (text, proposed) =
            resolve_node_dirs(store, &input.project_id, &input.text, input.dry_run)?;
        let mut snapshot = store.load_work_snapshot(&input.project_id)?;
        snapshot.repos.extend(proposed);
        let refs = polter_references(store, &snapshot)?;
        let (doc, lint_diags) = polter::lint(&text, Some(&refs));
        let (mut ops, plan_diags) = polter::plan_as(&doc, &snapshot, &input.agent);
        result.diagnostics = merge_diagnostics(lint_diags, plan_diags);
        if polter::has_errors(&result.diagnostics) {
            return Err(anyhow::anyhow!(ERR_POLTER_INVALID));
        }
        result.ops = ops.clone();
        result.summary = first_non_empty(&[&input.summary, &doc.summary]);
        result.rationale = first_non_empty(&[&input.rationale, &doc.rationale]);
        if input.dry_run || ops.is_empty() {
            return Ok(());
        }

        let summary = result.summary.clone();
        let rationale = result.rationale.clone();
        let actor_kind = first_non_empty(&[&input.actor_kind, agentrun::ACTOR_KIND_AGENT]);
        if let Err(err) = agentrun::validate_change_as(&actor_kind, &summary, &rationale) {
            return Err(anyhow::anyhow!(
                "{err}: add them to the frontmatter or pass --summary and --rationale"
            ));
        }
        let tool = first_non_empty(&[&input.tool, "polter.apply"]);
        result.batch_id = ids::new_ulid();
        let envelope = ChangeEnvelope {
            summary,
            rationale,
            agent_run_id: input.agent_run_id.clone(),
            project_id: input.project_id.clone(),
            paths: Vec::new(),
            auto_start_label: input.auto_start_label.clone(),
            batch_id: result.batch_id.clone(),
        };
        let agent = input.agent.clone();
        let project_id = input.project_id.clone();
        let batch_id = result.batch_id.clone();
        let written = execute_audited_write(store, &envelope, &actor_kind, &tool, |run| {
            for op in ops.iter_mut() {
                if op.entity == work::ENTITY_FACT {
                    let actor = if agent.is_empty() {
                        actor_kind.clone()
                    } else {
                        agent.clone()
                    };
                    op.fields
                        .insert("actor".to_string(), serde_json::Value::String(actor));
                }
            }
            let card_ids = store.apply_work_ops(&project_id, &run.id, &batch_id, &ops)?;
            Ok(card_ids)
        })?;
        result.applied = true;
        result.agent_run_id = written.agent_run_id;
        prune_repositories(store, &input.project_id, &ops)
    }

    /// One `polterware init` call.
    /// TODO(dedupe): `init.go` owns `InitInput` and `InitProject`.
    #[cfg(test)]
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct InitInput {
        pub id: String,
        pub name: String,
        /// Given a folder, the project starts with one component linked to the
        /// repository cloned there.
        pub dir: String,
    }

    #[cfg(test)]
    pub fn init_project(
        store: &sqlite::Store,
        input: &InitInput,
    ) -> anyhow::Result<projects::Project> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(anyhow::anyhow!("name is required"));
        }
        let mut dir = input.dir.trim().to_string();
        if !dir.is_empty() {
            dir = absolute_dir(&dir)?;
        }
        let mut id = input.id.trim().to_string();
        if id.is_empty() {
            id = new_project_id()?;
        }
        let project = projects::new_project(&id, name)?;
        store.save_project(project.clone())?;
        if store.get_project_settings(&project.id).is_err() {
            store.save_project_settings(projects::Settings {
                project_id: project.id.clone(),
                ..projects::Settings::default()
            })?;
        }
        let envs = store.list_environments(&project.id)?;
        if envs.is_empty() {
            let env_id = format!("{}-local", project.id);
            let mut env = projects::new_environment(&env_id, &project.id, "local", "local")?;
            env.is_default = true;
            env.protection = "relaxed".to_string();
            env.description = "local environment".to_string();
            store.save_environment(env)?;
            store.save_project_settings(projects::Settings {
                project_id: project.id.clone(),
                default_environment_id: env_id,
                ..projects::Settings::default()
            })?;
        }
        if !dir.is_empty() {
            link_initial_component(store, &project.id, &dir)?;
        }
        Ok(project)
    }

    /// Gives a new project its first component, named after the folder and
    /// linked to the repository cloned there.
    #[cfg(test)]
    fn link_initial_component(
        store: &sqlite::Store,
        project_id: &str,
        dir: &str,
    ) -> anyhow::Result<()> {
        let snapshot = store.load_work_snapshot(project_id)?;
        if !snapshot.nodes.is_empty() {
            return Ok(());
        }
        let slug = repo_name_for("", dir);
        let title = Path::new(dir)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let text =
            format!("---\npolter: 1\n---\n\n# {slug}: {title}\nlevel: component\ndir: {dir}\n");
        let (_, outcome) = apply_polter(
            store,
            &ApplyInput {
                project_id: project_id.to_string(),
                text,
                actor_kind: agentrun::ACTOR_KIND_HUMAN.to_string(),
                tool: "init".to_string(),
                summary: "Link the project folder".to_string(),
                rationale: "The project was created from this folder".to_string(),
                ..ApplyInput::default()
            },
        );
        outcome
    }

    #[cfg(test)]
    fn new_project_id() -> anyhow::Result<String> {
        use rand::RngCore;
        let mut buf = [0u8; 4];
        rand::rng().fill_bytes(&mut buf);
        Ok(format!("proj-{}", hex::encode(buf)))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::helpers::{self, ApplyInput, InitInput};
    use super::*;
    use crate::domains::projects;
    use crate::providers::sqlite::test_support::TempStore;

    fn apply(store: &storepkg::Store, project_id: &str, text: &str) {
        let (result, outcome) = helpers::apply_polter(
            store,
            &ApplyInput {
                project_id: project_id.to_string(),
                text: text.to_string(),
                auto_start_label: "cli".to_string(),
                ..ApplyInput::default()
            },
        );
        assert!(
            outcome.is_ok() && result.diagnostics.is_empty(),
            "apply: {:?} {:?}",
            outcome.err(),
            result.diagnostics
        );
    }

    fn init_with_repo(store: &storepkg::Store, name: &str, root: &str) -> projects::Project {
        let project = helpers::init_project(
            store,
            &InitInput {
                name: name.to_string(),
                ..InitInput::default()
            },
        )
        .expect("init project");
        helpers::ensure_repository(store, &project.id, root).expect("ensure repository");
        project
    }

    fn query(
        store: &storepkg::Store,
        project_id: &str,
        text: &str,
        scope: &structure::MultiScope,
    ) -> anyhow::Result<String> {
        let snapshot = store.load_work_snapshot(project_id)?;
        let query = polter::parse_query(text).map_err(|err| anyhow::anyhow!("{err}"))?;
        match query.kind.as_str() {
            polter::QUERY_BRIEF => query_brief(store, &snapshot, scope, &query.slug, query.budget),
            polter::QUERY_SEARCH => {
                query_search(store, &snapshot, scope, &query.slug, query.budget)
            }
            other => Err(anyhow::anyhow!("unhandled query {other}")),
        }
    }

    #[test]
    fn brief_resolves_files_to_the_node_that_holds_them() {
        let temp = TempStore::named("work.db");
        let store = &temp.store;
        let root_dir = tempfile::tempdir().expect("root");
        let root = root_dir.path().to_string_lossy().into_owned();
        let api_dir = tempfile::tempdir().expect("api");
        let api = api_dir.path().join("api");
        std::fs::create_dir_all(&api).expect("api dir");
        let api = api.to_string_lossy().into_owned();
        let project = init_with_repo(store, "Demo", &root);
        helpers::ensure_repository(store, &project.id, &api).expect("api repository");
        apply(
            store,
            &project.id,
            &format!(
                r#"---
polter: 1
summary: Map the checkouts
rationale: Briefs find a file's node by its checkout
---

# app: App
level: domain

# app/core: Core
level: component · under: app · dir: {root} · path: src

# app/web: Web
level: component · under: app · dir: {root} · path: src/web

# app/auth: Auth
level: component · under: app · repo: api · path: internal/auth

## login: Login
stage: build · under: app/auth
"#
            ),
        );
        let scope = structure::MultiScope::default();
        let q = |text: &str| query(store, &project.id, text, &scope).expect(text);

        let absolute_web = format!("{root}/src/web/index.ts");
        let absolute_auth = format!("{api}/internal/auth/x.go");
        for (file, node) in [
            ("src/web/app.tsx", "app/web"),
            ("src/lib/util.ts", "app/core"),
            (absolute_web.as_str(), "app/web"),
            (absolute_auth.as_str(), "app/auth"),
            ("api:internal/auth/session.go", "app/auth"),
        ] {
            let text = q(&format!("brief {file}"));
            assert!(
                text.starts_with(&format!("file {file} is under {node}\n# {node}:")),
                "brief {file}:\n{text}"
            );
        }
        let text = q("brief login");
        assert!(
            text.starts_with(
                "## login: Login\nstage: build · under: app/auth\nrepo: api · path: internal/auth\n"
            ),
            "card brief:\n{text}"
        );
        let err = query(store, &project.id, "brief docs/readme.md", &scope)
            .expect_err("a file no node holds");
        assert!(err.to_string().contains("unknown brief target"), "{err}");
        let text = q("search auth");
        assert!(
            text.contains("node app/auth  component  \"Auth\""),
            "search:\n{text}"
        );
    }

    #[test]
    fn file_context_sends_each_node_brief_once_per_session() {
        let temp = TempStore::named("file-context.db");
        let store = &temp.store;
        let root_dir = tempfile::tempdir().expect("root");
        let root = root_dir.path().to_string_lossy().into_owned();
        let project = init_with_repo(store, "Demo", &root);
        let apply_text = |text: &str| {
            apply(
                store,
                &project.id,
                &format!(
                    "---\npolter: 1\nsummary: Map the app\nrationale: Hooks brief by file\n---\n\n{text}"
                ),
            );
        };
        apply_text(
            "# app: App\nlevel: domain\n\n# app/core: Core\nlevel: component · under: app · path: src\n\n# app/web: Web\nlevel: component · under: app · path: src/web\n\n! No inline styles\n",
        );
        let file = format!("{root}/src/web/page.tsx");

        let (text, node) = file_context(store, &project.id, "s1", &file).expect("first read");
        assert_eq!(node, "app/web");
        assert!(
            text.starts_with(&format!("Polterware: {file} is under app/web.")),
            "{text:?}"
        );
        assert!(text.contains("Rules:\n- No inline styles\n"), "{text:?}");
        let (again, _) = file_context(
            store,
            &project.id,
            "s1",
            &format!("{root}/src/web/other.tsx"),
        )
        .expect("second read");
        assert_eq!(again, "", "a second file under the same node sends nothing");
        let (other, _) = file_context(store, &project.id, "s2", &file).expect("other session");
        assert!(!other.is_empty(), "another session gets its own brief");
        let (parent, node) = file_context(store, &project.id, "s1", &format!("{root}/src/main.ts"))
            .expect("parent read");
        assert_eq!(node, "app/core");
        assert!(!parent.is_empty());
        apply_text(
            "# app/web: Web\nlevel: component · under: app · path: src/web\n\n! No inline styles\n! Server components by default\n",
        );
        let (changed, _) = file_context(store, &project.id, "s1", &file).expect("changed read");
        assert!(
            changed.contains("Server components by default"),
            "a changed brief is sent again: {changed:?}"
        );
        let (none, node) = file_context(store, &project.id, "s1", "/elsewhere/readme.md")
            .expect("a file no node holds");
        assert_eq!(none, "");
        assert_eq!(node, "");
        let (first, _) = file_context(store, &project.id, "", &file).expect("no session");
        assert!(
            !first.is_empty(),
            "without a session id the brief is always sent"
        );
    }
}
