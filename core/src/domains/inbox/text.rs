//! The Inbox text `polterware status` prints.

use super::{ACTION_DELEGATE, Item, Result, STATE_DEFERRED, STATE_OPEN};

/// Caps the blocking items `polterware status` prints.
const STATUS_BLOCKING_LIMIT: usize = 5;

/// The Inbox part of `polterware status`: the count, the blocking items, and the
/// items a person routed to agents. It is empty when nothing is waiting.
pub fn status_lines(r: &Result) -> String {
    let mut out = String::new();
    let open = r.open();
    if open > 0 {
        out.push_str(&format!("inbox: {open} open ({} blocking)\n", r.blocking()));
        let mut shown = 0;
        for item in &r.items {
            if item.state != STATE_OPEN || !item.blocking {
                continue;
            }
            if shown == STATUS_BLOCKING_LIMIT {
                out.push_str("! … more in polterware inbox\n");
                break;
            }
            out.push_str(&format!(
                "! {} {} {}{}\n",
                item.id,
                item.kind,
                item.title,
                where_suffix(item)
            ));
            shown += 1;
        }
    }
    for routed in &r.routed {
        let item = &routed.item;
        if routed.action == ACTION_DELEGATE {
            out.push_str(&format!(
                "delegated: {}{}: choose a default and record it with `? {} -> answer`\n",
                item.title,
                where_suffix(item),
                item.question_text
            ));
        }
    }
    out
}

/// Renders `polterware inbox`: one line per item, open first, then the deferred
/// ones.
pub fn list(r: &Result) -> String {
    if r.items.is_empty() {
        return "inbox empty\n".to_string();
    }
    let width = r
        .items
        .iter()
        .map(|item| item.kind.chars().count())
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    let mut deferred = false;
    for item in &r.items {
        if item.state == STATE_DEFERRED && !deferred {
            out.push_str("deferred:\n");
            deferred = true;
        }
        let mark = if item.blocking && item.state == STATE_OPEN {
            "!"
        } else {
            " "
        };
        let mut title = item.title.clone();
        if !item.options.is_empty() {
            title += &format!(" ({})", item.options.join(" | "));
        }
        out.push_str(&format!(
            "{} {} {:<width$}  {}{}\n",
            item.id,
            mark,
            item.kind,
            title,
            where_suffix(item),
            width = width,
        ));
    }
    out
}

fn where_suffix(item: &Item) -> String {
    if item.where_.is_empty() {
        return String::new();
    }
    format!("  {}", item.where_)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::inbox::{
        ACTION_DEFER, ACTION_DELEGATE, Conflict, Disposition, KIND_DECISION, Question, derive,
    };
    use pretty_assertions::assert_eq;
    use time::macros::datetime;

    /// The exact bytes the status text must keep for these inputs: the field
    /// order and spacing are part of what agents parse.
    #[test]
    fn the_text_matches_the_go_core_on_a_full_inbox() {
        let conflict = Conflict {
            slug: "drop-columns".to_string(),
            request: "Drop the legacy columns".to_string(),
            against: "Rule: migrations are additive".to_string(),
            card_slug: "schema-cleanup".to_string(),
            options: vec!["additive only".to_string(), "drop it".to_string()],
            details: [("rule".to_string(), "migrations-additive".to_string())].into(),
        };
        let questions = vec![
            Question {
                id: "q1".to_string(),
                card_slug: "a".to_string(),
                text: "Later?".to_string(),
                ..Question::default()
            },
            Question {
                id: "q2".to_string(),
                card_slug: "a".to_string(),
                text: "Now?".to_string(),
                blocking: true,
                options: vec!["yes".to_string(), "no".to_string()],
            },
            Question {
                id: "q3".to_string(),
                card_slug: "a".to_string(),
                text: "Defaults?".to_string(),
                ..Question::default()
            },
        ];
        let t0 = datetime!(2026-09-28 12:00:00 UTC);
        let dispositions = vec![
            Disposition {
                key: "decision:q1".to_string(),
                action: ACTION_DEFER.to_string(),
                created_at: t0,
                ..Disposition::default()
            },
            Disposition {
                key: "decision:q3".to_string(),
                action: ACTION_DELEGATE.to_string(),
                created_at: t0,
                ..Disposition::default()
            },
        ];
        let result = derive(std::slice::from_ref(&conflict), &questions, &dispositions);

        assert_eq!(
            list(&result),
            "ic78c873a ! conflict  Drop the legacy columns (additive only | drop it)  schema-cleanup\n\
             i6df375dc ! decision  Now? (yes | no)  a\n\
             deferred:\n\
             ic4c66dc4   decision  Later?  a\n"
        );
        assert_eq!(
            status_lines(&result),
            "inbox: 2 open (2 blocking)\n\
             ! ic78c873a conflict Drop the legacy columns  schema-cleanup\n\
             ! i6df375dc decision Now?  a\n\
             delegated: Defaults?  a: choose a default and record it with `? Defaults? -> answer`\n"
        );
        assert_eq!(result.open(), 2);
        assert_eq!(result.blocking(), 2);
        assert_eq!(result.routed.len(), 1);
    }

    #[test]
    fn status_lines_show_the_count_blocking_items_and_routes() {
        let result = derive(
            &[],
            &[
                Question {
                    id: "q1".to_string(),
                    card_slug: "a".to_string(),
                    text: "Pick one?".to_string(),
                    blocking: true,
                    ..Question::default()
                },
                Question {
                    id: "q2".to_string(),
                    card_slug: "a".to_string(),
                    text: "Later?".to_string(),
                    ..Question::default()
                },
            ],
            &[Disposition {
                key: "decision:q2".to_string(),
                action: ACTION_DELEGATE.to_string(),
                ..Disposition::default()
            }],
        );
        let lines = status_lines(&result);
        for want in [
            "inbox: 1 open (1 blocking)\n",
            "decision Pick one?  a\n",
            "delegated: Later?  a: choose a default and record it with `? Later? -> answer`\n",
        ] {
            assert!(
                lines.contains(want),
                "status lines missing {want:?}:\n{lines}"
            );
        }
        assert_eq!(status_lines(&Result::default()), "");
        assert_eq!(list(&Result::default()), "inbox empty\n");
    }

    #[test]
    fn status_caps_the_blocking_items_it_prints() {
        let questions: Vec<Question> = (0..7)
            .map(|n| Question {
                id: format!("q{n}"),
                card_slug: "a".to_string(),
                text: format!("Question {n}?"),
                blocking: true,
                ..Question::default()
            })
            .collect();
        let result = derive(&[], &questions, &[]);
        let lines = status_lines(&result);
        assert_eq!(lines.matches("! i").count(), STATUS_BLOCKING_LIMIT);
        assert!(lines.ends_with("! … more in polterware inbox\n"), "{lines}");
    }

    #[test]
    fn list_groups_deferred_items_last() {
        let result = derive(
            &[],
            &[
                Question {
                    id: "q1".to_string(),
                    card_slug: "a".to_string(),
                    text: "Later?".to_string(),
                    ..Question::default()
                },
                Question {
                    id: "q2".to_string(),
                    card_slug: "a".to_string(),
                    text: "Now?".to_string(),
                    blocking: true,
                    options: vec!["yes".to_string(), "no".to_string()],
                },
            ],
            &[Disposition {
                key: "decision:q1".to_string(),
                action: "defer".to_string(),
                ..Disposition::default()
            }],
        );
        let text = list(&result);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(
            lines[0].split(' ').next(),
            Some(result.items[0].id.as_str())
        );
        assert!(lines[0].contains(KIND_DECISION), "{text}");
        assert!(lines[0].ends_with("Now? (yes | no)  a"), "{text}");
        assert_eq!(lines[1], "deferred:");
        assert!(lines[2].ends_with("Later?  a"), "{text}");
    }
}
