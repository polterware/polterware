//! Task claims: the path globs two cards must not both hold.

use std::collections::HashMap;

use time::OffsetDateTime;

/// Reserves one task's write scope for one agent. There is at most one active
/// claim per task; release and done delete it, and the history stays in the
/// change operation log.
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    pub id: String,
    pub project_id: String,
    pub task_id: String,
    pub holder: String,
    /// The task's glob copied when the claim was taken, prefixed with the
    /// task's checkout (`api:src/auth/*`) when it is not the project root. An
    /// empty scope reserves only the task, not any files.
    pub scope: String,
    pub created_at: OffsetDateTime,
}

impl Default for Claim {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            task_id: String::new(),
            holder: String::new(),
            scope: String::new(),
            created_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

/// Writes a task glob with its checkout: `api:src/auth/*`. The project root has
/// no prefix, and an empty glob stays empty.
pub fn qualify_scope(repo: &str, glob: &str) -> String {
    let glob = glob.trim();
    if glob.is_empty() || repo.is_empty() {
        return glob.to_string();
    }
    format!("{repo}:{glob}")
}

/// Separates a `repo:glob` scope. A scope without a slug prefix belongs to the
/// project root, so claims written before repositories keep their meaning.
pub fn split_scope(scope: &str) -> (String, String) {
    let scope = scope.trim();
    if let Some(i) = scope.find(':')
        && i > 0
        && super::valid_slug(&scope[..i])
    {
        return (scope[..i].to_string(), scope[i + 1..].trim().to_string());
    }
    (String::new(), scope.to_string())
}

/// Reports whether some path is covered by both scopes. Scopes in different
/// checkouts never overlap. Inside one checkout, segments are split on `/`;
/// `*` and `?` stay inside one segment and `**` matches any number of
/// segments. A pattern also covers everything below a path it matches, so
/// `core/*` overlaps `core/domains/canon/*`. Character classes are
/// treated as any single character, which can only over-report overlap. An
/// empty pattern overlaps nothing.
pub fn scopes_overlap(a: &str, b: &str) -> bool {
    let (repo_a, a) = split_scope(a);
    let (repo_b, b) = split_scope(b);
    if repo_a != repo_b {
        return false;
    }
    let a = a.trim_matches('/');
    let b = b.trim_matches('/');
    if a.is_empty() || b.is_empty() {
        return false;
    }
    let a: Vec<&str> = a.split('/').collect();
    let b: Vec<&str> = b.split('/').collect();
    segments_overlap(&a, &b)
}

fn segments_overlap(a: &[&str], b: &[&str]) -> bool {
    // An exhausted pattern matched a directory that the other pattern can
    // still extend below.
    if a.is_empty() || b.is_empty() {
        return true;
    }
    if a[0] == "**" {
        return segments_overlap(&a[1..], b) || segments_overlap(a, &b[1..]);
    }
    if b[0] == "**" {
        return segments_overlap(a, &b[1..]) || segments_overlap(&a[1..], b);
    }
    segment_overlap(&glob_tokens(a[0]), &glob_tokens(b[0])) && segments_overlap(&a[1..], &b[1..])
}

/// One element of a segment pattern: `*`, `?`, or a literal rune.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GlobToken {
    kind: char,
    char: char,
}

fn glob_tokens(segment: &str) -> Vec<GlobToken> {
    let mut out: Vec<GlobToken> = Vec::new();
    let runes: Vec<char> = segment.chars().collect();
    let mut i = 0;
    while i < runes.len() {
        match runes[i] {
            '*' => {
                if out.last().map(|token| token.kind) != Some('*') {
                    out.push(GlobToken {
                        kind: '*',
                        char: '\0',
                    });
                }
            }
            '?' => out.push(GlobToken {
                kind: '?',
                char: '\0',
            }),
            '[' => {
                let mut j = i + 1;
                while j < runes.len() && runes[j] != ']' {
                    j += 1;
                }
                out.push(GlobToken {
                    kind: '?',
                    char: '\0',
                });
                i = j;
            }
            '\\' => {
                if i + 1 < runes.len() {
                    i += 1;
                }
                out.push(GlobToken {
                    kind: '\0',
                    char: runes[i],
                });
            }
            r => out.push(GlobToken {
                kind: '\0',
                char: r,
            }),
        }
        i += 1;
    }
    out
}

/// Reports whether one string matches both segment patterns.
fn segment_overlap(a: &[GlobToken], b: &[GlobToken]) -> bool {
    fn walk(
        a: &[GlobToken],
        b: &[GlobToken],
        i: usize,
        j: usize,
        memo: &mut HashMap<(usize, usize), bool>,
    ) -> bool {
        if let Some(done) = memo.get(&(i, j)) {
            return *done;
        }
        let result = if i == a.len() && j == b.len() {
            true
        } else if i < a.len() && a[i].kind == '*' {
            walk(a, b, i + 1, j, memo) || (j < b.len() && walk(a, b, i, j + 1, memo))
        } else if j < b.len() && b[j].kind == '*' {
            walk(a, b, i, j + 1, memo) || (i < a.len() && walk(a, b, i + 1, j, memo))
        } else if i == a.len() || j == b.len() {
            false
        } else {
            let same = a[i].kind == '?' || b[j].kind == '?' || a[i].char == b[j].char;
            same && walk(a, b, i + 1, j + 1, memo)
        };
        memo.insert((i, j), result);
        result
    }
    walk(a, b, 0, 0, &mut HashMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scopes_overlap_cases() {
        let cases = [
            ("backend/*", "backend/domains/canon/*", true),
            ("docs/*", "backend/*", false),
            ("backend/import/*", "backend/import/*", true),
            ("backend/import/*.go", "backend/import/git.go", true),
            ("backend/import/*.go", "backend/import/README.md", false),
            ("backend/**", "backend/a/b/c.go", true),
            ("**/*.sql", "backend/platform/db/0018.sql", true),
            // A literal path may be a folder; over-report on purpose.
            ("**/*.sql", "docs/spec.md", true),
            ("src/a?c/*", "src/abc/x", true),
            // Classes over-report on purpose.
            ("src/[ab]/*", "src/c/x", true),
            ("backend/import.go", "backend/export.go", false),
            ("", "backend/*", false),
            ("backend/*", "", false),
            ("/backend/*/", "backend/x", true),
        ];
        for (a, b, want) in cases {
            assert_eq!(scopes_overlap(a, b), want, "scopes_overlap({a:?}, {b:?})");
            assert_eq!(
                scopes_overlap(b, a),
                want,
                "scopes_overlap({b:?}, {a:?}) swapped"
            );
        }
    }

    #[test]
    fn scopes_overlap_across_repos() {
        let cases = [
            ("api:src/auth/*", "api:src/auth/login.go", true),
            ("api:src/auth/*", "web:src/auth/*", false),
            // No prefix is the project root.
            ("api:src/**", "src/auth/*", false),
            ("src/auth/*", "src/auth/x.go", true),
            ("api:src/*", "api:docs/*", false),
            ("api:", "api:src/*", false),
        ];
        for (a, b, want) in cases {
            assert_eq!(scopes_overlap(a, b), want, "scopes_overlap({a:?}, {b:?})");
            assert_eq!(
                scopes_overlap(b, a),
                want,
                "scopes_overlap({b:?}, {a:?}) swapped"
            );
        }
    }

    #[test]
    fn qualify_and_split_scope() {
        assert_eq!(qualify_scope("api", "src/auth/*"), "api:src/auth/*");
        assert_eq!(qualify_scope("", "src/*"), "src/*");
        assert_eq!(qualify_scope("api", ""), "");

        let (repo, glob) = split_scope("api:src/auth/*");
        assert_eq!((repo.as_str(), glob.as_str()), ("api", "src/auth/*"));
        let (repo, glob) = split_scope("src/auth/*");
        assert_eq!((repo.as_str(), glob.as_str()), ("", "src/auth/*"));
    }
}
