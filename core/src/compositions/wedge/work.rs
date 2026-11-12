//! The Work operations over Polter text: apply, query, and the Git view.

#[allow(unused_imports)]
use super::*;

use std::collections::HashSet;

use time::OffsetDateTime;

use crate::domains::agentrun;
use crate::domains::inbox;
use crate::domains::polter;
use crate::domains::projects;
use crate::domains::structure;
use crate::domains::work;
use crate::platform::ids;
use crate::providers::git as gitpkg;
use crate::providers::sqlite::{self, Store};

use super::path::{absolute_path, clean_path, relative_path};
use super::repos::{project_checkouts, prune_repositories, resolve_node_dirs};

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
    Repos(#[from] super::repos::ReposError),
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
    let dir = absolute_path(dir);
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
    let mut refs = polter::References {
        cards: snapshot.cards.clone(),
        nodes: snapshot.nodes.clone(),
        repos: snapshot.repos.clone(),
        environments: snapshot.environments.clone(),
        ..polter::References::default()
    };
    for rule in &snapshot.rules {
        refs.decisions.push(polter::DecisionRef {
            id: rule.id.clone(),
            title: rule.statement.clone(),
            status: rule.status.clone(),
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
        .map_err(|error| fail(&result, error.into()))?;
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
            let subjects = store.apply_work_ops(&project_id, &run.id, &batch_id, &write_ops)?;
            Ok((subjects, write_ops))
        },
    )
    .map_err(|error| fail(&result, error.into()))?;
    result.applied = true;
    result.agent_run_id = written.agent_run_id;
    if let Err(error) = prune_repositories(store, &in_.project_id, &result.ops) {
        return Err(fail(&result, error.into()));
    }
    Ok(result)
}

/// Answers a v1 query ("cards", "card <slug>", "status", "inbox",
/// "handoff <slug>", "oracle", "structure", "rules [status]",
/// "feature <slug>", "verify <target>", "canvas", "canvas <name|id>") in text.
pub fn query_polter(store: &Store, project_id: &str, text: &str) -> Result<String> {
    query_polter_scoped(store, project_id, text, &structure::MultiScope::default())
}

/// Answers a query over the cards, Rules and Inbox items inside scope. An
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
        polter::QUERY_BRIEF => {
            query_brief(store, &snapshot, scope, &query.slug, query.budget).map_err(message_error)
        }
        polter::QUERY_SEARCH => {
            query_search(store, &snapshot, scope, &query.slug, query.budget).map_err(message_error)
        }
        polter::QUERY_SHOW => query_show(store, &snapshot, &query.slug).map_err(message_error),
        polter::QUERY_COMMITS => {
            query_commits(store, project_id, &query.slug).map_err(message_error)
        }
        polter::QUERY_CONFLICTS => Ok(polter::encode_conflicts(&snapshot)),
        polter::QUERY_DEPLOYS => {
            deploys_text(store, project_id, &query.slug, OffsetDateTime::now_utc())
                .map_err(message_error)
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
        polter::QUERY_RULES => Ok(polter::encode_rules(&snapshot.rules, &query.slug)),
        polter::QUERY_FEATURE | polter::QUERY_VERIFY => {
            query_feature(store, &snapshot, &query).map_err(message_error)
        }
        polter::QUERY_CANVAS => {
            query_canvas(store, project_id, &query.slug, scope).map_err(message_error)
        }
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
    let (card, slugs, unresolved) = select_work_card(snapshot, "", &view.branch);
    if unresolved.is_some() {
        return polter::encode_status_unresolved(&view.branch, &slugs)
            + &polter::encode_status_claims("", snapshot)
            + &inbox_lines;
    }
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

/// Folds a sibling read's error into this file's error, keeping its text.
fn message_error(err: impl std::fmt::Display) -> Error {
    Error::Message(err.to_string())
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
pub(super) fn first_non_empty(values: &[&str]) -> String {
    for value in values {
        if !value.trim().is_empty() {
            return value.trim().to_string();
        }
    }
    String::new()
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
    pub(crate) fn init_test_project(store: &Store, name: &str) -> projects::Project {
        super::init::init_project(
            store,
            super::init::InitInput {
                name: name.to_string(),
                ..super::init::InitInput::default()
            },
        )
        .expect("project")
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
    fn query_rules_lists_by_status_and_shows_one() {
        let TempStore { store, .. } = TempStore::new();
        let project = init_test_project(&store, "Demo");
        let text = concat!(
            "---\npolter: 1\nsummary: Add the Rules\nrationale: The law lives in Rules\n---\n\n",
            "#### rule no-network: Never call the network.\n",
            "why: Offline installs must work.\n",
            "\n",
            "#### rule next-law: Read the local store.\n",
            "status: proposed\n",
        );
        let result = apply(&store, &project.id, text);
        assert!(result.applied, "apply: {result:?}");

        let all = query_polter(&store, &project.id, "rules").expect("rules");
        assert!(
            all.lines()
                .next()
                .is_some_and(|line| line.starts_with("rules: proposed")),
            "header: {all}"
        );
        assert!(
            all.contains("- no-network · active · Never call the network. · whole project"),
            "{all}"
        );

        let proposed = query_polter(&store, &project.id, "rules proposed").expect("proposed");
        assert!(proposed.contains("next-law"), "{proposed}");
        assert!(!proposed.contains("no-network ·"), "{proposed}");

        let one = query_polter(&store, &project.id, "show no-network").expect("show a rule");
        assert!(
            one.contains("rule no-network [active] Never call the network."),
            "{one}"
        );
        assert!(one.contains("Offline installs must work."), "{one}");
    }

    #[test]
    fn a_rule_write_names_the_rule_in_its_history() {
        let TempStore { store, .. } = TempStore::new();
        let project = init_test_project(&store, "Demo");
        let created = apply(
            &store,
            &project.id,
            concat!(
                "---\npolter: 1\nsummary: Add the no-network Rule\n",
                "rationale: Offline installs must work\n---\n\n",
                "#### rule no-network: Never call the network.\n",
            ),
        );
        let rule_id = created.ops[0].entity_id.clone();
        assert_eq!(created.ops[0].entity, work::ENTITY_RULE, "ops: {created:?}");
        let history = store
            .list_subject_changes(&project.id, "", &rule_id, 10)
            .expect("rule history");
        assert_eq!(history.len(), 1, "history: {history:?}");
        assert_eq!(history[0].summary, "Add the no-network Rule");

        // An edit names the same Rule, and leaves another id alone.
        apply(
            &store,
            &project.id,
            concat!(
                "---\npolter: 1\nsummary: Tighten the no-network Rule\n",
                "rationale: The law reads clearer\n---\n\n",
                "#### rule no-network: Never reach the network.\n",
                "why: Offline installs must work.\n",
                "status: under_review\n",
            ),
        );
        let history = store
            .list_subject_changes(&project.id, "", &rule_id, 10)
            .expect("rule history");
        let summaries: Vec<&str> = history.iter().map(|row| row.summary.as_str()).collect();
        assert_eq!(
            summaries,
            vec!["Tighten the no-network Rule", "Add the no-network Rule"],
            "history: {history:?}"
        );
        let other = store
            .list_subject_changes(&project.id, "", "some-other-id", 10)
            .expect("other subject");
        assert!(other.is_empty(), "other subject: {other:?}");
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
