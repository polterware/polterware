//! Differential parity for the Polter codec, byte for byte.
//!
//! Each fixture under `testdata/parity` carries one Polter document, the
//! snapshot it plans against, and the expected text of
//! `Decode + Plan + FormatPlan + Summarize + Brief`. The test replays the same
//! calls and compares the output byte for byte, with generated ULIDs masked so
//! the comparison stays deterministic.
//!
//! The fixtures are the oracle: they were captured once from the reference
//! implementation and never hand-edited.

use std::fs;
use std::path::Path;

use regex::Regex;
use serde_json::Value;

use crate::domains::{structure, work};

use super::model::Diagnostic;
use super::{BriefInput, Doc, brief, decode, format_plan, plan, summarize};

/// Reads a string field, or an empty string.
fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// Reads an integer field, or zero.
fn number(value: &Value, key: &str) -> i32 {
    value.get(key).and_then(Value::as_i64).unwrap_or(0) as i32
}

/// Reads a boolean field, or false.
fn boolean(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

/// Reads an array field as strings, or an empty list.
fn strings(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Reads an array field as objects, or an empty list.
fn objects<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn parse_claim(value: &Value) -> Option<work::Claim> {
    let claim = value.get("Claim").filter(|claim| !claim.is_null())?;
    Some(work::Claim {
        id: text(claim, "ID"),
        project_id: text(claim, "ProjectID"),
        task_id: text(claim, "TaskID"),
        holder: text(claim, "Holder"),
        scope: text(claim, "Scope"),
        ..work::Claim::default()
    })
}

fn parse_task(value: &Value) -> work::Task {
    work::Task {
        id: text(value, "ID"),
        card_id: text(value, "CardID"),
        number: number(value, "Number"),
        title: text(value, "Title"),
        state: text(value, "State"),
        repo: text(value, "Repo"),
        scope: text(value, "Scope"),
        position: number(value, "Position"),
        claim: parse_claim(value),
    }
}

fn parse_question(value: &Value) -> work::Question {
    work::Question {
        id: text(value, "ID"),
        card_id: text(value, "CardID"),
        text: text(value, "Text"),
        blocking: boolean(value, "Blocking"),
        options: strings(value, "Options"),
        state: text(value, "State"),
        answer: text(value, "Answer"),
        position: number(value, "Position"),
    }
}

fn parse_invariant(value: &Value) -> work::Invariant {
    work::Invariant {
        id: text(value, "ID"),
        card_id: text(value, "CardID"),
        text: text(value, "Text"),
        state: text(value, "State"),
        position: number(value, "Position"),
    }
}

fn parse_handoff(value: &Value) -> work::Handoff {
    work::Handoff {
        id: text(value, "ID"),
        project_id: text(value, "ProjectID"),
        card_id: text(value, "CardID"),
        branch: text(value, "Branch"),
        goal: text(value, "Goal"),
        done: text(value, "Done"),
        in_progress: text(value, "InProgress"),
        next: text(value, "Next"),
        touched: strings(value, "Touched"),
        agent: text(value, "Agent"),
        ..work::Handoff::default()
    }
}

fn parse_card(value: &Value) -> work::Card {
    work::Card {
        id: text(value, "ID"),
        project_id: text(value, "ProjectID"),
        slug: text(value, "Slug"),
        title: text(value, "Title"),
        intent: text(value, "Intent"),
        stage: text(value, "Stage"),
        status: text(value, "Status"),
        owner: text(value, "Owner"),
        kind: text(value, "Kind"),
        under: text(value, "Under"),
        decisions: strings(value, "Decisions"),
        milestone_id: text(value, "MilestoneID"),
        milestone: text(value, "Milestone"),
        position: number(value, "Position"),
        tasks: objects(value, "Tasks").iter().map(parse_task).collect(),
        questions: objects(value, "Questions")
            .iter()
            .map(parse_question)
            .collect(),
        invariants: objects(value, "Invariants")
            .iter()
            .map(parse_invariant)
            .collect(),
        handoffs: objects(value, "Handoffs")
            .iter()
            .map(parse_handoff)
            .collect(),
        ..work::Card::default()
    }
}

fn parse_node(value: &Value) -> structure::Node {
    structure::Node {
        id: text(value, "ID"),
        project_id: text(value, "ProjectID"),
        parent_id: text(value, "ParentID"),
        level: text(value, "Level"),
        slug: text(value, "Slug"),
        path: text(value, "Path"),
        title: text(value, "Title"),
        intent: text(value, "Intent"),
        status: text(value, "Status"),
        decisions: strings(value, "Decisions"),
        position: number(value, "Position"),
        repo: text(value, "Repo"),
        folder: text(value, "Folder"),
        kind: text(value, "Kind"),
        tech: text(value, "Tech"),
        migration_from: text(value, "MigrationFrom"),
        migration_to: text(value, "MigrationTo"),
    }
}

fn parse_rule(value: &Value) -> work::Rule {
    work::Rule {
        id: text(value, "ID"),
        slug: text(value, "Slug"),
        statement: text(value, "Statement"),
        why: text(value, "Why"),
        scopes: strings(value, "Scopes"),
        status: text(value, "Status"),
        replaced_by: text(value, "ReplacedBy"),
        position: number(value, "Position"),
    }
}

fn parse_conflict(value: &Value) -> work::Conflict {
    work::Conflict {
        id: text(value, "ID"),
        slug: text(value, "Slug"),
        request: text(value, "Request"),
        rule_id: text(value, "RuleID"),
        decision: text(value, "Decision"),
        card_slug: text(value, "CardSlug"),
        options: strings(value, "Options"),
        state: text(value, "State"),
        answer: text(value, "Answer"),
        created_at: text(value, "CreatedAt"),
        decided_at: text(value, "DecidedAt"),
    }
}

fn parse_milestone(value: &Value) -> work::Milestone {
    work::Milestone {
        id: text(value, "ID"),
        project_id: text(value, "ProjectID"),
        slug: text(value, "Slug"),
        name: text(value, "Name"),
        goal: text(value, "Goal"),
        exit_criteria: strings(value, "ExitCriteria"),
        target_date: text(value, "TargetDate"),
        state: text(value, "State"),
        position: number(value, "Position"),
        archived_at: text(value, "ArchivedAt"),
        ..work::Milestone::default()
    }
}

fn parse_snapshot(value: &Value) -> work::Snapshot {
    work::Snapshot {
        project_id: text(value, "ProjectID"),
        cards: objects(value, "Cards").iter().map(parse_card).collect(),
        nodes: objects(value, "Nodes").iter().map(parse_node).collect(),
        repos: strings(value, "Repos"),
        rules: objects(value, "Rules").iter().map(parse_rule).collect(),
        conflicts: objects(value, "Conflicts")
            .iter()
            .map(parse_conflict)
            .collect(),
        milestones: objects(value, "Milestones")
            .iter()
            .map(parse_milestone)
            .collect(),
    }
}

fn parse_doc(value: &Value) -> Doc {
    Doc {
        id: text(value, "ID"),
        title: text(value, "Title"),
        doc_type: text(value, "DocType"),
        scope_node_ids: strings(value, "ScopeNodeIDs"),
        body: text(value, "Body"),
    }
}

/// Renders diagnostics the way the generator wrote them.
fn diagnostics(out: &mut String, diags: &[Diagnostic]) {
    for diag in diags {
        out.push_str(&format!(
            "{}:{}:{}\n",
            diag.line, diag.severity, diag.message
        ));
    }
}

/// The whole fixture output, in the generator's section order.
fn render(head: &Value, input: &BriefInput, target: &str, budget: i32) -> String {
    let document = text(head, "text");
    let (doc, decode_diags) = decode(&document);
    let (ops, plan_diags) = plan(&doc, &input.snapshot);
    let mut out = String::new();
    out.push_str("## decode\n");
    diagnostics(&mut out, &decode_diags);
    out.push_str("## plan diagnostics\n");
    diagnostics(&mut out, &plan_diags);
    out.push_str("## plan\n");
    out.push_str(&format_plan(&ops));
    out.push_str("## summary\n");
    out.push_str(&summarize(&ops));
    out.push('\n');
    out.push_str("## brief\n");
    match brief(input, target, budget) {
        Ok(brief) => out.push_str(&brief),
        Err(err) => out.push_str(&format!("error: {err}\n")),
    }
    out
}

/// Masks generated ULIDs so a fixture generated once keeps comparing.
fn mask_ulids(text: &str) -> String {
    let ulid = Regex::new(r"\b[0-9A-HJKMNP-TV-Z]{26}\b").expect("the ULID pattern");
    ulid.replace_all(text, "<ulid>").into_owned()
}

#[test]
fn parity_fixtures_match_the_go_oracle() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/domains/polter/testdata/parity");
    let mut files: Vec<_> = fs::read_dir(&dir)
        .expect("the parity fixture directory")
        .map(|entry| entry.expect("a fixture entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    assert!(
        files.len() >= 11,
        "expected at least 11 parity fixtures, found {}",
        files.len()
    );
    let mut failures: Vec<String> = Vec::new();
    for path in &files {
        let raw = fs::read_to_string(path).expect("a fixture file");
        let head: Value = serde_json::from_str(&raw).expect("a fixture document");
        let name = text(&head, "name");
        let snapshot = parse_snapshot(head.get("snapshot").expect("a snapshot"));
        let docs: Vec<Doc> = objects(&head, "docs").iter().map(parse_doc).collect();
        let input = BriefInput {
            snapshot,
            docs,
            canvases: Vec::new(),
        };
        let target = text(&head, "target");
        let budget = number(&head, "budget");
        let want = mask_ulids(&text(&head, "expected"));
        let got = mask_ulids(&render(&head, &input, &target, budget));
        if got != want {
            let diff = first_difference(&got, &want);
            failures.push(format!("{name}: {diff}"));
        }
    }
    assert!(
        failures.is_empty(),
        "parity fixtures differ:\n{}",
        failures.join("\n")
    );
}

/// Names the first line where two outputs differ.
fn first_difference(got: &str, want: &str) -> String {
    let got_lines: Vec<&str> = got.lines().collect();
    let want_lines: Vec<&str> = want.lines().collect();
    for i in 0..got_lines.len().max(want_lines.len()) {
        let left = got_lines.get(i).copied().unwrap_or("<missing>");
        let right = want_lines.get(i).copied().unwrap_or("<missing>");
        if left != right {
            return format!("line {}: got {left:?}, want {right:?}", i + 1);
        }
    }
    "outputs match".to_string()
}
