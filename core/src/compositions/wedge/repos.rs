//! Repositories, checkouts, and which one a run works in.
//!
//! Repositories belong to Structure components by name: the ones the project
//! registers, which clone on this machine, and which one a run works in.

use std::sync::LazyLock;

use regex::Regex;
use serde_json::Value;

use crate::domains::{polter, projects, structure, work};
use crate::platform::ids;
use crate::providers::{git, sqlite};

#[allow(unused_imports)]
use super::*;

use super::path::{absolute_path, clean_path, expand_path, path_base};

/// The errors `repos.go` returns, one per Go `fmt.Errorf` site plus the store.
#[derive(Debug, thiserror::Error)]
pub enum ReposError {
    #[error(transparent)]
    Store(#[from] sqlite::Error),
    /// A run or read needs a repository and the project has several (or none)
    /// with nothing naming one.
    #[error("pick a repository: the project has none or several, and nothing names one")]
    PickRepository,
    #[error("{0}")]
    Message(String),
}

impl ReposError {
    fn message(text: impl Into<String>) -> ReposError {
        ReposError::Message(text.into())
    }
}

pub type ReposResult<T> = std::result::Result<T, ReposError>;

/// One repository cloned on this machine.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitCheckout {
    pub name: String,
    pub root: String,
}

/// Lists the project's repositories that have a clone here.
pub fn project_checkouts(store: &sqlite::Store, project_id: &str) -> ReposResult<Vec<GitCheckout>> {
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

/// Returns the clone of the named repository, or of the only one when `name`
/// is empty; empty when there is none to pick.
pub fn checkout_dir(checkouts: &[GitCheckout], name: &str) -> String {
    if name.is_empty() && checkouts.len() == 1 {
        return checkouts[0].root.clone();
    }
    for checkout in checkouts {
        if checkout.name == name && !name.is_empty() {
            return checkout.root.clone();
        }
    }
    String::new()
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

/// Proposes a repository name: the GitHub repository's, else the folder's, as
/// a lowercase slug.
pub(super) fn repo_name_for(remote: &str, dir: &str) -> String {
    let mut base = path_base(dir);
    if !remote.is_empty() {
        let start = remote.rfind('/').map_or(0, |index| index + 1);
        base = remote[start..].to_string();
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
pub fn ensure_repository(
    store: &sqlite::Store,
    project_id: &str,
    dir: &str,
) -> ReposResult<projects::Repository> {
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
    let mut name = repo_name_for(&remote, &abs);
    let taken: Vec<&str> = existing.iter().map(|repo| repo.name.as_str()).collect();
    let mut index = 2;
    while taken.contains(&name.as_str()) {
        let base = repo_name_for(&remote, &abs);
        let suffix = format!("-{}", index - 1);
        let stem = base.strip_suffix(&suffix).unwrap_or(&base);
        name = format!("{stem}-{index}");
        index += 1;
    }
    let repo = projects::new_repository(&ids::new_ulid(), project_id, &name, &remote, &abs)
        .map_err(|err| ReposError::message(err.to_string()))?;
    store.save_repository(repo.clone())?;
    Ok(repo)
}

pub(super) fn absolute_dir(path: &str) -> ReposResult<String> {
    let expanded = expand_path(path);
    if expanded.is_empty() {
        return Err(ReposError::message("repository folder is empty"));
    }
    let abs = absolute_path(&expanded);
    let info = match std::fs::metadata(&abs) {
        Ok(info) => info,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(ReposError::message(format!(
                "repository folder {abs} does not exist"
            )));
        }
        Err(err) => return Err(ReposError::message(err.to_string())),
    };
    if !info.is_dir() {
        return Err(ReposError::message(format!(
            "repository folder {abs} is not a directory"
        )));
    }
    Ok(abs)
}

pub(super) fn same_path(a: &str, b: &str) -> bool {
    clean_path(a) == clean_path(b)
}

/// Rewrites each node's `dir: <folder>` line to `repo: <name>`, the repository
/// cloned there (created when the project has none). A dry run creates nothing
/// and returns the names it would create.
pub(super) fn resolve_node_dirs(
    store: &sqlite::Store,
    project_id: &str,
    text: &str,
    dry_run: bool,
) -> ReposResult<(String, Vec<String>)> {
    let (doc, _) = polter::decode(text);
    let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
    let mut proposed = Vec::new();
    for node in &doc.nodes {
        for entry in &node.meta {
            if entry.key != "dir" || entry.line < 1 || entry.line as usize > lines.len() {
                continue;
            }
            let name = if dry_run {
                let abs = absolute_dir(&entry.value)?;
                let mut name = repo_name_for(&repository_remote(&abs), &abs);
                let repos = store.list_repositories(project_id)?;
                for repo in repos {
                    if same_path(&expand_path(&repo.local_path), &abs) {
                        name = repo.name;
                    }
                }
                proposed.push(name.clone());
                name
            } else {
                ensure_repository(store, project_id, &entry.value)?.name
            };
            let position = entry.line as usize - 1;
            let line = lines[position].clone();
            if let Some(start) = line.find("dir:") {
                let rest = &line[start + "dir:".len()..];
                let end = rest.find(" · ").unwrap_or(rest.len());
                lines[position] = format!("{}repo: {}{}", &line[..start], name, &rest[end..]);
            }
        }
    }
    Ok((lines.join("\n"), proposed))
}

/// Removes the repositories a write unlinked from its last node, so a
/// repository lives only while a component uses it.
pub(super) fn prune_repositories(
    store: &sqlite::Store,
    project_id: &str,
    ops: &[work::Op],
) -> ReposResult<()> {
    for op in ops {
        if op.entity != work::ENTITY_NODE {
            continue;
        }
        if let Some(Value::String(name)) = op.before.get("repo")
            && !name.is_empty()
        {
            store.delete_unused_repository(project_id, name)?;
        }
    }
    Ok(())
}

/// The error a run or read returns when the project has several repositories
/// (or none) with nothing naming one.
pub const ERR_PICK_REPOSITORY: &str =
    "pick a repository: the project has none or several, and nothing names one";

/// Picks the clone a dispatch run works in: the named repository, else the
/// card's component repository, else the project's only repository.
pub fn run_dir(
    store: &sqlite::Store,
    project_id: &str,
    repo: &str,
    card_slug: &str,
) -> ReposResult<String> {
    let checkouts = project_checkouts(store, project_id)?;
    let repo = repo.trim();
    if !repo.is_empty() {
        let dir = checkout_dir(&checkouts, repo);
        if !dir.is_empty() {
            return Ok(dir);
        }
        return Err(ReposError::message(format!(
            "repository {} has no clone on this machine",
            polter::go_quote(repo)
        )));
    }
    let card_slug = card_slug.trim();
    if !card_slug.is_empty() {
        let snapshot = store.load_work_snapshot(project_id)?;
        if let Some(card) = snapshot.card(card_slug) {
            for node in &snapshot.nodes {
                if node.path == card.under && !node.repo.is_empty() {
                    let dir = checkout_dir(&checkouts, &node.repo);
                    if !dir.is_empty() {
                        return Ok(dir);
                    }
                }
            }
        }
    }
    let dir = checkout_dir(&checkouts, "");
    if !dir.is_empty() {
        return Ok(dir);
    }
    Err(ReposError::PickRepository)
}

/// The clone of the project's first repository by name, empty when none is
/// cloned here.
pub fn first_checkout(store: &sqlite::Store, project_id: &str) -> String {
    match project_checkouts(store, project_id) {
        Ok(checkouts) if !checkouts.is_empty() => checkouts[0].root.clone(),
        _ => String::new(),
    }
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

/// The card a dispatch run names, as its classifier reads it, or empty when
/// the slug names no card.
pub fn run_card_text(store: &sqlite::Store, project_id: &str, card_slug: &str) -> String {
    let card_slug = card_slug.trim();
    if project_id.is_empty() || card_slug.is_empty() {
        return String::new();
    }
    let Ok(snapshot) = store.load_work_snapshot(project_id) else {
        return String::new();
    };
    match snapshot.card(card_slug) {
        Some(card) => polter::task_text(card),
        None => String::new(),
    }
}

/// The Structure component a dispatch run works in: the card's component, else
/// the node whose checkout folder holds `dir`, else the only component of the
/// repository cloned at `dir`. The flag is false when none applies.
pub fn run_node(
    store: &sqlite::Store,
    project_id: &str,
    card_slug: &str,
    dir: &str,
) -> ReposResult<(structure::Node, bool)> {
    if project_id.is_empty() {
        return Ok((structure::Node::default(), false));
    }
    let snapshot = store.load_work_snapshot(project_id)?;
    if let Some(card) = snapshot.card(card_slug.trim())
        && !card.under.is_empty()
    {
        for node in &snapshot.nodes {
            if node.path == card.under && node.status != structure::NODE_REMOVED {
                return Ok((node.clone(), true));
            }
        }
    }
    if dir.is_empty() {
        return Ok((structure::Node::default(), false));
    }
    let (node, ok) = super::brief::node_for_path(store, &snapshot, dir)
        .map_err(|err| ReposError::message(err.to_string()))?;
    if ok {
        return Ok((node, true));
    }
    let checkouts = project_checkouts(store, project_id)?;
    let mut repo = String::new();
    for checkout in &checkouts {
        if clean_path(&checkout.root) == clean_path(dir) {
            repo = checkout.name.clone();
        }
    }
    let mut found = structure::Node::default();
    let mut count = 0;
    for node in with_default_repo(&snapshot.nodes, &checkouts) {
        if node.level == structure::LEVEL_COMPONENT
            && node.status != structure::NODE_REMOVED
            && !node.repo.is_empty()
            && node.repo == repo
        {
            found = node;
            count += 1;
        }
    }
    Ok((found, count == 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_names_are_short_slugs() {
        assert_eq!(repo_name_for("", "/code/Polterware_Web"), "polterware-web");
        assert_eq!(
            repo_name_for("github.com/polterware/dispatch", "/code/x"),
            "dispatch"
        );
        assert_eq!(repo_name_for("", "/"), "repo");
        assert_eq!(
            repo_name_for("", "/code/a-very-long-repository-name-for-polterware"),
            "a-very-long-repository-name-for"
        );
    }

    #[test]
    fn checkout_picks_the_only_one_without_a_name() {
        let checkouts = vec![GitCheckout {
            name: "web".to_string(),
            root: "/code/web".to_string(),
        }];
        assert_eq!(checkout_dir(&checkouts, ""), "/code/web");
        assert_eq!(checkout_dir(&checkouts, "web"), "/code/web");
        assert_eq!(checkout_dir(&checkouts, "api"), "");
        assert_eq!(checkout_dir(&[], ""), "");
    }

    #[test]
    fn default_repository_applies_only_with_one_checkout() {
        let nodes = vec![structure::Node {
            path: "web".to_string(),
            ..structure::Node::default()
        }];
        let one = vec![GitCheckout {
            name: "web".to_string(),
            root: "/code/web".to_string(),
        }];
        let given = with_default_repo(&nodes, &one);
        assert_eq!(given[0].repo, "web");
        assert!(with_default_repo(&nodes, &[])[0].repo.is_empty());
        assert_eq!(nodes[0].repo, "", "the input is untouched");
    }

    #[test]
    fn run_node_finds_the_component_by_card_folder_or_repository() {
        use super::super::init::{InitInput, init_project};
        use super::super::work::{ApplyInput, apply_polter};
        use crate::providers::sqlite::test_support::TempStore;

        let holder = TempStore::new();
        let store = &holder.store;
        let base = tempfile::tempdir().expect("root");
        let root = base.path().join("demo");
        std::fs::create_dir_all(root.join("web").join("src")).expect("layout");
        let root = root.to_string_lossy().into_owned();
        let project = init_project(
            store,
            InitInput {
                name: "Demo".to_string(),
                ..InitInput::default()
            },
        )
        .expect("project");
        ensure_repository(store, &project.id, &root).expect("repository");
        let text = r#"---
polter: 1
summary: Map the parts
rationale: Runs price models by the component they work in
---

# api: API
level: component · dir: {root} · path: api · kind: service · tech: Go

# web: Web
level: component · dir: {root} · path: web · kind: frontend · tech: TypeScript, Solid

## login: Login
stage: build · under: api
"#
        .replace("{root}", &root);
        let result = apply_polter(
            store,
            ApplyInput {
                project_id: project.id.clone(),
                auto_start_label: "cli".to_string(),
                text,
                ..ApplyInput::default()
            },
        )
        .expect("apply");
        assert!(
            result.diagnostics.is_empty(),
            "apply: {:?}",
            result.diagnostics
        );
        for (card, dir, want) in [
            ("login", format!("{root}/web"), "api"),
            ("", format!("{root}/web/src"), "web"),
            ("missing", format!("{root}/web"), "web"),
        ] {
            let (node, ok) = run_node(store, &project.id, card, &dir).expect("node");
            assert!(
                ok && node.path == want,
                "card {card:?} dir {dir:?}: {} {ok}, want {want}",
                node.path
            );
        }
        let (_, ok) = run_node(store, &project.id, "", &root).expect("node");
        assert!(!ok, "a root shared by two components names neither");
    }
}
