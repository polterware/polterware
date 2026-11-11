//! Port of `backend/domains/polter/coord.go`: task references
//! (`slug#T2` / `slug T2`) and the coordination text agents write.

use std::sync::LazyLock;

use regex::Regex;

use super::commitref::CommitRef;
use super::model::{COORD_CLAIM, COORD_DONE, COORD_RELEASE, go_quote};

static TASK_REF_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([a-z0-9]+(?:-[a-z0-9]+)*)(?:#|[\t\n\f\r ]+)[Tt]([1-9][0-9]{0,5})$")
        .expect("the task reference pattern")
});

/// Reads a task id written as `slug#T2` or `slug T2` (the same citation commit
/// subjects use).
pub fn parse_task_ref(text: &str) -> Result<CommitRef, String> {
    let trimmed = text.trim();
    let Some(m) = TASK_REF_PATTERN.captures(trimmed) else {
        return Err(format!(
            "invalid task {}: use slug#T2 or slug T2",
            go_quote(trimmed)
        ));
    };
    let number = m
        .get(2)
        .expect("group 2")
        .as_str()
        .parse::<i32>()
        .unwrap_or(0);
    Ok(CommitRef {
        slug: m.get(1).expect("group 1").as_str().to_string(),
        number,
    })
}

/// The minimal Polter text for one coordination line on an existing card, with
/// a generated summary and rationale.
pub fn coord_text(action: &str, reference: &CommitRef, holder: &str) -> Result<String, String> {
    let slug = &reference.slug;
    let number = reference.number;
    let (summary, rationale) = match action {
        COORD_CLAIM => (
            format!("Claim {slug} T{number} for {holder}"),
            "Reserve the task scope before editing so other agents do not write the same files",
        ),
        COORD_DONE => (
            format!("Mark {slug} T{number} done by {holder}"),
            "The agent finished the task",
        ),
        COORD_RELEASE => (
            format!("Release {slug} T{number} held by {holder}"),
            "The agent stopped working on the task without finishing it",
        ),
        _ => {
            return Err(format!(
                "unknown action {}: use claim, done, or release",
                go_quote(action)
            ));
        }
    };
    Ok(format!(
        "---\npolter: 1\nsummary: {summary}\nrationale: {rationale}\n---\n\n## {slug}\n{action}: T{number}\n"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_task_references() {
        let reference = parse_task_ref(" git-metadata-import#T2 ").expect("a hash reference");
        assert_eq!(reference.slug, "git-metadata-import");
        assert_eq!(reference.number, 2);
        let reference = parse_task_ref("git-metadata-import T3").expect("a space reference");
        assert_eq!(reference.number, 3);
        for text in [
            "git-metadata-import",
            "git-metadata-import T0",
            "Git-Import T2",
            "#T2",
        ] {
            assert!(parse_task_ref(text).is_err(), "{text} must be refused");
        }
    }

    #[test]
    fn writes_coordination_text() {
        let reference = CommitRef {
            slug: "git-import".into(),
            number: 2,
        };
        let text = coord_text(COORD_CLAIM, &reference, "codex").expect("a claim");
        assert_eq!(
            text,
            "---\npolter: 1\nsummary: Claim git-import T2 for codex\nrationale: Reserve the task scope before editing so other agents do not write the same files\n---\n\n## git-import\nclaim: T2\n"
        );
        let text = coord_text(COORD_DONE, &reference, "codex").expect("a done");
        assert!(text.contains("done: T2\n"), "{text}");
        assert!(
            coord_text("steal", &reference, "codex").is_err(),
            "an unknown action fails"
        );
    }
}
