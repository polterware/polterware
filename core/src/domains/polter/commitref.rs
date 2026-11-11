//! Port of `backend/domains/polter/commitref.go`: the task citations a commit
//! subject carries, and the tasks they name.

use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::domains::work;

/// One task citation found in a commit subject.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommitRef {
    pub slug: String,
    pub number: i32,
}

/// `polter: git-metadata-import#T2`.
static POLTER_COMMIT_REF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:^|[^A-Za-z0-9-])polter:[ \t]*([a-z0-9]+(?:-[a-z0-9]+)*)#T([1-9][0-9]{0,5})(?:[^0-9]|$)",
    )
    .expect("the polter commit reference pattern")
});

/// `git-metadata-import#T2`.
static HASH_COMMIT_REF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|[^A-Za-z0-9-])([a-z0-9]+(?:-[a-z0-9]+)*)#T([1-9][0-9]{0,5})(?:[^0-9]|$)")
        .expect("the hash commit reference pattern")
});

/// `git-metadata-import T2`.
static PLAIN_COMMIT_REF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|[^A-Za-z0-9-])([a-z0-9]+(?:-[a-z0-9]+)*)[ \t]+T([1-9][0-9]{0,5})(?:[^0-9]|$)")
        .expect("the plain commit reference pattern")
});

/// Finds explicit task citations in a commit subject.
///
/// The forms are `slug#T2`, `slug T2`, and `polter: slug#T2`. Order is first
/// appearance. A repeated citation is returned once.
pub fn parse_commit_refs(subject: &str) -> Vec<CommitRef> {
    let mut hits: Vec<(usize, CommitRef)> = Vec::new();
    for pattern in [&*POLTER_COMMIT_REF, &*HASH_COMMIT_REF, &*PLAIN_COMMIT_REF] {
        for found in pattern.captures_iter(subject) {
            let (Some(slug), Some(number)) = (found.get(1), found.get(2)) else {
                continue;
            };
            let Ok(number) = number.as_str().parse::<i32>() else {
                continue;
            };
            hits.push((
                slug.start(),
                CommitRef {
                    slug: slug.as_str().to_string(),
                    number,
                },
            ));
        }
    }
    hits.sort_by_key(|(at, _)| *at);

    let mut out = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for (_, reference) in hits {
        let key = format!("{}#{}", reference.slug, reference.number);
        if !seen.insert(key) {
            continue;
        }
        out.push(reference);
    }
    out
}

/// Resolves citations against active cards. A removed card, a dropped task, or
/// an unknown slug or number is ignored.
pub fn match_explicit_tasks(subject: &str, cards: &[work::Card]) -> Vec<work::Task> {
    let mut out: Vec<work::Task> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for reference in parse_commit_refs(subject) {
        let Some(task) = explicit_task(cards, &reference) else {
            continue;
        };
        if !seen.insert(task.id.clone()) {
            continue;
        }
        out.push(task);
    }
    out
}

fn explicit_task(cards: &[work::Card], reference: &CommitRef) -> Option<work::Task> {
    for card in cards {
        if card.status == work::CARD_REMOVED || card.slug != reference.slug {
            continue;
        }
        for task in &card.tasks {
            if task.number == reference.number && task.state != work::TASK_DROPPED {
                return Some(task.clone());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_commit_references() {
        let got = parse_commit_refs("fix git-metadata-import T2 and other-card T1");
        assert_eq!(
            got,
            vec![
                CommitRef {
                    slug: "git-metadata-import".into(),
                    number: 2
                },
                CommitRef {
                    slug: "other-card".into(),
                    number: 1
                },
            ],
            "plain references keep their order"
        );
        let hashed = parse_commit_refs("polter: git-metadata-import#T2");
        assert_eq!(
            hashed,
            vec![CommitRef {
                slug: "git-metadata-import".into(),
                number: 2
            }],
            "the polter form"
        );
        assert_eq!(
            parse_commit_refs("git-metadata-import T2 git-metadata-import T2").len(),
            1,
            "a repeated citation is returned once"
        );
        for subject in [
            "Git-Metadata-Import T2",
            "git-metadata-importT2",
            "git-metadata-import T0",
            "not a task",
        ] {
            assert!(
                parse_commit_refs(subject).is_empty(),
                "{subject} must carry no reference"
            );
        }
    }

    #[test]
    fn matches_explicit_tasks_against_active_cards() {
        let cards = vec![
            work::Card {
                slug: "git-metadata-import".into(),
                status: work::CARD_ACTIVE.into(),
                tasks: vec![
                    work::Task {
                        id: "t1".into(),
                        number: 1,
                        state: work::TASK_OPEN.into(),
                        ..work::Task::default()
                    },
                    work::Task {
                        id: "t2".into(),
                        number: 2,
                        state: work::TASK_DONE.into(),
                        ..work::Task::default()
                    },
                ],
                ..work::Card::default()
            },
            work::Card {
                slug: "dropped-card".into(),
                status: work::CARD_ACTIVE.into(),
                tasks: vec![work::Task {
                    id: "td".into(),
                    number: 1,
                    state: work::TASK_DROPPED.into(),
                    ..work::Task::default()
                }],
                ..work::Card::default()
            },
            work::Card {
                slug: "gone".into(),
                status: work::CARD_REMOVED.into(),
                tasks: vec![work::Task {
                    id: "tg".into(),
                    number: 1,
                    state: work::TASK_OPEN.into(),
                    ..work::Task::default()
                }],
                ..work::Card::default()
            },
        ];
        let matched = match_explicit_tasks(
            "polter: git-metadata-import#T2 dropped-card T1 gone T1 missing T3",
            &cards,
        );
        assert_eq!(matched.len(), 1, "{matched:?}");
        assert_eq!(matched[0].id, "t2", "the done task still counts");
    }
}
