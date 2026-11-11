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
}
