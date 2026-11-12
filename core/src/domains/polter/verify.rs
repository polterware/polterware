//! What a card's `confirm:` lines record, verified.
//!
//! Verify reports whether a card (or one task) is finished. It is a read: it
//! never moves the stored stage. Beyond tasks and questions it lists the card's
//! acceptance criteria and the binding Rules scoped to its `under:` chain as
//! manual checks, each a pass once a person or an agent confirmed it; the
//! binding whole-project Rules are one info line, never one check each.

use crate::domains::work;

use super::feature::ReviewInvariant;
use super::*;

/// The verify check statuses. Only fail makes the result fail; manual lines are
/// contract items a person or agent confirms by hand.
pub const VERIFY_PASS: &str = "pass";
pub const VERIFY_FAIL: &str = "fail";
pub const VERIFY_INFO: &str = "info";
pub const VERIFY_MANUAL: &str = "manual";

/// What verify reads for one card, or one task when `task > 0`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VerifyInput {
    pub card: work::Card,
    pub task: i32,
    /// True when HEAD is an ancestor of local main or master.
    pub delivered: bool,
    /// The card's own invariants (its acceptance criteria).
    pub contract: Vec<ReviewInvariant>,
    /// The binding Rules that apply to the card: its `under:` chain's, each a
    /// manual check, and the whole project's, which are one info line.
    pub rules: Vec<VerifyRule>,
    /// What a person or an agent already confirmed for the card.
    pub confirmed: Vec<work::Confirmation>,
}

/// One Rule that applies to the card. `scope` is the node path that brings it
/// in, empty for a whole-project Rule, which is listed as info only.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VerifyRule {
    pub id: String,
    pub slug: String,
    pub statement: String,
    pub scope: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VerifyCheck {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VerifyResult {
    pub target: String,
    pub passed: bool,
    pub checks: Vec<VerifyCheck>,
    /// Every manual item, confirmed or not: what the board counts as
    /// "confirmed / total" and what the Inbox counts as still open.
    pub manual: Vec<ManualItem>,
}

/// One manual check: an acceptance criterion or a binding Rule scoped to the
/// card's `under:` chain, and whether a person or an agent confirmed it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManualItem {
    /// The line verify prints, with its origin and who confirmed it.
    pub label: String,
    /// The `confirm:` target that answers it: `! <criterion>` or `rule <slug>`.
    pub target: String,
    pub confirmed: bool,
}

/// The manual checks a person or an agent has not confirmed yet.
pub fn unconfirmed(result: &VerifyResult) -> Vec<String> {
    result
        .checks
        .iter()
        .filter(|check| check.status == VERIFY_MANUAL)
        .map(|check| check.message.clone())
        .collect()
}

/// Names who confirmed an item, for the pass line.
fn confirmed_by(confirmation: &work::Confirmation) -> String {
    let who = confirmation.confirmed_by.trim();
    if who.is_empty() {
        "a person".to_string()
    } else {
        who.to_string()
    }
}

fn confirmation_for<'a>(
    confirmed: &'a [work::Confirmation],
    kind: &str,
    target_id: &str,
) -> Option<&'a work::Confirmation> {
    if target_id.is_empty() {
        return None;
    }
    confirmed
        .iter()
        .find(|item| item.kind == kind && item.target_id == target_id)
}

/// The card's manual items: one per acceptance criterion and one per binding
/// Rule scoped to the card's `under:` chain. A confirmed item says who
/// confirmed it and carries the confirmed flag; the rest wait for a person or
/// an agent. The binding whole-project Rules hold for every card at once, so
/// they are one info line ("Constitution: N rules apply") and never a manual
/// item, a confirmation, or an Inbox count.
pub fn manual_items(
    contract: &[ReviewInvariant],
    rules: &[VerifyRule],
    confirmed: &[work::Confirmation],
) -> Vec<ManualItem> {
    let mut items: Vec<ManualItem> = Vec::new();
    for invariant in contract {
        let from = if invariant.from.is_empty() || invariant.from == "card" {
            String::new()
        } else {
            format!(" (from {})", invariant.from)
        };
        let label = format!("! {}{from}", invariant.text);
        let target = format!("! {}", invariant.text);
        items.push(
            match confirmation_for(confirmed, work::CONFIRMATION_INVARIANT, &invariant.id) {
                Some(item) => ManualItem {
                    label: format!("{label} (confirmed by {})", confirmed_by(item)),
                    target,
                    confirmed: true,
                },
                None => ManualItem {
                    label,
                    target,
                    confirmed: false,
                },
            },
        );
    }
    for rule in rules.iter().filter(|rule| !rule.scope.is_empty()) {
        // A node's `!` line becomes a Rule with no slug; its id still names it.
        let reference = if rule.slug.is_empty() {
            rule.id.as_str()
        } else {
            rule.slug.as_str()
        };
        let label = format!(
            "rule {}: {} (from {})",
            reference, rule.statement, rule.scope
        );
        let target = format!("rule {reference}");
        items.push(
            match confirmation_for(confirmed, work::CONFIRMATION_RULE, &rule.id) {
                Some(item) => ManualItem {
                    label: format!("{label} (confirmed by {})", confirmed_by(item)),
                    target,
                    confirmed: true,
                },
                None => ManualItem {
                    label,
                    target,
                    confirmed: false,
                },
            },
        );
    }
    items
}

/// The card's manual checks: one per acceptance criterion and one per binding
/// Rule scoped to the card's `under:` chain. A confirmed item is a pass that
/// names who confirmed it; the rest stay manual for a person or an agent to
/// confirm. The binding whole-project Rules hold for every card at once, so
/// they are one info line ("Constitution: N rules apply") and never a manual
/// item, a confirmation, or an Inbox count.
pub fn manual_checks(
    contract: &[ReviewInvariant],
    rules: &[VerifyRule],
    confirmed: &[work::Confirmation],
) -> Vec<VerifyCheck> {
    let mut checks: Vec<VerifyCheck> = manual_items(contract, rules, confirmed)
        .into_iter()
        .map(|item| VerifyCheck {
            status: if item.confirmed {
                VERIFY_PASS.to_string()
            } else {
                VERIFY_MANUAL.to_string()
            },
            message: item.label,
        })
        .collect();
    let constitution = rules.iter().filter(|rule| rule.scope.is_empty()).count();
    if constitution > 0 {
        checks.push(VerifyCheck {
            status: VERIFY_INFO.to_string(),
            message: if constitution == 1 {
                "Constitution: 1 rule applies".to_string()
            } else {
                format!("Constitution: {constitution} rules apply")
            },
        });
    }
    checks
}

/// Reports whether a card (or one task) is finished: tasks done or dropped, no
/// blocking question open. It is a read; it never moves the stored stage.
pub fn verify_feature(input: &VerifyInput) -> Result<VerifyResult, String> {
    let card = &input.card;
    let mut result = VerifyResult {
        target: card.slug.clone(),
        ..VerifyResult::default()
    };
    let mut tasks: Vec<&work::Task> = card.tasks.iter().collect();
    if input.task > 0 {
        result.target = format!("{}#T{}", card.slug, input.task);
        tasks = card
            .tasks
            .iter()
            .filter(|task| task.number == input.task)
            .collect();
        if tasks.is_empty() {
            return Err(format!("card {} has no task T{}", card.slug, input.task));
        }
    } else if tasks.is_empty() {
        result.checks.push(VerifyCheck {
            status: VERIFY_FAIL.to_string(),
            message: "no tasks: plan the feature before verifying it".to_string(),
        });
    }
    for task in &tasks {
        let (status, message) = match task.state.as_str() {
            work::TASK_OPEN => (
                VERIFY_FAIL,
                format!("T{} is open: {}", task.number, task.title),
            ),
            work::TASK_DROPPED => (
                VERIFY_INFO,
                format!("T{} dropped: {}", task.number, task.title),
            ),
            work::TASK_DONE => (
                VERIFY_PASS,
                format!("T{} done: {}", task.number, task.title),
            ),
            _ => continue,
        };
        result.checks.push(VerifyCheck {
            status: status.to_string(),
            message,
        });
    }

    if input.task == 0 {
        for question in &card.questions {
            if question.state != work::QUESTION_OPEN {
                continue;
            }
            let (status, message) = if question.blocking {
                (
                    VERIFY_FAIL,
                    format!("blocking question open: {}", go_quote(&question.text)),
                )
            } else {
                (
                    VERIFY_INFO,
                    format!("question open: {}", go_quote(&question.text)),
                )
            };
            result.checks.push(VerifyCheck {
                status: status.to_string(),
                message,
            });
        }
    }
    result.manual = manual_items(&input.contract, &input.rules, &input.confirmed);
    result.checks.extend(manual_checks(
        &input.contract,
        &input.rules,
        &input.confirmed,
    ));
    if input.delivered {
        result.checks.push(VerifyCheck {
            status: VERIFY_INFO.to_string(),
            message: "Git: HEAD is on main".to_string(),
        });
    } else {
        result.checks.push(VerifyCheck {
            status: VERIFY_INFO.to_string(),
            message: format!(
                "Git: HEAD is not on main yet; the stored stage stays {}",
                card.stage
            ),
        });
    }

    result.passed = !result
        .checks
        .iter()
        .any(|check| check.status == VERIFY_FAIL);
    Ok(result)
}

/// Renders one line per check under a pass or fail heading.
pub fn encode_verify(result: &VerifyResult) -> String {
    let decision = if result.passed { "PASS" } else { "FAIL" };
    let mut out = format!("verify {}  {}\n", result.target, decision);
    for check in &result.checks {
        let label = if check.status == VERIFY_MANUAL {
            "check"
        } else {
            check.status.as_str()
        };
        out.push_str(&format!("{label:<5} {}\n", check.message));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_feature_reports_every_check() {
        let card = work::Card {
            slug: "git-import".to_string(),
            stage: work::STAGE_BUILD.to_string(),
            tasks: vec![
                work::Task {
                    number: 1,
                    title: "Read HEAD".to_string(),
                    state: work::TASK_DONE.to_string(),
                    ..work::Task::default()
                },
                work::Task {
                    number: 2,
                    title: "Stamp facts".to_string(),
                    state: work::TASK_OPEN.to_string(),
                    ..work::Task::default()
                },
                work::Task {
                    number: 3,
                    title: "Old idea".to_string(),
                    state: work::TASK_DROPPED.to_string(),
                    ..work::Task::default()
                },
                work::Task {
                    number: 4,
                    title: "Link commits".to_string(),
                    state: work::TASK_DONE.to_string(),
                    ..work::Task::default()
                },
            ],
            ..work::Card::default()
        };
        let input = VerifyInput {
            card: card.clone(),
            contract: vec![ReviewInvariant {
                text: "No network calls".to_string(),
                from: "core".to_string(),
                ..ReviewInvariant::default()
            }],
            ..VerifyInput::default()
        };
        let result = verify_feature(&input).expect("a card");
        assert!(!result.passed, "an open task should fail: {result:?}");
        let text = encode_verify(&result);
        assert_eq!(
            text,
            "verify git-import  FAIL\n\
             pass  T1 done: Read HEAD\n\
             fail  T2 is open: Stamp facts\n\
             info  T3 dropped: Old idea\n\
             pass  T4 done: Link commits\n\
             check ! No network calls (from core)\n\
             info  Git: HEAD is not on main yet; the stored stage stays build\n"
        );
        for want in [
            "verify git-import  FAIL",
            "fail  T2 is open",
            "pass  T4 done",
            "pass  T1 done",
            "check ! No network calls (from core)",
        ] {
            assert!(text.contains(want), "verify text misses {want:?}:\n{text}");
        }
        let one = verify_feature(&VerifyInput {
            card: card.clone(),
            task: 1,
            ..VerifyInput::default()
        })
        .expect("task 1");
        assert!(
            one.passed && one.target == "git-import#T1",
            "task verify = {one:?}"
        );
        assert!(
            verify_feature(&VerifyInput {
                card,
                task: 9,
                ..VerifyInput::default()
            })
            .is_err(),
            "unknown task should fail"
        );
    }

    #[test]
    fn verify_reports_questions_and_a_delivered_head() {
        let card = work::Card {
            slug: "git-import".to_string(),
            stage: work::STAGE_VERIFY.to_string(),
            tasks: vec![work::Task {
                number: 1,
                title: "Read HEAD".to_string(),
                state: work::TASK_DONE.to_string(),
                ..work::Task::default()
            }],
            questions: vec![
                work::Question {
                    text: "Include merge commits?".to_string(),
                    blocking: true,
                    state: work::QUESTION_OPEN.to_string(),
                    ..work::Question::default()
                },
                work::Question {
                    text: "Keep tags?".to_string(),
                    state: work::QUESTION_OPEN.to_string(),
                    ..work::Question::default()
                },
                work::Question {
                    text: "Answered?".to_string(),
                    state: work::QUESTION_ANSWERED.to_string(),
                    answer: "yes".to_string(),
                    ..work::Question::default()
                },
            ],
            ..work::Card::default()
        };
        let result = verify_feature(&VerifyInput {
            card: card.clone(),
            delivered: true,
            contract: vec![ReviewInvariant {
                text: "No network calls".to_string(),
                from: "card".to_string(),
                ..ReviewInvariant::default()
            }],
            ..VerifyInput::default()
        })
        .expect("a card");
        assert!(!result.passed, "a blocking question fails: {result:?}");
        let text = encode_verify(&result);
        assert!(
            text.contains("fail  blocking question open: \"Include merge commits?\""),
            "{text}"
        );
        assert!(
            text.contains("info  question open: \"Keep tags?\""),
            "{text}"
        );
        assert!(!text.contains("Answered?"), "{text}");
        assert!(text.contains("check ! No network calls\n"), "{text}");
        assert!(text.contains("info  Git: HEAD is on main\n"), "{text}");

        let result = verify_feature(&VerifyInput {
            card: work::Card {
                tasks: vec![],
                ..card
            },
            ..VerifyInput::default()
        })
        .expect("a card");
        assert!(!result.passed, "no tasks fails: {result:?}");
        let text = encode_verify(&result);
        assert!(
            text.contains("fail  no tasks: plan the feature before verifying it"),
            "{text}"
        );
        assert!(
            text.contains("info  Git: HEAD is not on main yet; the stored stage stays verify"),
            "{text}"
        );
    }

    #[test]
    fn a_rule_without_a_slug_is_named_by_its_id() {
        let rules = vec![VerifyRule {
            id: "rule-1".to_string(),
            slug: String::new(),
            statement: "Keep it local".to_string(),
            scope: "core".to_string(),
        }];
        let items = manual_items(&[], &rules, &[]);
        assert_eq!(items.len(), 1, "items = {items:?}");
        assert_eq!(items[0].label, "rule rule-1: Keep it local (from core)");
        assert_eq!(items[0].target, "rule rule-1");
        let result = verify_feature(&VerifyInput {
            card: work::Card {
                slug: "git-import".to_string(),
                stage: work::STAGE_VERIFY.to_string(),
                tasks: vec![work::Task {
                    number: 1,
                    title: "Read HEAD".to_string(),
                    state: work::TASK_DONE.to_string(),
                    ..work::Task::default()
                }],
                ..work::Card::default()
            },
            rules,
            ..VerifyInput::default()
        })
        .expect("a card");
        let text = encode_verify(&result);
        assert!(
            text.contains("check rule rule-1: Keep it local (from core)\n"),
            "{text}"
        );
    }

    #[test]
    fn verify_lists_binding_rules_and_confirmed_items_pass() {
        let card = work::Card {
            id: "c1".to_string(),
            slug: "git-import".to_string(),
            stage: work::STAGE_VERIFY.to_string(),
            tasks: vec![work::Task {
                number: 1,
                title: "Read HEAD".to_string(),
                state: work::TASK_DONE.to_string(),
                ..work::Task::default()
            }],
            ..work::Card::default()
        };
        let contract = vec![
            ReviewInvariant {
                id: "inv-1".to_string(),
                text: "No network calls".to_string(),
                from: "card".to_string(),
            },
            ReviewInvariant {
                id: "inv-2".to_string(),
                text: "Reads HEAD only".to_string(),
                from: "card".to_string(),
            },
        ];
        let rules = vec![
            VerifyRule {
                id: "r1".to_string(),
                slug: "local-only".to_string(),
                statement: "Never call the network".to_string(),
                scope: "core".to_string(),
            },
            VerifyRule {
                id: "r2".to_string(),
                slug: "typed-errors".to_string(),
                statement: "Handlers return typed errors".to_string(),
                scope: String::new(),
            },
        ];
        let confirmed = vec![work::Confirmation {
            card_id: "c1".to_string(),
            kind: work::CONFIRMATION_INVARIANT.to_string(),
            target_id: "inv-1".to_string(),
            confirmed_by: "codex".to_string(),
            ..work::Confirmation::default()
        }];
        let input = VerifyInput {
            card: card.clone(),
            delivered: true,
            contract: contract.clone(),
            rules: rules.clone(),
            confirmed: confirmed.clone(),
            ..VerifyInput::default()
        };
        let result = verify_feature(&input).expect("a card");
        assert!(result.passed, "manual checks do not fail: {result:?}");
        let text = encode_verify(&result);
        assert!(
            text.contains("pass  ! No network calls (confirmed by codex)\n"),
            "{text}"
        );
        assert!(text.contains("check ! Reads HEAD only\n"), "{text}");
        assert!(
            text.contains("check rule local-only: Never call the network (from core)\n"),
            "{text}"
        );
        assert!(
            text.contains("info  Constitution: 1 rule applies\n"),
            "a whole-project Rule is one info line: {text}"
        );
        assert!(
            !text.contains("typed-errors"),
            "a whole-project Rule is never a check: {text}"
        );
        assert_eq!(
            unconfirmed(&result),
            vec![
                "! Reads HEAD only".to_string(),
                "rule local-only: Never call the network (from core)".to_string(),
            ]
        );

        let rule_confirmed = verify_feature(&VerifyInput {
            card: card.clone(),
            delivered: true,
            contract: contract.clone(),
            rules: rules.clone(),
            confirmed: vec![work::Confirmation {
                card_id: "c1".to_string(),
                kind: work::CONFIRMATION_RULE.to_string(),
                target_id: "r1".to_string(),
                confirmed_by: "person".to_string(),
                ..work::Confirmation::default()
            }],
            ..VerifyInput::default()
        })
        .expect("a card");
        let text = encode_verify(&rule_confirmed);
        assert!(
            text.contains(
                "pass  rule local-only: Never call the network (from core) (confirmed by person)\n"
            ),
            "{text}"
        );
        assert!(
            text.contains("info  Constitution: 1 rule applies\n"),
            "confirming a whole-project Rule changes no check: {text}"
        );

        let all = manual_checks(
            &contract,
            &rules,
            &[
                confirmed[0].clone(),
                work::Confirmation {
                    card_id: "c1".to_string(),
                    kind: work::CONFIRMATION_INVARIANT.to_string(),
                    target_id: "inv-2".to_string(),
                    confirmed_by: String::new(),
                    ..work::Confirmation::default()
                },
                work::Confirmation {
                    card_id: "c1".to_string(),
                    kind: work::CONFIRMATION_RULE.to_string(),
                    target_id: "r1".to_string(),
                    confirmed_by: "codex".to_string(),
                    ..work::Confirmation::default()
                },
            ],
        );
        assert!(
            all.iter().all(|check| check.status != VERIFY_MANUAL),
            "every item is confirmed: {all:?}"
        );
        assert!(
            all.iter()
                .any(|check| check.message.contains("confirmed by codex")),
            "{all:?}"
        );
        assert!(
            all.iter()
                .any(|check| check.message.contains("confirmed by a person")),
            "an empty name reads as a person: {all:?}"
        );
    }
}
