#[allow(unused_imports)] // The package-wide glob; every name this file uses is explicit.
use super::*;

use crate::domains::polter;
use crate::domains::work;
use crate::providers::sqlite as storepkg;

use super::work::{ApplyInput, ApplyResult, apply_polter, first_non_empty, project_git};

/// One `polterware handoff` call. The core fills branch and touched files from
/// Git, then writes Polter text through the apply path.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HandoffInput {
    pub project_id: String,
    pub card_slug: String,
    pub note: String,
    pub agent: String,
    pub summary: String,
    pub rationale: String,
    /// The audited-write envelope copied onto the composed text.
    pub apply: ApplyInput,
}

/// Picks the card a handoff or status command is about. `slug` wins. Otherwise
/// the only active card that already has a handoff on `branch`, otherwise the
/// only active card. The returned slugs list the active cards when the choice
/// is ambiguous, and the error is set when it is. Go's `(card, slugs, error)`
/// shape, which the status read shares.
#[allow(clippy::type_complexity)]
pub fn select_work_card(
    snapshot: &work::Snapshot,
    slug: &str,
    branch: &str,
) -> (work::Card, Vec<String>, Option<anyhow::Error>) {
    let active: Vec<&work::Card> = snapshot
        .cards
        .iter()
        .filter(|card| card.status == work::CARD_ACTIVE)
        .collect();
    let slugs: Vec<String> = active.iter().map(|card| card.slug.clone()).collect();
    let slug = slug.trim();
    if !slug.is_empty() {
        return match snapshot.card(slug) {
            Some(card) if card.status == work::CARD_ACTIVE => (card.clone(), Vec::new(), None),
            _ => (
                work::Card::default(),
                slugs,
                Some(anyhow::anyhow!(
                    "unknown card {}: list cards with polterware query cards",
                    polter::go_quote(slug)
                )),
            ),
        };
    }
    if !branch.is_empty() && branch != "HEAD" {
        let on_branch: Vec<&work::Card> = active
            .iter()
            .filter(|card| card.latest_handoff(branch).is_some())
            .copied()
            .collect();
        if on_branch.len() == 1 {
            return (on_branch[0].clone(), Vec::new(), None);
        }
    }
    if active.len() == 1 {
        return (active[0].clone(), Vec::new(), None);
    }
    if active.is_empty() {
        return (
            work::Card::default(),
            Vec::new(),
            Some(anyhow::anyhow!(
                "no active card: create one with polterware apply"
            )),
        );
    }
    let message = format!("pass --card <slug>: {}", slugs.join(", "));
    (work::Card::default(), slugs, Some(anyhow::anyhow!(message)))
}

/// Builds the Polter text for a handoff without writing it. Extra diagnostics
/// are hints that the apply path does not see (a note branch that disagrees
/// with Git, when the stored branch is the Git one).
#[allow(clippy::type_complexity)]
pub fn compose_handoff(
    store: &storepkg::Store,
    input: &HandoffInput,
) -> (ApplyInput, Vec<polter::Diagnostic>, Option<anyhow::Error>) {
    let view = project_git(store, &input.project_id);
    if !view.available || view.branch.is_empty() || view.branch == "HEAD" {
        return (
            ApplyInput::default(),
            Vec::new(),
            Some(anyhow::anyhow!(
                "handoff needs a git branch: checkout a branch in the project root"
            )),
        );
    }
    let snapshot = match store.load_work_snapshot(&input.project_id) {
        Ok(snapshot) => snapshot,
        Err(err) => return (ApplyInput::default(), Vec::new(), Some(err.into())),
    };
    let (card, _, err) = select_work_card(&snapshot, &input.card_slug, &view.branch);
    if let Some(err) = err {
        return (ApplyInput::default(), Vec::new(), Some(err));
    }
    let note = polter::parse_handoff_note(&input.note);
    let mut touched = note.touched.clone();
    if touched.is_empty() {
        touched = view.files.clone();
    }
    let node = polter::HandoffNode {
        line: 0,
        branch: view.branch.clone(),
        agent: one_line(&first_non_empty(&[&input.agent, "cli"])),
        goal: one_line(&first_non_empty(&[&note.goal, &card.intent])),
        done: one_line(&first_non_empty(&[
            &note.done,
            &join_task_titles(&card, work::TASK_DONE),
        ])),
        in_progress: one_line(&note.in_progress),
        next: one_line(&first_non_empty(&[
            &note.next,
            &join_task_titles(&card, work::TASK_OPEN),
        ])),
        touched,
    };
    let default_summary = format!("Handoff for {}", card.slug);
    let summary = first_non_empty(&[
        &input.summary,
        &input.apply.summary,
        default_summary.as_str(),
    ]);
    let rationale = first_non_empty(&[&input.rationale, &input.apply.rationale, "Session handoff"]);
    let text = polter::format(&polter::Document {
        summary: summary.clone(),
        rationale: rationale.clone(),
        cards: vec![polter::CardNode {
            slug: card.slug.clone(),
            handoff: Some(node),
            questions: note.questions,
            ..polter::CardNode::default()
        }],
        ..polter::Document::default()
    });
    let mut apply = input.apply.clone();
    apply.project_id = input.project_id.clone();
    apply.text = text;
    apply.summary = summary;
    apply.rationale = rationale;
    let mut hints: Vec<polter::Diagnostic> = Vec::new();
    if !note.branch.is_empty() && note.branch != view.branch {
        hints.push(polter::Diagnostic {
            line: 0,
            severity: polter::SEVERITY_HINT.to_string(),
            message: format!(
                "handoff branch {} does not match git branch {}",
                polter::go_quote(&note.branch),
                polter::go_quote(&view.branch)
            ),
        });
    }
    (apply, hints, None)
}

/// Composes the record and applies it as one audited write.
pub fn write_handoff(
    store: &storepkg::Store,
    input: &HandoffInput,
) -> (ApplyResult, anyhow::Result<()>) {
    let (apply, mut hints, err) = compose_handoff(store, input);
    if let Some(err) = err {
        return (
            ApplyResult {
                project_id: input.project_id.clone(),
                diagnostics: hints,
                ..ApplyResult::default()
            },
            Err(err),
        );
    }
    let mut result = match apply_polter(store, apply) {
        Ok(result) => result,
        Err(failure) => {
            let mut result = *failure.result;
            hints.append(&mut result.diagnostics);
            result.diagnostics = hints;
            return (result, Err(failure.error.into()));
        }
    };
    hints.append(&mut result.diagnostics);
    result.diagnostics = hints;
    (result, Ok(()))
}

/// Folds every run of whitespace into one space.
fn one_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<&str>>().join(" ")
}

fn join_task_titles(card: &work::Card, state: &str) -> String {
    let mut titles: Vec<&str> = Vec::new();
    for task in &card.tasks {
        if task.state == state && !task.title.trim().is_empty() {
            titles.push(&task.title);
        }
    }
    titles.join("; ")
}
