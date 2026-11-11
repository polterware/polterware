//! Reads GitHub identities from remote URLs. A URL is parsed, never fetched.

/// The URL prefixes that name github.com; the path after one is `owner/repo`.
const GITHUB_PREFIXES: [&str; 4] = [
    "https://github.com/",
    "http://github.com/",
    "ssh://git@github.com/",
    "git://github.com/",
];

/// Reads `owner/repo` from a GitHub remote URL: `git@github.com:o/r`, one of the
/// `GITHUB_PREFIXES` URLs, or `https://user@github.com/o/r`. Any other host, and
/// any path that is not exactly two segments, gives None.
pub fn parse_github_remote(remote: &str) -> Option<String> {
    let path = github_path(remote.trim()).unwrap_or_default();
    let path = path.strip_suffix('/').unwrap_or(path);
    let path = path.strip_suffix(".git").unwrap_or(path);
    let (owner, repo) = path.split_once('/')?;
    if owner.is_empty() || repo.is_empty() || repo.contains('/') {
        return None;
    }
    Some(format!("{owner}/{repo}"))
}

/// The commit's page on GitHub.
pub fn github_commit_url(owner_repo: &str, sha: &str) -> String {
    format!("https://github.com/{owner_repo}/commit/{sha}")
}

/// The path after github.com in a remote URL, or None for another host.
fn github_path(remote: &str) -> Option<&str> {
    if let Some(path) = remote.strip_prefix("git@github.com:") {
        return Some(path);
    }
    GITHUB_PREFIXES
        .into_iter()
        .find_map(|prefix| remote.strip_prefix(prefix))
        .or_else(|| {
            // https://user@github.com/o/r
            let at = remote.find("@github.com/")?;
            let path = &remote[at + "@github.com/".len()..];
            remote.starts_with("https://").then_some(path)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_github_remote_cases() {
        let cases = [
            ("git@github.com:polterware/polterware.git", Some("polterware/polterware")),
            ("https://github.com/polterware/polterware", Some("polterware/polterware")),
            ("https://github.com/polterware/polterware.git", Some("polterware/polterware")),
            ("ssh://git@github.com/polterware/polterware.git", Some("polterware/polterware")),
            ("https://token@github.com/polterware/polterware/", Some("polterware/polterware")),
            ("git@gitlab.com:polterware/polterware.git", None),
            ("https://github.com/polterware", None),
            ("", None),
        ];
        for (remote, want) in cases {
            assert_eq!(
                parse_github_remote(remote),
                want.map(str::to_string),
                "remote {remote:?}",
            );
        }
    }
}
