//! Port of `backend/compositions/wedge/path.go`.

#[allow(unused_imports)]
use super::*;

/// Expands a leading ~ to the user home directory.
pub fn expand_path(path: &str) -> String {
    let path = path.trim();
    if path.is_empty() {
        return String::new();
    }
    if path == "~" {
        return home_dir().unwrap_or_default();
    }
    let separator = std::path::MAIN_SEPARATOR;
    let rest = path
        .strip_prefix("~/")
        .or_else(|| path.strip_prefix(&format!("~{separator}")));
    if let Some(rest) = rest {
        let Some(home) = home_dir() else {
            return path.to_string();
        };
        return join_path(&home, rest);
    }
    path.to_string()
}

/// Go's `os.UserHomeDir`: the home directory of the current user.
fn home_dir() -> Option<String> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|home| home.to_string_lossy().into_owned())
        .filter(|home| !home.is_empty())
}

/// Go's `filepath.Clean`: lexical, with no filesystem access.
pub fn clean_path(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let rooted = path.starts_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.last().is_some_and(|last| *last != "..") {
                    parts.pop();
                } else if !rooted {
                    parts.push("..");
                }
            }
            other => parts.push(other),
        }
    }
    let joined = parts.join("/");
    if rooted {
        format!("/{joined}")
    } else if joined.is_empty() {
        ".".to_string()
    } else {
        joined
    }
}

/// Go's `filepath.Abs`: clean, joined to the working directory when relative.
pub fn absolute_path(path: &str) -> String {
    let path = path.trim();
    if path.starts_with('/') {
        return clean_path(path);
    }
    match std::env::current_dir() {
        Ok(cwd) => clean_path(&format!("{}/{}", cwd.to_string_lossy(), path)),
        Err(_) => clean_path(path),
    }
}

/// Go's `filepath.Base`.
pub fn path_base(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return "/".to_string();
    }
    match trimmed.rfind('/') {
        Some(index) => trimmed[index + 1..].to_string(),
        None => trimmed.to_string(),
    }
}

/// Go's `filepath.Rel`: the relative path of `target` from `root`, with `None`
/// for the cases it errors on (mixing an absolute and a relative path).
pub fn relative_path(root: &str, target: &str) -> Option<String> {
    let root_parts: Vec<&str> = root.split('/').filter(|part| !part.is_empty()).collect();
    let target_parts: Vec<&str> = target.split('/').filter(|part| !part.is_empty()).collect();
    if root.starts_with('/') != target.starts_with('/') {
        return None;
    }
    let mut common = 0;
    while common < root_parts.len()
        && common < target_parts.len()
        && root_parts[common] == target_parts[common]
    {
        common += 1;
    }
    let mut parts: Vec<&str> = vec![".."; root_parts.len() - common];
    parts.extend_from_slice(&target_parts[common..]);
    if parts.is_empty() {
        return Some(".".to_string());
    }
    Some(parts.join("/"))
}

/// Go's `filepath.ToSlash`.
pub fn slash_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Go's `filepath.Join` for two parts: the cleaned join, with `..` resolved.
fn join_path(base: &str, rest: &str) -> String {
    let mut out: Vec<&str> = base.split('/').collect();
    if out.is_empty() {
        out.push("");
    }
    for part in rest.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if out.len() > 1 {
                    out.pop();
                }
            }
            other => out.push(other),
        }
    }
    out.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_a_leading_home() {
        let home = home_dir().expect("the test machine has a home");
        assert_eq!(expand_path(""), "");
        assert_eq!(expand_path("   "), "");
        assert_eq!(expand_path("~"), home);
        assert_eq!(
            expand_path("~/code/polterware"),
            format!("{home}/code/polterware")
        );
        assert_eq!(
            expand_path("~/code/../code/polterware"),
            format!("{home}/code/polterware")
        );
        assert_eq!(expand_path("/abs/path"), "/abs/path");
        assert_eq!(expand_path("relative/path"), "relative/path");
        assert_eq!(
            expand_path("~someone/else"),
            "~someone/else",
            "only a leading ~/ expands"
        );
    }

    #[test]
    fn paths_clean_join_and_relativize_like_go() {
        assert_eq!(clean_path(""), ".");
        assert_eq!(clean_path("a//b/./c"), "a/b/c");
        assert_eq!(clean_path("a/b/../c"), "a/c");
        assert_eq!(clean_path("/a/../.."), "/");
        assert_eq!(clean_path("../a"), "../a");
        assert_eq!(path_base(""), ".");
        assert_eq!(path_base("/"), "/");
        assert_eq!(path_base("/a/b/"), "b");
        assert_eq!(path_base("b"), "b");
        assert_eq!(slash_path("a\\b"), "a/b");
        assert_eq!(relative_path("/a/b", "/a/b/c"), Some("c".to_string()));
        assert_eq!(relative_path("/a/b", "/a/b"), Some(".".to_string()));
        assert_eq!(relative_path("/a/b", "/a/c"), Some("../c".to_string()));
        assert_eq!(
            relative_path("/a", "a"),
            None,
            "an absolute and a relative path do not relate"
        );
        let abs = absolute_path("relative/path");
        assert!(abs.starts_with('/'), "absolute_path = {abs}");
        assert!(abs.ends_with("/relative/path"), "absolute_path = {abs}");
        assert_eq!(absolute_path("/a/../b"), "/b");
    }
}
