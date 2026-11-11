//! Port of `backend/domains/projects`.
//!
//! A project is a name and its environments, with no folder. Repositories are
//! Git clones Structure components link by name.

use std::sync::LazyLock;

use regex::Regex;
use time::OffsetDateTime;

pub const STATUS_ACTIVE: &str = "active";
pub const STATUS_ARCHIVED: &str = "archived";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProjectError {
    #[error("invalid project entity")]
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub status: String,
    pub archived_at: Option<OffsetDateTime>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
            status: String::new(),
            archived_at: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Environment {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub protection: String,
    pub is_default: bool,
    pub description: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Settings {
    pub project_id: String,
    pub description: String,
    pub default_environment_id: String,
    pub repository: String,
    /// Keeps the generated AGENTS.md block of each checkout current. It is set
    /// on its own, never by a settings save.
    pub agents_sync: bool,
}

/// A Git repository Structure components link by name. `remote` is its
/// GitHub-level identity (github.com/owner/repo, empty when it has none);
/// `local_path` is this machine's clone, empty when it is not cloned here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub remote: String,
    pub local_path: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Default for Repository {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            name: String::new(),
            remote: String::new(),
            local_path: String::new(),
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

static REPO_NAME_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").expect("valid repo name pattern"));

/// Reports whether `name` is a short lowercase handle.
pub fn valid_repo_name(name: &str) -> bool {
    name.len() <= 40 && REPO_NAME_PATTERN.is_match(name)
}

pub fn new_project(id: &str, name: &str) -> Result<Project, ProjectError> {
    if id.is_empty() || name.is_empty() {
        return Err(ProjectError::Invalid);
    }
    let now = OffsetDateTime::now_utc();
    Ok(Project {
        id: id.to_string(),
        name: name.to_string(),
        created_at: now,
        updated_at: now,
        status: STATUS_ACTIVE.to_string(),
        archived_at: None,
    })
}

pub fn new_repository(
    id: &str,
    project_id: &str,
    name: &str,
    remote: &str,
    local_path: &str,
) -> Result<Repository, ProjectError> {
    if id.is_empty()
        || project_id.is_empty()
        || (remote.is_empty() && local_path.is_empty())
        || !valid_repo_name(name)
    {
        return Err(ProjectError::Invalid);
    }
    let now = OffsetDateTime::now_utc();
    Ok(Repository {
        id: id.to_string(),
        project_id: project_id.to_string(),
        name: name.to_string(),
        remote: remote.to_string(),
        local_path: local_path.to_string(),
        created_at: now,
        updated_at: now,
    })
}

pub fn new_environment(
    id: &str,
    project_id: &str,
    name: &str,
    kind: &str,
) -> Result<Environment, ProjectError> {
    if id.is_empty() || project_id.is_empty() || name.is_empty() {
        return Err(ProjectError::Invalid);
    }
    let kind = if kind.is_empty() { "local" } else { kind };
    Ok(Environment {
        id: id.to_string(),
        project_id: project_id.to_string(),
        name: name.to_string(),
        kind: kind.to_string(),
        protection: "standard".to_string(),
        is_default: false,
        description: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_project_needs_no_folder() {
        assert!(new_project("project", "Project").is_ok());
        assert!(
            new_project("project", "").is_err(),
            "a project needs a name"
        );
    }

    #[test]
    fn repository_needs_a_remote_or_a_clone_and_a_short_handle() {
        assert!(new_repository("r1", "p1", "polterware", "", "/tmp/polterware").is_ok());
        assert!(
            new_repository(
                "r1",
                "p1",
                "polterware",
                "github.com/polterware/polterware",
                ""
            )
            .is_ok()
        );
        assert_eq!(
            new_repository("r1", "p1", "polterware", "", "").unwrap_err(),
            ProjectError::Invalid
        );
        assert!(!valid_repo_name("Polterware"));
        assert!(!valid_repo_name("polterware_ui"));
        assert!(!valid_repo_name(&"a".repeat(41)));
        assert!(valid_repo_name("polterware-ui-2"));
    }

    #[test]
    fn environment_defaults() {
        let environment = new_environment("e1", "p1", "Local", "").expect("valid environment");
        assert_eq!(environment.kind, "local");
        assert_eq!(environment.protection, "standard");
        assert_eq!(
            new_environment("e1", "p1", "", "local").unwrap_err(),
            ProjectError::Invalid
        );
    }
}
