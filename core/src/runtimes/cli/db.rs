//! The database file the CLI uses, and its path helpers.
//! needs: the default database and absolute, cleaned paths.

use std::env;
use std::io;
use std::path::Path;

/// The database shared with the Desktop: `<user config dir>/polterware/local.db`
/// (the Desktop's app data directory on macOS, Linux, and Windows).
pub fn default_db() -> String {
    let dir = user_config_dir().unwrap_or_else(|| {
        let home = home_dir().unwrap_or_default();
        path_join(&home, ".config")
    });
    path_join(&path_join(&dir, "polterware"), "local.db")
}

/// Go's `os.UserConfigDir`.
pub fn user_config_dir() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        // macOS keeps app data in Application Support and ignores XDG.
        let home = home_dir()?;
        Some(path_join(&home, "Library/Application Support"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
            let xdg = xdg.to_string_lossy().into_owned();
            if xdg.starts_with('/') {
                return Some(clean(&xdg));
            }
        }
        let home = home_dir()?;
        Some(path_join(&home, ".config"))
    }
}

/// Go's `os.UserHomeDir`.
pub fn home_dir() -> Option<String> {
    env::var_os("HOME")
        .map(|home| home.to_string_lossy().into_owned())
        .filter(|home| !home.is_empty())
}

/// Go's `filepath.Abs`: an absolute path cleaned, or the working directory
/// joined with `path` and cleaned. It never resolves symlinks.
pub fn abs(path: &str) -> io::Result<String> {
    if Path::new(path).is_absolute() {
        return Ok(clean(path));
    }
    let cwd = env::current_dir()?;
    Ok(clean(&format!("{}/{}", cwd.to_string_lossy(), path)))
}

/// Go's `filepath.Join` for the two-segment joins this package makes.
pub fn path_join(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        return clean(name);
    }
    clean(&format!("{dir}/{name}"))
}

/// Go's `filepath.Clean`: lexical only, so `a/../b` becomes `b` without
/// touching the disk.
pub fn clean(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let absolute = path.starts_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                if matches!(parts.last(), Some(last) if *last != "..") {
                    parts.pop();
                } else if !absolute {
                    parts.push("..");
                }
            }
            part => parts.push(part),
        }
    }
    let joined = parts.join("/");
    match (absolute, joined.is_empty()) {
        (true, true) => "/".to_string(),
        (true, false) => format!("/{joined}"),
        (false, true) => ".".to_string(),
        (false, false) => joined,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn the_default_database_sits_in_the_config_directory() {
        let db = default_db();
        assert!(db.ends_with("/polterware/local.db"), "{db}");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_uses_application_support() {
        let dir = user_config_dir().expect("config dir");
        assert!(dir.ends_with("Library/Application Support"), "{dir}");
    }

    #[test]
    fn paths_are_cleaned_like_go() {
        assert_eq!(clean(""), ".");
        assert_eq!(clean("."), ".");
        assert_eq!(clean("/"), "/");
        assert_eq!(clean("/tmp//a/./b/../c"), "/tmp/a/c");
        assert_eq!(clean("../a"), "../a");
        assert_eq!(clean("a/../../b"), "../b");
        assert_eq!(path_join("", ".config"), ".config");
        assert_eq!(path_join("/tmp", "local.db"), "/tmp/local.db");
        assert_eq!(path_join("", "local.db"), "local.db");
    }

    #[test]
    fn absolute_paths_are_resolved_against_the_working_directory() {
        assert_eq!(abs("/tmp/./x.db").expect("abs"), "/tmp/x.db");
        let cwd = env::current_dir().expect("cwd");
        assert_eq!(
            abs("data/local.db").expect("abs"),
            clean(&format!("{}/data/local.db", cwd.to_string_lossy()))
        );
    }
}
