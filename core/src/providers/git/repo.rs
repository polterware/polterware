//! Reads a work tree and its commits through the `git` CLI. Nothing here
//! contacts a remote.

use std::collections::HashMap;
use std::time::Duration;

use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::providers::exec;
use crate::providers::git::remote::parse_github_remote;

/// How many first-parent commits a metadata read keeps.
pub const RECENT_COMMIT_LIMIT: usize = 20;

/// The deadline of each read that the Go port bounds with a five-second context.
const READ_DEADLINE: Duration = Duration::from_secs(5);

/// One local commit: its full sha and subject. It is not a remote object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub sha: String,
    pub subject: String,
}

/// A local read of one work tree. It never contacts a remote.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Observation {
    pub available: bool,
    pub branch: String,
    pub files: Vec<String>,
    /// True when HEAD is an ancestor of local main or master.
    pub delivered: bool,
}

/// HEAD, the current branch, and recent first-parent commits. `available` is
/// false outside a work tree and when HEAD cannot be read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Metadata {
    pub available: bool,
    pub sha: String,
    pub branch: String,
    pub commits: Vec<Commit>,
}

/// One commit's full sha, subject, and committer time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitDetail {
    pub sha: String,
    pub subject: String,
    /// None when git's committer time is not RFC 3339.
    pub committed_at: Option<OffsetDateTime>,
}

/// A work tree that git runs in, with the deadline each of its runs gets.
#[derive(Clone, Copy)]
struct Git<'a> {
    root: &'a str,
    deadline: Duration,
}

impl<'a> Git<'a> {
    fn new(root: &'a str, deadline: Duration) -> Self {
        Self { root, deadline }
    }

    /// The trimmed stdout of `git args`, or None when the run fails or outlives
    /// its deadline.
    fn out(self, args: &[&str]) -> Option<String> {
        let owned: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
        let res = exec::run_with_timeout("git", &owned, self.root, Some(self.deadline));
        // exec keeps exit code 0 for a run it killed at the deadline, so the error
        // is what marks it as failed. Go reads that run as exit code -1.
        match res.err {
            Some(_) => None,
            None => Some(res.stdout.trim().to_string()),
        }
    }

    /// The lines of the output of `git args`; none when the output is empty or
    /// the run fails.
    fn lines(self, args: &[&str]) -> Vec<String> {
        match self.out(args) {
            Some(out) if !out.is_empty() => out.split('\n').map(str::to_string).collect(),
            _ => Vec::new(),
        }
    }
}

/// Runs `git status --porcelain` in `cwd`. A failed run answers with git's
/// stderr, or with the run's own error when git wrote nothing.
pub fn status(cwd: &str) -> Result<String, exec::Error> {
    let res = exec::run("git", &["status", "--porcelain"].map(String::from), cwd);
    match res.err {
        None => Ok(res.stdout),
        Some(err) if res.stderr.is_empty() => Err(err),
        Some(_) => Err(exec::Error(res.stderr)),
    }
}

/// Reads the branch, the changed paths, and whether HEAD is an ancestor of the
/// delivery base. A directory that is not a work tree returns a zero Observation.
pub fn observe(root: &str) -> Observation {
    let git = Git::new(root, READ_DEADLINE);
    if root.trim().is_empty() || !is_work_tree(git) {
        return Observation::default();
    }
    let mut obs = Observation {
        available: true,
        branch: git.out(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default(),
        ..Observation::default()
    };
    let mut files: Vec<String> = Vec::new();
    if let Some(main) = delivery_base(root) {
        let ancestor = ["merge-base", "--is-ancestor", "HEAD", main.as_str()];
        obs.delivered = git.out(&ancestor).is_some();
        files.extend(paths_since_base(git, &main));
    }
    files.extend(git.lines(&["diff", "--name-only"]));
    files.extend(git.lines(&["diff", "--name-only", "--cached"]));
    files.extend(git.lines(&["ls-files", "--others", "--exclude-standard"]));
    obs.files = unique_paths(files);
    obs
}

/// Reads HEAD, the branch name, and up to `RECENT_COMMIT_LIMIT` first-parent
/// commits. A missing work tree, or a repository with no commits, returns a zero
/// Metadata. A failed commit log still returns HEAD.
pub fn read_metadata(root: &str) -> Metadata {
    let git = Git::new(root, READ_DEADLINE);
    if root.trim().is_empty() || !is_work_tree(git) {
        return Metadata::default();
    }
    let sha = match git.out(&["rev-parse", "HEAD"]) {
        Some(sha) if !sha.is_empty() => sha,
        _ => return Metadata::default(),
    };
    let mut meta = Metadata {
        available: true,
        sha,
        branch: "HEAD".to_string(),
        commits: Vec::new(),
    };
    let branch = git.out(&["rev-parse", "--abbrev-ref", "HEAD"]);
    if let Some(branch) = branch.filter(|branch| !branch.is_empty()) {
        meta.branch = branch;
    }
    let limit = RECENT_COMMIT_LIMIT.to_string();
    let log_args = ["log", "-n", limit.as_str(), "--first-parent", "--format=%H%x1f%s"];
    if let Some(log) = git.out(&log_args).filter(|log| !log.is_empty()) {
        meta.commits = log.split('\n').filter_map(commit_line).collect();
    }
    meta
}

/// The local branch a commit must be an ancestor of before it counts as
/// delivered: `refs/heads/main`, or `refs/heads/master` when main is absent.
/// None outside a work tree and when neither ref exists.
pub fn delivery_base(root: &str) -> Option<String> {
    if root.trim().is_empty() {
        return None;
    }
    let git = Git::new(root, READ_DEADLINE);
    for reference in ["refs/heads/main", "refs/heads/master"] {
        if git.out(&["rev-parse", "--verify", "--quiet", reference]).is_some() {
            return Some(reference.to_string());
        }
    }
    None
}

/// Reports which of `shas` are ancestors of `base`, including `base` itself. A
/// sha that cannot be resolved is false. The read is local.
pub fn ancestors_of(root: &str, base: &str, shas: &[String]) -> HashMap<String, bool> {
    let mut out = HashMap::new();
    if root.trim().is_empty() || base.trim().is_empty() {
        return out;
    }
    let git = Git::new(root, READ_DEADLINE);
    for sha in shas {
        let sha = sha.trim();
        if sha.is_empty() || out.contains_key(sha) {
            continue;
        }
        let ancestor = git.out(&["merge-base", "--is-ancestor", sha, base]).is_some();
        out.insert(sha.to_string(), ancestor);
    }
    out
}

/// The committer time of `reference` (a sha, branch, or tag). None outside a
/// work tree, or when `reference` does not name a commit.
pub fn commit_time(root: &str, reference: &str) -> Option<OffsetDateTime> {
    let git = Git::new(root, exec::DEFAULT_TIMEOUT);
    let target = format!("{reference}^{{commit}}");
    let out = git.out(&["log", "-1", "--format=%cI", target.as_str(), "--"])?;
    OffsetDateTime::parse(&out, &Rfc3339).ok()
}

/// The full sha of `reference` when it names a commit in `root`.
pub fn resolve_commit(root: &str, reference: &str) -> Option<String> {
    let reference = reference.trim();
    if reference.is_empty() || reference.starts_with('-') {
        return None;
    }
    let git = Git::new(root, exec::DEFAULT_TIMEOUT);
    let target = format!("{reference}^{{commit}}");
    git.out(&["rev-parse", "--verify", "--quiet", target.as_str()])
        .filter(|sha| !sha.is_empty())
}

/// Reads the full sha, subject, and committer time of `reference` when it names
/// a commit in `root`. A failed log read still answers with the sha.
pub fn read_commit(root: &str, reference: &str) -> Option<CommitDetail> {
    let sha = resolve_commit(root, reference)?;
    let git = Git::new(root, exec::DEFAULT_TIMEOUT);
    let log_args = ["log", "-1", "--format=%s%x1f%cI", sha.as_str(), "--"];
    let Some(out) = git.out(&log_args) else {
        return Some(CommitDetail {
            sha,
            subject: String::new(),
            committed_at: None,
        });
    };
    let (subject, when) = out.split_once('\x1f').unwrap_or((out.as_str(), ""));
    Some(CommitDetail {
        sha,
        subject: subject.trim().to_string(),
        committed_at: OffsetDateTime::parse(when.trim(), &Rfc3339).ok(),
    })
}

/// `owner/repo` for `root`'s origin when it is on GitHub, and None for any other
/// remote or none.
pub fn github_repo(root: &str) -> Option<String> {
    let git = Git::new(root, exec::DEFAULT_TIMEOUT);
    let remote = git.out(&["remote", "get-url", "origin"])?;
    parse_github_remote(&remote)
}

/// True when `git` runs inside a work tree.
fn is_work_tree(git: Git<'_>) -> bool {
    git.out(&["rev-parse", "--is-inside-work-tree"]).as_deref() == Some("true")
}

/// The paths HEAD changed since it forked from `main`, or none when git finds no
/// merge base between them.
fn paths_since_base(git: Git<'_>, main: &str) -> Vec<String> {
    let Some(base) = git.out(&["merge-base", "HEAD", main]) else {
        return Vec::new();
    };
    if base.is_empty() {
        return Vec::new();
    }
    git.lines(&["diff", "--name-only", base.as_str(), "HEAD"])
}

/// Parses one `%H%x1f%s` line of the commit log. A line with no separator, or
/// with an empty sha, is skipped.
fn commit_line(line: &str) -> Option<Commit> {
    let (sha, subject) = line.split_once('\x1f')?;
    let sha = sha.trim();
    if sha.is_empty() {
        return None;
    }
    Some(Commit {
        sha: sha.to_string(),
        subject: subject.trim().to_string(),
    })
}

/// Trims each path and turns backslashes into slashes, drops the empty ones, and
/// returns the rest sorted in byte order without repeats.
fn unique_paths(paths: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = paths
        .into_iter()
        .map(|path| path.replace('\\', "/").trim().to_string())
        .filter(|path| !path.is_empty())
        .collect();
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use tempfile::TempDir;

    use super::*;

    /// Runs git in `dir` with a fixed author and committer, as the Go test's
    /// `gitCmd` does.
    fn git_cmd(dir: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .output()
            .expect("git starts");
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr),
        );
    }

    fn dir_str(dir: &TempDir) -> &str {
        dir.path().to_str().expect("temp dir path is UTF-8")
    }

    #[test]
    fn status_fixture() {
        let dir = tempfile::tempdir().expect("temp dir");
        git_cmd(dir.path(), &["init"]);
        std::fs::write(dir.path().join("a.txt"), "x").expect("write a.txt");
        let out = status(dir_str(&dir)).expect("status of a work tree");
        assert!(out.contains("?? a.txt"), "untracked file listed: {out:?}");
    }

    #[test]
    fn read_metadata_outside_work_tree() {
        let dir = tempfile::tempdir().expect("temp dir");
        assert_eq!(read_metadata(dir_str(&dir)), Metadata::default());
    }

    #[test]
    fn read_metadata_first_parent() {
        let dir = tempfile::tempdir().expect("temp dir");
        git_cmd(dir.path(), &["init", "-b", "main"]);
        git_cmd(
            dir.path(),
            &["commit", "--allow-empty", "-m", "first subject"],
        );
        let meta = read_metadata(dir_str(&dir));
        assert!(meta.available, "expected a work tree");
        assert_eq!(meta.branch, "main");
        assert!(!meta.sha.is_empty(), "expected a head sha");
        assert!(
            !meta.sha.contains('\n'),
            "sha not a single token: {:?}",
            meta.sha,
        );
        let head = Commit {
            sha: meta.sha.clone(),
            subject: "first subject".to_string(),
        };
        assert_eq!(meta.commits, [head]);
    }
}
