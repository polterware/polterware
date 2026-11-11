//! Port of `backend/domains/polter/handoff.go`.
//!
//! The session start reads agents get: hints that compare a declared handoff
//! with Git, the `--note` parser, and the status and handoff text.

use std::sync::LazyLock;

use regex::Regex;

use crate::domains::work;

use super::*;

/// A trailing `[blocking]` on a question note.
static TRAILING_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.*?)[\t\n\f\r ]*\[([a-zA-Z]+)\]$").expect("the tag pattern"));

/// The structured part of `polterware handoff --note`. A note with none of the
/// labels is stored as `in_progress`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HandoffNote {
    pub branch: String,
    pub goal: String,
    pub done: String,
    pub in_progress: String,
    pub next: String,
    pub touched: Vec<String>,
    pub questions: Vec<QuestionNode>,
}

/// Hides "delivered" unless Git confirms the commit is on main.
pub fn display_stage(stage: &str, git_confirmed: bool) -> &str {
    if stage == work::STAGE_DELIVERED && !git_confirmed {
        return work::STAGE_VERIFY;
    }
    stage
}

/// Reads a handoff note. Labelled lines fill the record; anything else becomes
/// `in_progress` when no label was used.
pub fn parse_handoff_note(note: &str) -> HandoffNote {
    let mut parsed = HandoffNote::default();
    let mut prose: Vec<String> = Vec::new();
    let mut structured = false;
    for line in note.replace("\r\n", "\n").split('\n') {
        let Some((key, value)) = match_note_key(line) else {
            if !line.trim().is_empty() {
                prose.push(line.trim().to_string());
            }
            continue;
        };
        structured = true;
        match key {
            "branch" => parsed.branch = value,
            "goal" => parsed.goal = value,
            "done" => parsed.done = value,
            "in_progress" => parsed.in_progress = value,
            "next" => parsed.next = value,
            "touched" => parsed.touched = split_list(&value),
            "question" => {
                if let Some(question) = note_question(&value) {
                    parsed.questions.push(question);
                }
            }
            _ => {}
        }
    }
    if !structured {
        parsed.in_progress = note.trim().to_string();
        return parsed;
    }
    if !prose.is_empty() {
        let extra = prose.join("\n");
        if parsed.in_progress.is_empty() {
            parsed.in_progress = extra;
        } else {
            parsed.in_progress = format!("{}\n{extra}", parsed.in_progress);
        }
    }
    parsed
}

/// The note labels, in the order they are matched: the first prefix that fits
/// names the field.
const NOTE_KEYS: [(&str, &str); 8] = [
    ("in progress:", "in_progress"),
    ("in_progress:", "in_progress"),
    ("open question:", "question"),
    ("branch:", "branch"),
    ("goal:", "goal"),
    ("done:", "done"),
    ("next:", "next"),
    ("touched:", "touched"),
];

fn match_note_key(line: &str) -> Option<(&'static str, String)> {
    let trimmed = line.trim();
    let lower = trimmed.to_lowercase();
    for (prefix, key) in NOTE_KEYS {
        if lower.starts_with(prefix) {
            let value = trimmed.get(prefix.len()..).unwrap_or("");
            return Some((key, value.trim().to_string()));
        }
    }
    None
}

fn note_question(value: &str) -> Option<QuestionNode> {
    let mut node = QuestionNode::default();
    let mut body = value.to_string();
    while let Some(captures) = TRAILING_TAG.captures(&body) {
        let tag = captures.get(2).expect("group 2").as_str();
        if tag.eq_ignore_ascii_case("blocking") {
            node.blocking = true;
        }
        body = captures
            .get(1)
            .expect("group 1")
            .as_str()
            .trim()
            .to_string();
    }
    if body.is_empty() {
        return None;
    }
    node.text = body;
    Some(node)
}

/// The short text an agent reads at session start.
pub fn encode_status(
    card: &work::Card,
    branch: &str,
    stage: &str,
    handoff: Option<&work::Handoff>,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{}  {}  branch {}\n",
        card.slug,
        stage,
        branch_or_unknown(branch)
    ));
    match handoff {
        None => out.push_str("no handoff\n"),
        Some(handoff) => {
            write_status_line(&mut out, "Done", &handoff.done);
            write_status_line(&mut out, "In progress", &handoff.in_progress);
            write_status_line(&mut out, "Next", &handoff.next);
        }
    }
    write_status_questions(&mut out, card);
    out
}

/// Lists active claims for status: `claim: T2 codex (scope)` for the current
/// card, then one short line per claim on other cards so an agent sees a
/// conflict before it claims. An empty current slug lists every claim as an
/// other-card line.
pub fn encode_status_claims(current: &str, snapshot: &work::Snapshot) -> String {
    let mut mine = String::new();
    let mut others = String::new();
    for card in &snapshot.cards {
        if card.status != work::CARD_ACTIVE {
            continue;
        }
        for task in &card.tasks {
            let Some(claim) = &task.claim else {
                continue;
            };
            let scope = if claim.scope.is_empty() {
                String::new()
            } else {
                format!(" ({})", claim.scope)
            };
            if card.slug == current {
                mine.push_str(&format!(
                    "claim: T{} {}{scope}\n",
                    task.number, claim.holder
                ));
            } else {
                others.push_str(&format!(
                    "claimed: {} T{} {}{scope}\n",
                    card.slug, task.number, claim.holder
                ));
            }
        }
    }
    format!("{mine}{others}")
}

/// Status when no single card can be chosen.
pub fn encode_status_unresolved(branch: &str, slugs: &[String]) -> String {
    let mut out = format!("branch {}\n", branch_or_unknown(branch));
    if slugs.is_empty() {
        out.push_str("no cards\n");
        return out;
    }
    out.push_str(&format!("pass --card: {}\n", slugs.join(", ")));
    out
}

/// Renders one handoff, plus the card's open questions, as Polter text.
pub fn encode_handoff(card: &work::Card, handoff: &work::Handoff) -> String {
    let mut node = CardNode {
        slug: card.slug.clone(),
        title: card.title.clone(),
        handoff: Some(handoff_node_from(handoff)),
        ..CardNode::default()
    };
    for question in open_question_list(card) {
        node.questions.push(QuestionNode {
            text: question.text.clone(),
            blocking: question.blocking,
            options: question.options.clone(),
            ..QuestionNode::default()
        });
    }
    format(&Document {
        cards: vec![node],
        ..Document::default()
    })
}

fn write_status_line(out: &mut String, label: &str, value: &str) {
    if value.is_empty() {
        return;
    }
    out.push_str(&format!("{label}: {value}\n"));
}

fn write_status_questions(out: &mut String, card: &work::Card) {
    for question in open_question_list(card) {
        out.push_str("? ");
        out.push_str(&question.text);
        if question.blocking {
            out.push_str(" [blocking]");
        }
        out.push('\n');
    }
}

/// The card's open questions, blocking ones first. Go calls this unexported
/// `openQuestions`; `encode.rs` already carries the exported `OpenQuestions`
/// count under the same snake_case name.
fn open_question_list(card: &work::Card) -> Vec<&work::Question> {
    let mut blocking: Vec<&work::Question> = Vec::new();
    let mut rest: Vec<&work::Question> = Vec::new();
    for question in &card.questions {
        if question.state != work::QUESTION_OPEN {
            continue;
        }
        if question.blocking {
            blocking.push(question);
        } else {
            rest.push(question);
        }
    }
    blocking.extend(rest);
    blocking
}

fn branch_or_unknown(branch: &str) -> &str {
    if branch.is_empty() || branch == "HEAD" {
        return "unknown";
    }
    branch
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::polter::decode::{decode, has_diag};
    use crate::domains::polter::has_errors;

    fn must_decode(text: &str) -> Document {
        let (doc, diags) = decode(text);
        assert!(!has_errors(&diags), "decode {text:?}: {diags:?}");
        doc
    }

    fn card_question(text: &str, blocking: bool) -> work::Question {
        work::Question {
            text: text.into(),
            blocking,
            state: work::QUESTION_OPEN.into(),
            ..work::Question::default()
        }
    }

    #[test]
    fn handoff_hints_and_stage() {
        let doc = must_decode(
            "---\npolter: 1\n---\n## git-import: Git import\nhandoff:\nbranch: feat/other\ntouched: docs/spec.md, missing.go\n",
        );
        let git = GitView {
            available: true,
            branch: "feat/git-import".into(),
            files: vec!["docs/spec.md".into(), "backend/import.go".into()],
            delivered: false,
        };
        let diags = handoff_hints(&doc, &git);
        assert!(!has_errors(&diags), "mismatches must be hints: {diags:?}");
        assert!(
            has_diag(&diags, 5, "does not match git branch \"feat/git-import\""),
            "missing branch hint: {diags:?}"
        );
        assert!(
            has_diag(&diags, 5, "not declared backend/import.go"),
            "missing touched hint: {diags:?}"
        );
        assert!(
            has_diag(&diags, 5, "not in the diff missing.go"),
            "missing touched hint: {diags:?}"
        );
        assert!(
            handoff_hints(&doc, &GitView::default()).is_empty(),
            "no repository should not hint"
        );
        assert_eq!(
            display_stage(work::STAGE_DELIVERED, false),
            work::STAGE_VERIFY,
            "unconfirmed delivered must read as verify"
        );
        assert_eq!(
            display_stage(work::STAGE_DELIVERED, true),
            work::STAGE_DELIVERED,
            "git-confirmed delivered stays delivered"
        );
        assert_eq!(
            display_stage(work::STAGE_BUILD, true),
            work::STAGE_BUILD,
            "git must not promote a card to delivered"
        );
    }

    #[test]
    fn parse_handoff_note_reads_labels() {
        let note = parse_handoff_note("still wiring the importer");
        assert_eq!(note.in_progress, "still wiring the importer", "free note");
        assert_eq!(note.goal, "", "free note has no goal");
        let note = parse_handoff_note(
            "Goal: import commits\nDone: parser\nIn progress: status\nNext: docs\nTouched: a.go, b.go\nOpen question: include merges? [blocking]\nBranch: other\n",
        );
        assert_eq!(note.goal, "import commits", "goal");
        assert_eq!(note.done, "parser", "done");
        assert_eq!(note.in_progress, "status", "in progress");
        assert_eq!(note.next, "docs", "next");
        assert_eq!(note.touched.len(), 2, "touched");
        assert_eq!(note.branch, "other", "branch");
        assert_eq!(note.questions.len(), 1, "one question");
        assert!(note.questions[0].blocking, "the question is blocking");
    }

    #[test]
    fn encode_status_lists_blocking_questions_first() {
        let card = work::Card {
            slug: "git-import".into(),
            stage: work::STAGE_BUILD.into(),
            questions: vec![
                card_question("later?", false),
                card_question("include merges?", true),
            ],
            ..work::Card::default()
        };
        let handoff = work::Handoff {
            done: "parser".into(),
            in_progress: "status".into(),
            next: "docs".into(),
            ..work::Handoff::default()
        };
        let status = encode_status(&card, "feat/git-import", work::STAGE_VERIFY, Some(&handoff));
        assert!(
            status.contains("git-import  verify  branch feat/git-import"),
            "status = {status:?}"
        );
        assert!(status.contains("Done: parser"), "status = {status:?}");
        let blocking = status.find("include merges? [blocking]");
        let later = status.find("later?");
        assert!(
            blocking.is_some() && later.is_some() && blocking < later,
            "blocking question should come first:\n{status}"
        );
    }

    #[test]
    fn claim_status_lines() {
        let mut snapshot = work::Snapshot {
            project_id: "p1".into(),
            cards: vec![
                work::Card {
                    id: "c1".into(),
                    slug: "git-import".into(),
                    title: "Git import".into(),
                    stage: work::STAGE_BUILD.into(),
                    status: work::CARD_ACTIVE.into(),
                    tasks: vec![
                        work::Task {
                            id: "t1".into(),
                            card_id: "c1".into(),
                            number: 1,
                            title: "Evidence".into(),
                            state: work::TASK_OPEN.into(),
                            scope: "backend/import/*".into(),
                            ..work::Task::default()
                        },
                        work::Task {
                            id: "t2".into(),
                            card_id: "c1".into(),
                            number: 2,
                            title: "Stamp".into(),
                            state: work::TASK_OPEN.into(),
                            scope: "backend/domains/canon/*".into(),
                            ..work::Task::default()
                        },
                        work::Task {
                            id: "t3".into(),
                            card_id: "c1".into(),
                            number: 3,
                            title: "Docs".into(),
                            state: work::TASK_OPEN.into(),
                            ..work::Task::default()
                        },
                        work::Task {
                            id: "t4".into(),
                            card_id: "c1".into(),
                            number: 4,
                            title: "Old".into(),
                            state: work::TASK_DONE.into(),
                            scope: "docs/*".into(),
                            ..work::Task::default()
                        },
                    ],
                    ..work::Card::default()
                },
                work::Card {
                    id: "c2".into(),
                    slug: "canon-core".into(),
                    title: "Canon core".into(),
                    stage: work::STAGE_BUILD.into(),
                    status: work::CARD_ACTIVE.into(),
                    tasks: vec![work::Task {
                        id: "u1".into(),
                        card_id: "c2".into(),
                        number: 1,
                        title: "Core".into(),
                        state: work::TASK_OPEN.into(),
                        scope: "backend/*".into(),
                        claim: Some(work::Claim {
                            id: "k1".into(),
                            project_id: "p1".into(),
                            task_id: "u1".into(),
                            holder: "claude".into(),
                            scope: "backend/*".into(),
                            ..work::Claim::default()
                        }),
                        ..work::Task::default()
                    }],
                    ..work::Card::default()
                },
            ],
            ..work::Snapshot::default()
        };
        snapshot.cards[0].tasks[2].claim = Some(work::Claim {
            holder: "codex".into(),
            ..work::Claim::default()
        });
        let status = encode_status_claims("git-import", &snapshot);
        assert_eq!(
            status, "claim: T3 codex\nclaimed: canon-core T1 claude (backend/*)\n",
            "status claims"
        );
    }

    #[test]
    fn unresolved_status_names_the_cards() {
        assert_eq!(
            encode_status_unresolved("", &["a".to_string(), "b".to_string()]),
            "branch unknown\npass --card: a, b\n"
        );
        assert_eq!(
            encode_status_unresolved("main", &[]),
            "branch main\nno cards\n"
        );
    }

    #[test]
    fn handoff_encode_round_trips_the_handoff_and_questions() {
        let card = work::Card {
            slug: "git-import".into(),
            title: "Git import".into(),
            questions: vec![
                card_question("later?", false),
                work::Question {
                    options: vec!["yes".into(), "no".into()],
                    ..card_question("include merges?", true)
                },
            ],
            ..work::Card::default()
        };
        let handoff = work::Handoff {
            branch: "feat/git-import".into(),
            agent: "codex".into(),
            done: "parser".into(),
            next: "docs".into(),
            touched: vec!["a.go".into()],
            ..work::Handoff::default()
        };
        let text = encode_handoff(&card, &handoff);
        assert!(text.contains("## git-import: Git import"), "text = {text}");
        assert!(text.contains("branch: feat/git-import"), "text = {text}");
        assert!(text.contains("agent: codex"), "text = {text}");
        assert!(text.contains("touched: a.go"), "text = {text}");
        assert!(
            text.contains("? include merges? [blocking] (yes | no)"),
            "text = {text}"
        );
        assert!(text.contains("? later?"), "text = {text}");
    }
}
