#[allow(unused_imports)] // The package-wide glob; every name this file uses is explicit.
use super::*;

use std::collections::{HashMap, HashSet};

use time::OffsetDateTime;

use crate::domains::polter;
use crate::domains::work;
use crate::providers::git as gitpkg;
use crate::providers::sqlite as storepkg;

use super::brief::helpers;

/// One commit of a card's task, for the card page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardCommit {
    pub task_number: i32,
    pub repo: String,
    pub sha: String,
    pub subject: String,
    pub url: String,
    pub committed_at: OffsetDateTime,
    /// True for a commit found by its subject (`slug#T2`), with no stored link.
    pub cited: bool,
}

impl Default for CardCommit {
    fn default() -> Self {
        Self {
            task_number: 0,
            repo: String::new(),
            sha: String::new(),
            subject: String::new(),
            url: String::new(),
            committed_at: OffsetDateTime::UNIX_EPOCH,
            cited: false,
        }
    }
}

impl CardCommit {
    /// The first seven characters of the sha.
    pub fn short_sha(&self) -> &str {
        if self.sha.len() > 7 {
            &self.sha[..7]
        } else {
            &self.sha
        }
    }
}

/// A caller mistake: an unknown task or commit.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("commit input: {0}")]
pub struct CommitInputError(pub String);

fn commit_input(message: impl Into<String>) -> CommitInputError {
    CommitInputError(message.into())
}

/// Lists a card's commits, newest first: the ones linked to its tasks, then the
/// ones whose subject cites a task (`slug#T2`) in a checkout's recent history.
/// It only reads.
pub fn card_commits(
    store: &storepkg::Store,
    project_id: &str,
    slug: &str,
) -> anyhow::Result<Vec<CardCommit>> {
    let snapshot = store.load_work_snapshot(project_id)?;
    let Some(card) = snapshot.card(slug) else {
        return Err(commit_input(format!("unknown card {}", polter::go_quote(slug))).into());
    };
    let mut numbers: HashMap<String, i32> = HashMap::new();
    for task in &card.tasks {
        numbers.insert(task.id.clone(), task.number);
    }
    let links = store.list_commit_links(project_id)?;
    let mut seen: HashSet<String> = HashSet::new();
    let key = |number: i32, sha: &str| format!("{number}:{sha}");
    let mut out: Vec<CardCommit> = Vec::new();
    for link in links {
        let Some(&number) = numbers.get(&link.task_id) else {
            continue;
        };
        seen.insert(key(number, &link.sha));
        out.push(CardCommit {
            task_number: number,
            repo: link.repo,
            sha: link.sha,
            subject: link.subject,
            url: link.url,
            committed_at: link.committed_at,
            cited: false,
        });
    }
    let checkouts = helpers::project_checkouts(store, project_id)?;
    for checkout in &checkouts {
        let meta = gitpkg::read_metadata(&checkout.root);
        if !meta.available {
            continue;
        }
        let owner_repo = gitpkg::github_repo(&checkout.root);
        for commit in &meta.commits {
            for task in polter::match_explicit_tasks(&commit.subject, std::slice::from_ref(card)) {
                let Some(&number) = numbers.get(&task.id) else {
                    continue;
                };
                if seen.contains(&key(number, &commit.sha)) {
                    continue;
                }
                seen.insert(key(number, &commit.sha));
                let mut found = CardCommit {
                    task_number: number,
                    repo: checkout.name.clone(),
                    sha: commit.sha.clone(),
                    subject: commit.subject.clone(),
                    cited: true,
                    ..CardCommit::default()
                };
                if let Some(owner_repo) = &owner_repo {
                    found.url = gitpkg::github_commit_url(owner_repo, &commit.sha);
                }
                if let Some(when) = gitpkg::commit_time(&checkout.root, &commit.sha) {
                    found.committed_at = when;
                }
                out.push(found);
            }
        }
    }
    out.sort_by(|a, b| b.committed_at.cmp(&a.committed_at));
    Ok(out)
}

/// Links a commit (a sha or a prefix of one, found in any checkout) to a task
/// given as `slug#T2`, keeping its subject, time, and GitHub URL.
pub fn link_task_commit(
    store: &storepkg::Store,
    project_id: &str,
    task_ref: &str,
    sha: &str,
) -> anyhow::Result<CardCommit> {
    let task = resolve_task_ref(store, project_id, task_ref)?;
    let sha = sha.trim();
    if sha.is_empty() {
        return Err(commit_input("link needs a commit sha").into());
    }
    let checkouts = helpers::project_checkouts(store, project_id)?;
    for checkout in &checkouts {
        let Some(detail) = gitpkg::read_commit(&checkout.root, sha) else {
            continue;
        };
        let mut link = work::CommitLink {
            project_id: project_id.to_string(),
            repo: checkout.name.clone(),
            sha: detail.sha.clone(),
            task_id: task.id.clone(),
            subject: detail.subject.clone(),
            committed_at: detail.committed_at.unwrap_or(OffsetDateTime::UNIX_EPOCH),
            ..work::CommitLink::default()
        };
        if let Some(owner_repo) = gitpkg::github_repo(&checkout.root) {
            link.url = gitpkg::github_commit_url(&owner_repo, &detail.sha);
        }
        store.insert_commit_link(link.clone(), OffsetDateTime::now_utc())?;
        return Ok(CardCommit {
            task_number: task.number,
            repo: link.repo,
            sha: link.sha,
            subject: link.subject,
            url: link.url,
            committed_at: link.committed_at,
            cited: false,
        });
    }
    Err(commit_input(format!(
        "unknown commit {}: no checkout of this project has it",
        polter::go_quote(sha)
    ))
    .into())
}

/// Removes a stored link. A commit found by its subject has none: its subject
/// names the task.
pub fn unlink_task_commit(
    store: &storepkg::Store,
    project_id: &str,
    task_ref: &str,
    sha: &str,
) -> anyhow::Result<()> {
    let task = resolve_task_ref(store, project_id, task_ref)?;
    let removed = store.delete_commit_link(project_id, &task.id, sha.trim())?;
    if !removed {
        return Err(commit_input(format!(
            "{} has no linked commit {}",
            task_ref.trim(),
            polter::go_quote(sha.trim())
        ))
        .into());
    }
    Ok(())
}

fn resolve_task_ref(
    store: &storepkg::Store,
    project_id: &str,
    task_ref: &str,
) -> anyhow::Result<work::Task> {
    let reference = polter::parse_task_ref(task_ref).map_err(commit_input)?;
    let snapshot = store.load_work_snapshot(project_id)?;
    if let Some(card) = snapshot.card(&reference.slug) {
        for task in &card.tasks {
            if task.number == reference.number {
                return Ok(task.clone());
            }
        }
    }
    Err(commit_input(format!(
        "unknown task {}: use slug#T2",
        polter::go_quote(task_ref.trim())
    ))
    .into())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use pretty_assertions::assert_eq;

    use super::super::brief::helpers::{self, ApplyInput, InitInput};
    use super::*;
    use crate::providers::sqlite::test_support::TempStore;

    /// Runs git in `dir` with the identity the test needs, answering stdout.
    fn git(dir: &str, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .expect("run git");
        assert!(
            output.status.success(),
            "git {args:?}: {}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn init_with_repo(store: &storepkg::Store, name: &str, root: &str) -> String {
        let project = helpers::init_project(
            store,
            &InitInput {
                name: name.to_string(),
                ..InitInput::default()
            },
        )
        .expect("init project");
        helpers::ensure_repository(store, &project.id, root).expect("ensure repository");
        project.id
    }

    /// A card shows the commits whose subject cites one of its tasks and the
    /// ones linked to a task, each with its GitHub URL when origin is on
    /// GitHub. The Inbox stays empty: commits are optional.
    #[test]
    fn card_commits_cited_and_linked() {
        if Command::new("git").arg("--version").output().is_err() {
            return;
        }
        let root_dir = tempfile::tempdir().expect("root");
        let root = root_dir.path().to_string_lossy().into_owned();
        git(&root, &["init", "-q", "-b", "main"]);
        git(
            &root,
            &["remote", "add", "origin", "git@github.com:acme/demo.git"],
        );
        git(
            &root,
            &[
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "Add the session table (login#T1)",
            ],
        );
        let cited = git(&root, &["rev-parse", "HEAD"]);
        git(
            &root,
            &["commit", "-q", "--allow-empty", "-m", "Tidy the readme"],
        );
        let bare = git(&root, &["rev-parse", "HEAD"]);

        let temp = TempStore::named("commits.db");
        let store = &temp.store;
        let project_id = init_with_repo(store, "Demo", &root);
        let card = "---\npolter: 1\nsummary: Add login\nrationale: test\n---\n\n## login: Login\nstage: build\n- [x] T1 Session table\n- [x] T2 Login route\n";
        let (result, outcome) = helpers::apply_polter(
            store,
            &ApplyInput {
                project_id: project_id.clone(),
                text: card.to_string(),
                auto_start_label: "cli".to_string(),
                ..ApplyInput::default()
            },
        );
        assert!(
            outcome.is_ok(),
            "{:?} {:?}",
            outcome.err(),
            result.diagnostics
        );

        let commits = card_commits(store, &project_id, "login").expect("cited");
        assert_eq!(commits.len(), 1, "cited: {commits:?}");
        let got = &commits[0];
        assert_eq!(got.task_number, 1);
        assert_eq!(got.sha, cited);
        assert!(got.cited);
        assert_eq!(
            got.url,
            format!("https://github.com/acme/demo/commit/{cited}")
        );

        let short = &bare[..7];
        assert!(
            link_task_commit(store, &project_id, "login#T9", short).is_err(),
            "an unknown task cannot take a commit"
        );
        assert!(
            link_task_commit(store, &project_id, "login#T2", "nope").is_err(),
            "an unknown commit cannot be linked"
        );
        let linked = link_task_commit(store, &project_id, "login#T2", short).expect("link");
        assert_eq!(linked.sha, bare);
        assert_eq!(linked.subject, "Tidy the readme");
        assert_eq!(linked.short_sha(), short);
        let commits = card_commits(store, &project_id, "login").expect("after link");
        assert_eq!(commits.len(), 2, "after link = {commits:?}");
        let text =
            super::super::show::query_commits(store, &project_id, "login").expect("commits query");
        assert!(
            text.contains(&format!(
                "T2 {short} Tidy the readme https://github.com/acme/demo/commit/{bare}"
            )),
            "query:\n{text}"
        );

        let inbox = super::super::inbox::list_inbox(store, &project_id).expect("inbox");
        assert!(
            inbox.items.is_empty(),
            "commits raise no Inbox items: {:?}",
            inbox.items
        );

        assert!(
            unlink_task_commit(store, &project_id, "login#T1", &cited).is_err(),
            "a cited commit has no stored link to remove"
        );
        unlink_task_commit(store, &project_id, "login#T2", &bare).expect("unlink");
        let commits = card_commits(store, &project_id, "login").expect("after unlink");
        assert_eq!(commits.len(), 1, "after unlink = {commits:?}");
    }
}
