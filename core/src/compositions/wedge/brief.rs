#[allow(unused_imports)] // The package-wide glob; every name this file uses is explicit.
use super::*;

use std::path::Path;

use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::domains::structure;
use crate::domains::{canvas, knowledge, polter, work};
use crate::providers::sqlite as storepkg;

use super::path::{clean_path, relative_path};
use super::repos::{project_checkouts, with_default_repo};

/// Renders the brief of a card, a Structure node, or a file. A target that is
/// neither a card nor a node is read as a file path and briefed as the node
/// whose checkout holds it.
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
    let repos = project_checkouts(store, &snapshot.project_id)?;
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
            let Some(inside) = relative_path(&clean_path(&item.root), &clean_path(file)) else {
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
    let rel = clean_path(&rel);
    let mut found = structure::Node::default();
    let mut ok = false;
    for node in with_default_repo(&snapshot.nodes, &repos) {
        // A relative path with no repo: prefix may sit in any repository.
        if node.status == structure::NODE_REMOVED
            || node.folder.is_empty()
            || (!repo.is_empty() && node.repo != repo)
        {
            continue;
        }
        let folder = clean_path(&node.folder).trim_matches('/').to_string();
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::super::init::{InitInput, init_project};
    use super::super::repos::ensure_repository;
    use super::super::work::{ApplyInput, apply_polter};
    use super::*;
    use crate::domains::projects;
    use crate::providers::sqlite::test_support::TempStore;

    fn apply(store: &storepkg::Store, project_id: &str, text: &str) {
        let result = apply_polter(
            store,
            ApplyInput {
                project_id: project_id.to_string(),
                text: text.to_string(),
                auto_start_label: "cli".to_string(),
                ..ApplyInput::default()
            },
        );
        let result = match result {
            Ok(result) => result,
            Err(failure) => panic!("apply: {failure} {:?}", failure.result.diagnostics),
        };
        assert!(
            result.diagnostics.is_empty(),
            "apply: {:?}",
            result.diagnostics
        );
    }

    fn init_with_repo(store: &storepkg::Store, name: &str, root: &str) -> projects::Project {
        let project = init_project(
            store,
            InitInput {
                name: name.to_string(),
                ..InitInput::default()
            },
        )
        .expect("init project");
        ensure_repository(store, &project.id, root).expect("ensure repository");
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
        ensure_repository(store, &project.id, &api).expect("api repository");
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
