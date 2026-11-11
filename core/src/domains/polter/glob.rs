//! Go's `path.Match`, ported: the syntax check the decoder runs on a task
//! `scope` glob.
//!
//! Only the pattern side is load-bearing here (the decoder validates a glob
//! against an empty name), but the matcher is complete so claims and search
//! can reuse it.

/// The pattern is malformed: Go's `path.ErrBadPattern`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BadPattern;

impl std::fmt::Display for BadPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("syntax error in pattern")
    }
}

impl std::error::Error for BadPattern {}

/// Reports whether `pattern` is a well-formed `path.Match` pattern.
pub fn valid_pattern(pattern: &str) -> bool {
    match_pattern(pattern, "").is_ok()
}

/// Reports whether a slash separated `name` matches `pattern`.
///
/// `*` matches any run of non-`/` characters, `?` one non-`/` character,
/// `[...]` one character of a class, and `\` escapes the next character.
pub fn match_pattern(pattern: &str, name: &str) -> Result<bool, BadPattern> {
    let pat: Vec<char> = pattern.chars().collect();
    let mut name: Vec<char> = name.chars().collect();
    let mut pi = 0usize;
    'pattern: while pi < pat.len() {
        let (star, chunk) = scan_chunk(&pat, &mut pi);
        if star && chunk.is_empty() {
            // A trailing star matches the rest of the name unless it holds a
            // slash.
            return Ok(!name.contains(&'/'));
        }
        let (rest, ok) = match_chunk(&chunk, &name)?;
        if ok && (rest.is_empty() || pi < pat.len()) {
            name = rest;
            continue;
        }
        if star {
            let mut i = 0usize;
            while i < name.len() && name[i] != '/' {
                let (rest, ok) = match_chunk(&chunk, &name[i + 1..])?;
                if ok {
                    // The last chunk must consume the whole name.
                    if pi == pat.len() && !rest.is_empty() {
                        i += 1;
                        continue;
                    }
                    name = rest;
                    continue 'pattern;
                }
                i += 1;
            }
        }
        // Before reporting no match, the rest of the pattern must still be
        // well formed.
        while pi < pat.len() {
            let (_, chunk) = scan_chunk(&pat, &mut pi);
            match_chunk(&chunk, &[])?;
        }
        return Ok(false);
    }
    Ok(name.is_empty())
}

/// Reads one chunk: leading stars, then the text up to the next star outside a
/// class. An escaped character never opens a class or splits the chunk.
fn scan_chunk(pat: &[char], pi: &mut usize) -> (bool, Vec<char>) {
    let mut star = false;
    while *pi < pat.len() && pat[*pi] == '*' {
        star = true;
        *pi += 1;
    }
    let start = *pi;
    let mut in_range = false;
    let mut i = start;
    while i < pat.len() {
        match pat[i] {
            '\\' => {
                if i + 1 < pat.len() {
                    i += 1;
                }
            }
            '[' => in_range = true,
            ']' => in_range = false,
            '*' => {
                if !in_range {
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }
    *pi = i;
    (star, pat[start..i].to_vec())
}

/// Matches one chunk at the start of `s`, answering the rest of `s` and
/// whether it matched.
fn match_chunk(chunk: &[char], s: &[char]) -> Result<(Vec<char>, bool), BadPattern> {
    let mut s: Vec<char> = s.to_vec();
    let mut failed = false;
    let mut i = 0usize;
    while i < chunk.len() {
        if !failed && s.is_empty() {
            failed = true;
        }
        match chunk[i] {
            '[' => {
                let mut r = '\u{0}';
                if !failed {
                    r = s[0];
                    s.remove(0);
                }
                i += 1;
                let mut negated = false;
                if i < chunk.len() && chunk[i] == '^' {
                    negated = true;
                    i += 1;
                }
                let mut matched = false;
                let mut ranges = 0;
                loop {
                    if i < chunk.len() && chunk[i] == ']' && ranges > 0 {
                        i += 1;
                        break;
                    }
                    let (lo, next) = get_esc(chunk, i)?;
                    i = next;
                    let mut hi = lo;
                    if i < chunk.len() && chunk[i] == '-' {
                        let (high, next) = get_esc(chunk, i + 1)?;
                        hi = high;
                        i = next;
                    }
                    if lo <= r && r <= hi {
                        matched = true;
                    }
                    ranges += 1;
                }
                if matched == negated {
                    failed = true;
                }
            }
            '?' => {
                if !failed {
                    if s[0] == '/' {
                        failed = true;
                    }
                    s.remove(0);
                }
                i += 1;
            }
            '\\' => {
                i += 1;
                if i == chunk.len() {
                    return Err(BadPattern);
                }
                if !failed {
                    if chunk[i] != s[0] {
                        failed = true;
                    }
                    s.remove(0);
                }
                i += 1;
            }
            c => {
                if !failed {
                    if c != s[0] {
                        failed = true;
                    }
                    s.remove(0);
                }
                i += 1;
            }
        }
    }
    Ok((s, !failed))
}

/// Reads one character of a class, honouring an escape. A class needs at least
/// one character after the one it reads, so an unclosed class is an error.
fn get_esc(chunk: &[char], at: usize) -> Result<(char, usize), BadPattern> {
    if at >= chunk.len() || chunk[at] == '-' || chunk[at] == ']' {
        return Err(BadPattern);
    }
    let mut i = at;
    if chunk[i] == '\\' {
        i += 1;
        if i >= chunk.len() {
            return Err(BadPattern);
        }
    }
    let r = chunk[i];
    i += 1;
    if i >= chunk.len() {
        return Err(BadPattern);
    }
    Ok((r, i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_scope_globs_the_decoder_takes() {
        for pattern in [
            "src/auth/*",
            "backend/domains/canon/*",
            "api:src/auth/*",
            "*",
            "a/**",
            "[a-z]x",
            "file?.go",
            "a\\*b",
        ] {
            assert!(valid_pattern(pattern), "{pattern} must be well formed");
        }
    }

    #[test]
    fn refuses_malformed_globs() {
        for pattern in ["src/[", "a\\", "[a-", "[]", "[a"] {
            assert!(!valid_pattern(pattern), "{pattern} must be malformed");
        }
    }

    #[test]
    fn matches_go_semantics() {
        assert!(match_pattern("src/*", "src/auth").expect("well formed"));
        assert!(!match_pattern("src/*", "src/auth/x").expect("well formed"));
        assert!(match_pattern("a?c", "abc").expect("well formed"));
        assert!(!match_pattern("a?c", "a/c").expect("well formed"));
        assert!(match_pattern("[a-c]x", "bx").expect("well formed"));
        assert!(!match_pattern("[^a-c]x", "bx").expect("well formed"));
        assert!(match_pattern("*", "anything/at/all").is_ok());
    }
}
