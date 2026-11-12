//! A checkout's Git: HEAD, branch, recent commits, and GitHub remotes.
//!
//! Reads a work tree and its recent commits through the `git` CLI, and parses
//! GitHub remote URLs. Nothing here contacts a remote.

mod remote;
mod repo;

pub use remote::{github_commit_url, parse_github_remote};
pub use repo::{
    Commit, CommitDetail, Metadata, Observation, RECENT_COMMIT_LIMIT, ancestors_of, commit_time,
    delivery_base, github_repo, observe, read_commit, read_metadata, resolve_commit, status,
};
