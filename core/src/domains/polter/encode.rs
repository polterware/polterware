//! Port of `backend/domains/polter/encode.go`: canonical rendering of a
//! decoded document, a stored card, and the card list.
//!
//! `format(Decode(format(Decode(x))))` equals `format(Decode(x))`: the codec's
//! round-trip property.

use crate::domains::work;

use super::model::*;

/// The canonical frontmatter every encoded document starts with.
pub const HEADER: &str = "---\npolter: 1\n---\n";

/// Renders a [`Document`] in canonical form.
pub fn format(doc: &Document) -> String {
    let mut out = String::new();
    out.push_str("---\npolter: 1\n");
    if !doc.summary.is_empty() {
        out.push_str(&format!("summary: {}\n", doc.summary));
    }
    if !doc.rationale.is_empty() {
        out.push_str(&format!("rationale: {}\n", doc.rationale));
    }
    out.push_str("---\n");
    for node in &doc.nodes {
        out.push('\n');
        write_structure_node(&mut out, node);
    }
    for card in &doc.cards {
        out.push('\n');
        write_card(&mut out, card);
    }
    for decision in &doc.decisions {
        out.push('\n');
        write_decision(&mut out, decision);
    }
    for fact in &doc.facts {
        out.push_str(&format!("\n#### {}", fact.reference));
        if !fact.statement.is_empty() {
            out.push_str(&format!(": {}", fact.statement));
        }
        out.push('\n');
        for key in FACT_META_KEYS {
            if let Some(value) = fact.meta_value(key) {
                out.push_str(&format!("{key}: {value}\n"));
            }
        }
    }
    for rule in &doc.rules {
        out.push_str(&format!("\n#### {RULE_HEADING_PREFIX}{}", rule.reference));
        if !rule.statement.is_empty() {
            out.push_str(&format!(": {}", rule.statement));
        }
        out.push('\n');
        for key in RULE_META_KEYS {
            if let Some(value) = rule.meta_value(key)
                && !value.is_empty()
            {
                out.push_str(&format!("{key}: {value}\n"));
            }
        }
        if rule.why_set || !rule.why.is_empty() {
            write_rule_why(&mut out, &rule.why);
        }
    }
    for conflict in &doc.conflicts {
        out.push_str(&format!(
            "\n#### {CONFLICT_HEADING_PREFIX}{}",
            conflict.reference
        ));
        if !conflict.request.is_empty() {
            out.push_str(&format!(": {}", conflict.request));
        }
        out.push('\n');
        for key in CONFLICT_META_KEYS {
            if let Some(value) = conflict.meta_value(key) {
                out.push_str(&format!("{key}: {value}\n"));
            }
        }
    }
    out
}

/// Renders one stored card as a full Polter document.
pub fn encode_card(card: &work::Card) -> String {
    let mut out = String::new();
    out.push_str(HEADER);
    out.push('\n');
    write_card(&mut out, &node_from_card(card));
    out
}

/// A card as a difficulty classifier reads it: the heading, intent, tasks,
/// questions, and invariants, with no metadata or handoff.
pub fn task_text(card: &work::Card) -> String {
    let mut node = node_from_card(card);
    node.meta.clear();
    node.handoff = None;
    let mut out = String::new();
    write_card(&mut out, &node);
    out.trim().to_string()
}

/// Converts a stored card to its AST. Removed questions and invariants are
/// left out; dropped tasks stay so their numbers remain visible.
pub fn node_from_card(card: &work::Card) -> CardNode {
    let mut node = CardNode {
        slug: card.slug.clone(),
        title: card.title.clone(),
        intent: card.intent.clone(),
        intent_set: !card.intent.is_empty(),
        ..CardNode::default()
    };
    node.meta
        .push(MetaEntry::new("stage", card.stage.clone(), 0));
    if !card.owner.is_empty() {
        node.meta
            .push(MetaEntry::new("owner", card.owner.clone(), 0));
    }
    if !card.under.is_empty() {
        node.meta
            .push(MetaEntry::new("under", card.under.clone(), 0));
    }
    if !card.milestone.is_empty() {
        node.meta
            .push(MetaEntry::new("milestone", card.milestone.clone(), 0));
    }
    if card.kind == work::KIND_ARCHITECTURE {
        node.meta.push(MetaEntry::new("kind", card.kind.clone(), 0));
    }
    if !card.decisions.is_empty() {
        node.meta
            .push(MetaEntry::new("decisions", card.decisions.join(", "), 0));
    }
    if card.status == work::CARD_REMOVED {
        node.meta
            .push(MetaEntry::new("status", work::CARD_REMOVED, 0));
    }
    for task in &card.tasks {
        node.tasks.push(TaskNode {
            line: 0,
            number: task.number,
            title: task.title.clone(),
            state: task.state.clone(),
            repo: task.repo.clone(),
            scope: task.scope.clone(),
        });
    }
    for question in &card.questions {
        match question.state.as_str() {
            work::QUESTION_OPEN => node.questions.push(QuestionNode {
                text: question.text.clone(),
                blocking: question.blocking,
                options: question.options.clone(),
                ..QuestionNode::default()
            }),
            work::QUESTION_ANSWERED => node.questions.push(QuestionNode {
                text: question.text.clone(),
                blocking: question.blocking,
                options: question.options.clone(),
                answer: question.answer.clone(),
                ..QuestionNode::default()
            }),
            _ => {}
        }
    }
    for invariant in &card.invariants {
        if invariant.state != work::INVARIANT_ACTIVE {
            continue;
        }
        node.invariants.push(InvariantNode {
            text: invariant.text.clone(),
            ..InvariantNode::default()
        });
    }
    if let Some(handoff) = card.latest_handoff_any() {
        node.handoff = Some(handoff_node_from(handoff));
    }
    node
}

/// Converts a stored handoff to its AST block.
pub fn handoff_node_from(handoff: &work::Handoff) -> HandoffNode {
    HandoffNode {
        line: 0,
        branch: handoff.branch.clone(),
        agent: handoff.agent.clone(),
        goal: handoff.goal.clone(),
        done: handoff.done.clone(),
        in_progress: handoff.in_progress.clone(),
        next: handoff.next.clone(),
        touched: handoff.touched.clone(),
    }
}

fn write_structure_node(out: &mut String, node: &StructureNode) {
    let path = if node.path.is_empty() {
        node.slug.as_str()
    } else {
        node.path.as_str()
    };
    out.push_str(&format!("# {path}: {}\n", node.title));
    let mut meta: Vec<String> = Vec::new();
    for key in NODE_META_KEYS {
        if let Some(value) = node.meta_value(key)
            && !value.is_empty()
        {
            meta.push(format!("{key}: {value}"));
        }
    }
    if !meta.is_empty() {
        out.push_str(&meta.join(" · "));
        out.push('\n');
    }
    if !node.intent.is_empty() {
        out.push('\n');
        out.push_str(&node.intent);
        out.push('\n');
    }
    if !node.invariants.is_empty() && (!node.intent.is_empty() || !meta.is_empty()) {
        out.push('\n');
    }
    for invariant in &node.invariants {
        out.push_str("! ");
        out.push_str(&invariant.text);
        if invariant.removed {
            out.push_str(" [removed]");
        }
        out.push('\n');
    }
}

fn write_card(out: &mut String, card: &CardNode) {
    if !card.title.is_empty() {
        out.push_str(&format!("## {}: {}\n", card.slug, card.title));
    } else {
        out.push_str(&format!("## {}\n", card.slug));
    }
    let meta = canonical_meta(card);
    if !meta.is_empty() {
        let parts: Vec<String> = meta
            .iter()
            .map(|entry| format!("{}: {}", entry.key, entry.value))
            .collect();
        out.push_str(&parts.join(" · "));
        out.push('\n');
    }
    if let Some(handoff) = &card.handoff {
        out.push('\n');
        write_handoff(out, handoff);
    }
    if card.intent_set {
        out.push('\n');
        out.push_str(&card.intent);
        out.push('\n');
    }
    if !card.tasks.is_empty() || !card.questions.is_empty() || !card.invariants.is_empty() {
        out.push('\n');
    }
    for task in &card.tasks {
        let boxed = match task.state.as_str() {
            work::TASK_DONE => "x",
            work::TASK_DROPPED => "-",
            _ => " ",
        };
        out.push_str(&format!("- [{boxed}] "));
        if task.number > 0 {
            out.push_str(&format!("T{} ", task.number));
        }
        out.push_str(&task.title);
        if !task.repo.is_empty() {
            out.push_str(&format!(" [repo: {}]", task.repo));
        }
        if !task.scope.is_empty() {
            out.push_str(&format!(" `{}`", task.scope));
        }
        out.push('\n');
    }
    for question in &card.questions {
        out.push_str("? ");
        out.push_str(&question.text);
        if question.blocking {
            out.push_str(" [blocking]");
        }
        if question.removed {
            out.push_str(" [removed]");
        }
        if !question.options.is_empty() {
            out.push_str(&format!(" ({})", question.options.join(" | ")));
        }
        if !question.answer.is_empty() {
            out.push_str(&format!(" -> {}", question.answer));
        }
        out.push('\n');
    }
    for invariant in &card.invariants {
        out.push_str("! ");
        out.push_str(&invariant.text);
        if invariant.removed {
            out.push_str(" [removed]");
        }
        out.push('\n');
    }
}

fn write_handoff(out: &mut String, handoff: &HandoffNode) {
    out.push_str("handoff:\n");
    out.push_str(&format!("branch: {}\n", handoff.branch));
    write_handoff_field(out, "agent", &handoff.agent);
    write_handoff_field(out, "goal", &handoff.goal);
    write_handoff_field(out, "done", &handoff.done);
    write_handoff_field(out, "in_progress", &handoff.in_progress);
    write_handoff_field(out, "next", &handoff.next);
    if !handoff.touched.is_empty() {
        out.push_str(&format!("touched: {}\n", handoff.touched.join(", ")));
    }
}

fn write_handoff_field(out: &mut String, key: &str, value: &str) {
    if value.is_empty() {
        return;
    }
    out.push_str(&format!("{key}: {value}\n"));
}

/// Keeps the last value per key, in `META_KEYS` order.
fn canonical_meta(card: &CardNode) -> Vec<MetaEntry> {
    let mut out = Vec::new();
    for key in META_KEYS {
        if let Some(value) = card.meta_value(key) {
            out.push(MetaEntry::new(key, value, 0));
        }
    }
    out
}

/// Renders active cards one per line:
/// slug  stage  2/5 tasks  1 open?  "Title"
pub fn encode_list(snapshot: &work::Snapshot) -> String {
    struct Row {
        slug: String,
        stage: String,
        tasks: String,
        questions: String,
        title: String,
    }
    let mut rows: Vec<Row> = Vec::new();
    let mut widths = [0usize; 4];
    for card in &snapshot.cards {
        if card.status != work::CARD_ACTIVE {
            continue;
        }
        let (done, total) = task_progress(card);
        let mut row = Row {
            slug: card.slug.clone(),
            stage: card.stage.clone(),
            tasks: format!("{done}/{total} tasks"),
            questions: String::new(),
            title: card.title.clone(),
        };
        let open = open_questions(card);
        if open > 0 {
            row.questions = format!("{open} open?");
        }
        for (i, value) in [&row.slug, &row.stage, &row.tasks, &row.questions]
            .into_iter()
            .enumerate()
        {
            widths[i] = widths[i].max(value.len());
        }
        rows.push(row);
    }
    if rows.is_empty() {
        return "no cards\n".to_string();
    }
    let mut out = String::new();
    for row in &rows {
        out.push_str(&format!(
            "{}  {}  {}  ",
            pad(&row.slug, widths[0]),
            pad(&row.stage, widths[1]),
            pad(&row.tasks, widths[2])
        ));
        if widths[3] > 0 {
            out.push_str(&format!("{}  ", pad(&row.questions, widths[3])));
        }
        out.push_str(&format!("{}\n", go_quote(&row.title)));
    }
    out
}

/// Left-pads a value with spaces to `width` characters, like Go's `%-*s`.
fn pad(value: &str, width: usize) -> String {
    let mut out = value.to_string();
    let len = value.chars().count();
    if len < width {
        out.push_str(&" ".repeat(width - len));
    }
    out
}

/// Counts done tasks against non-dropped tasks.
pub fn task_progress(card: &work::Card) -> (i32, i32) {
    let (mut done, mut total) = (0, 0);
    for task in &card.tasks {
        match task.state.as_str() {
            work::TASK_DONE => {
                done += 1;
                total += 1;
            }
            work::TASK_OPEN => total += 1,
            _ => {}
        }
    }
    (done, total)
}

/// Counts questions still waiting for a decision.
pub fn open_questions(card: &work::Card) -> i32 {
    card.questions
        .iter()
        .filter(|question| question.state == work::QUESTION_OPEN)
        .count() as i32
}

/// Writes a Rule's `why:` line and, when it spans lines, the Markdown body
/// under it. The blank `why:` keeps a cleared reason round-trippable.
pub(crate) fn write_rule_why(out: &mut String, why: &str) {
    let why = why.trim_end_matches('\n');
    if why.is_empty() {
        out.push_str("why:\n");
        return;
    }
    let mut lines = why.split('\n');
    out.push_str(&format!("why: {}\n", lines.next().unwrap_or_default()));
    for line in lines {
        out.push_str(line);
        out.push('\n');
    }
}

pub(crate) fn write_decision(out: &mut String, decision: &DecisionNode) {
    if !decision.title.is_empty() {
        out.push_str(&format!("### {}: {}\n", decision.reference, decision.title));
    } else {
        out.push_str(&format!("### {}\n", decision.reference));
    }
    let mut meta: Vec<String> = Vec::new();
    for key in DECISION_META_KEYS {
        if let Some(value) = decision.meta_value(key)
            && !value.is_empty()
        {
            meta.push(format!("{key}: {value}"));
        }
    }
    if !meta.is_empty() {
        out.push_str(&meta.join(" · "));
        out.push('\n');
    }
    if !decision.body.is_empty() {
        out.push('\n');
        out.push_str(&decision.body);
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::polter::decode::{SAMPLE, decode};

    #[test]
    fn format_round_trips() {
        let (doc, _) = decode(SAMPLE);
        let once = format(&doc);
        let (again, diags) = decode(&once);
        assert!(
            diags.is_empty(),
            "canonical text has diagnostics: {diags:?}\n{once}"
        );
        let twice = format(&again);
        assert_eq!(twice, once, "the round trip differs");
    }

    #[test]
    fn encodes_a_card_and_reads_it_back() {
        let card = work::Card {
            slug: "git-import".into(),
            title: "Git import".into(),
            intent: "Import HEAD.".into(),
            stage: work::STAGE_BUILD.into(),
            status: work::CARD_ACTIVE.into(),
            under: "core/import".into(),
            milestone: "beta".into(),
            kind: work::KIND_ARCHITECTURE.into(),
            decisions: vec!["Decision-0007".into()],
            tasks: vec![
                work::Task {
                    number: 1,
                    title: "Wire import".into(),
                    state: work::TASK_OPEN.into(),
                    ..work::Task::default()
                },
                work::Task {
                    number: 2,
                    title: "Stamp".into(),
                    state: work::TASK_DONE.into(),
                    repo: "api".into(),
                    scope: "src/auth/*".into(),
                    ..work::Task::default()
                },
            ],
            questions: vec![work::Question {
                text: "Include merge commits?".into(),
                blocking: true,
                options: vec!["yes".into(), "no".into()],
                state: work::QUESTION_OPEN.into(),
                ..work::Question::default()
            }],
            invariants: vec![work::Invariant {
                text: "No network calls".into(),
                state: work::INVARIANT_ACTIVE.into(),
                ..work::Invariant::default()
            }],
            ..work::Card::default()
        };
        let text = encode_card(&card);
        let (doc, diags) = decode(&text);
        assert!(!has_errors(&diags), "encode diags: {diags:?}\n{text}");
        let node = &doc.cards[0];
        assert_eq!(node.slug, "git-import");
        assert_eq!(node.meta_value("stage"), Some("build"));
        assert_eq!(node.meta_value("milestone"), Some("beta"));
        assert_eq!(node.meta_value("kind"), Some("architecture"));
        assert_eq!(node.meta_value("decisions"), Some("Decision-0007"));
        assert_eq!(node.tasks[1].repo, "api", "the task checkout");
        assert_eq!(node.tasks[1].scope, "src/auth/*", "the task glob");
        assert_eq!(node.questions[0].options.len(), 2);
        assert_eq!(node.invariants[0].text, "No network calls");
    }

    #[test]
    fn task_text_keeps_only_what_a_classifier_reads() {
        let card = work::Card {
            slug: "git-import".into(),
            title: "Git import".into(),
            intent: "Import HEAD.".into(),
            stage: work::STAGE_BUILD.into(),
            owner: "erick".into(),
            tasks: vec![work::Task {
                number: 1,
                title: "Wire import".into(),
                state: work::TASK_OPEN.into(),
                ..work::Task::default()
            }],
            ..work::Card::default()
        };
        let text = task_text(&card);
        assert!(text.contains("## git-import: Git import"), "{text}");
        assert!(text.contains("- [ ] T1 Wire import"), "{text}");
        assert!(!text.contains("stage:"), "no metadata: {text}");
        assert!(!text.trim_end().ends_with('\n'), "trimmed: {text:?}");
    }

    #[test]
    fn encode_list_aligns_its_columns() {
        let snapshot = work::Snapshot {
            cards: vec![
                work::Card {
                    slug: "a".into(),
                    stage: work::STAGE_SHAPE.into(),
                    status: work::CARD_ACTIVE.into(),
                    title: "A".into(),
                    tasks: vec![work::Task {
                        state: work::TASK_DONE.into(),
                        ..work::Task::default()
                    }],
                    questions: vec![work::Question {
                        state: work::QUESTION_OPEN.into(),
                        ..work::Question::default()
                    }],
                    ..work::Card::default()
                },
                work::Card {
                    slug: "longer".into(),
                    stage: work::STAGE_DELIVERED.into(),
                    status: work::CARD_REMOVED.into(),
                    title: "Gone".into(),
                    ..work::Card::default()
                },
            ],
            ..work::Snapshot::default()
        };
        assert_eq!(
            encode_list(&snapshot),
            "a  shape  1/1 tasks  1 open?  \"A\"\n",
            "one line per active card"
        );
        assert_eq!(
            encode_list(&work::Snapshot::default()),
            "no cards\n",
            "an empty projection"
        );
    }

    #[test]
    fn progress_counts_only_open_and_done() {
        let card = work::Card {
            tasks: vec![
                work::Task {
                    state: work::TASK_DONE.into(),
                    ..work::Task::default()
                },
                work::Task {
                    state: work::TASK_OPEN.into(),
                    ..work::Task::default()
                },
                work::Task {
                    state: work::TASK_DROPPED.into(),
                    ..work::Task::default()
                },
            ],
            ..work::Card::default()
        };
        assert_eq!(
            task_progress(&card),
            (1, 2),
            "dropped tasks are not counted"
        );
        assert_eq!(open_questions(&card), 0, "questions count separately");
    }

    #[test]
    fn encoded_task_scope_and_repo_survive_a_round_trip() {
        let (doc, diags) = decode("---\npolter: 1\n---\n## a: A\n- [ ] T1 Fix `api:src/auth/*`\n");
        assert!(!has_errors(&diags), "{diags:?}");
        let once = format(&doc);
        assert!(
            once.contains("- [ ] T1 Fix [repo: api] `src/auth/*`\n"),
            "{once}"
        );
    }

    #[test]
    fn a_rule_round_trips_with_every_field() {
        let text = concat!(
            "---\npolter: 1\n---\n\n",
            "#### rule local-only: Never call the network from the core.\n",
            "scope: core, core/import\n",
            "status: under_review\n",
            "replaced-by: new-local-only\n",
            "why: The core reads the local store only.\n",
            "Offline installs and tests must not need a network.\n",
            "\n",
            "#### rule new-local-only: Read the store.\n",
            "status: active\n",
        );
        let (doc, diags) = decode(text);
        assert!(!has_errors(&diags), "decode: {diags:?}");
        let rule = &doc.rules[0];
        assert_eq!(rule.statement, "Never call the network from the core.");
        assert_eq!(rule.meta_value("scope"), Some("core, core/import"));
        assert_eq!(rule.meta_value("status"), Some("under_review"));
        assert_eq!(rule.meta_value("replaced-by"), Some("new-local-only"));
        assert_eq!(
            rule.why,
            "The core reads the local store only.\nOffline installs and tests must not need a network."
        );
        assert!(rule.why_set);
        let once = format(&doc);
        for line in [
            "#### rule local-only: Never call the network from the core.",
            "scope: core, core/import",
            "status: under_review",
            "replaced-by: new-local-only",
            "why: The core reads the local store only.",
            "Offline installs and tests must not need a network.",
        ] {
            assert!(once.contains(line), "missing {line:?} in {once}");
        }
        let (again, diags) = decode(&once);
        assert!(diags.is_empty(), "canonical text: {diags:?}\n{once}");
        assert_eq!(format(&again), once, "the rule round trip differs");
        assert_eq!(again.rules[0].statement, doc.rules[0].statement);
        assert_eq!(again.rules[0].meta, doc.rules[0].meta);
        assert_eq!(again.rules[0].why, doc.rules[0].why);
        assert_eq!(again.rules[0].why_set, doc.rules[0].why_set);
    }

    #[test]
    fn a_cleared_rule_why_round_trips() {
        let (doc, diags) = decode("---\npolter: 1\n---\n#### rule local-only: Stay local.\nwhy:\n");
        assert!(!has_errors(&diags), "{diags:?}");
        assert!(doc.rules[0].why_set, "the block carried why:");
        assert!(doc.rules[0].why.is_empty(), "the reason is empty");
        let once = format(&doc);
        assert!(once.contains("why:\n"), "{once}");
        let (again, _) = decode(&once);
        assert!(again.rules[0].why_set, "an empty why survives");
        assert!(again.rules[0].why.is_empty());
    }
}
