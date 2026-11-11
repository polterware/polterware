//! Port of `backend/domains/polter/verify.go`.
//!
//! Verify reports whether a card (or one task) is finished. It is a read: it
//! never moves the stored stage.

use crate::domains::work;

use super::*;

/// The verify check statuses. Only fail makes the result fail; manual lines are
/// contract items a person or agent confirms by hand.
pub const VERIFY_PASS: &str = "pass";
pub const VERIFY_FAIL: &str = "fail";
pub const VERIFY_INFO: &str = "info";
pub const VERIFY_MANUAL: &str = "manual";

/// One active invariant inherited from a node. Go declares this in
/// `feature.go`, next to the review it feeds; `verify` reads it as the card's
/// contract.
// TODO(dedupe): shares its name with the `feature` submodule's type.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReviewInvariant {
    pub text: String,
    /// The node path the invariant came from, or `card` for the card's own.
    pub from: String,
}

/// What verify reads for one card, or one task when `task > 0`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VerifyInput {
    pub card: work::Card,
    pub task: i32,
    /// True when HEAD is an ancestor of local main or master.
    pub delivered: bool,
    /// The card's own and inherited invariants.
    pub contract: Vec<ReviewInvariant>,
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
    for invariant in &input.contract {
        let from = if invariant.from.is_empty() || invariant.from == "card" {
            String::new()
        } else {
            format!(" (from {})", invariant.from)
        };
        result.checks.push(VerifyCheck {
            status: VERIFY_MANUAL.to_string(),
            message: format!("! {}{from}", invariant.text),
        });
    }
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
}
