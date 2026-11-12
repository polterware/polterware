//! The project's repositories, stored.
//!
//! The repositories Structure components link by name: their GitHub identity
//! and this machine's clone.

use rusqlite::params;

use crate::domains::projects;

use super::{Result, format_rfc3339, parse_rfc3339};

impl super::Store {
    /// Returns the project's repositories by name.
    pub fn list_repositories(&self, project_id: &str) -> Result<Vec<projects::Repository>> {
        let mut stmt = self.db.prepare(
            r#"SELECT id, project_id, name, remote, local_path, created_at, updated_at FROM repositories WHERE project_id=? ORDER BY name"#,
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            let created: String = row.get(5)?;
            let updated: String = row.get(6)?;
            Ok(projects::Repository {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                remote: row.get(3)?,
                local_path: row.get(4)?,
                created_at: parse_rfc3339(&created),
                updated_at: parse_rfc3339(&updated),
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Inserts a repository or updates its remote and clone path.
    pub fn save_repository(&self, repo: projects::Repository) -> Result<()> {
        self.db.execute(
            r#"INSERT INTO repositories(id,project_id,name,remote,local_path,created_at,updated_at) VALUES(?,?,?,?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET remote=excluded.remote,local_path=excluded.local_path,updated_at=excluded.updated_at"#,
            params![
                repo.id,
                repo.project_id,
                repo.name,
                repo.remote,
                repo.local_path,
                format_rfc3339(repo.created_at),
                format_rfc3339(repo.updated_at),
            ],
        )?;
        Ok(())
    }

    /// Removes a repository no Structure node and no task names any more. It
    /// reports whether it removed one.
    pub fn delete_unused_repository(&self, project_id: &str, name: &str) -> Result<bool> {
        if name.is_empty() {
            return Ok(false);
        }
        let removed = self.db.execute(
            r#"DELETE FROM repositories WHERE project_id=? AND name=?
		AND NOT EXISTS (SELECT 1 FROM structure_nodes WHERE project_id=? AND repo=? AND status<>'removed')
		AND NOT EXISTS (SELECT 1 FROM work_tasks t JOIN work_cards c ON c.id=t.card_id WHERE c.project_id=? AND t.repo=?)"#,
            params![project_id, name, project_id, name, project_id, name],
        )?;
        Ok(removed > 0)
    }
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    use crate::domains::projects;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    fn repository(
        id: &str,
        name: &str,
        remote: &str,
        local_path: &str,
        at: OffsetDateTime,
    ) -> projects::Repository {
        projects::Repository {
            id: id.to_string(),
            project_id: "proj-a".to_string(),
            name: name.to_string(),
            remote: remote.to_string(),
            local_path: local_path.to_string(),
            created_at: at,
            updated_at: at,
        }
    }

    #[test]
    fn repositories_save_list_and_prune() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "proj-a", at);
        for repo in [
            repository("repo-web", "web", "", "/work/web", at),
            repository("repo-api", "api", "github.com/acme/api", "/work/api", at),
        ] {
            store.save_repository(repo).expect("save repository");
        }
        let listed = store.list_repositories("proj-a").expect("repositories");
        assert_eq!(listed.len(), 2, "repositories = {listed:?}");
        assert_eq!(listed[0].name, "api");
        assert_eq!(listed[0].remote, "github.com/acme/api");
        assert_eq!(listed[1].name, "web");

        let repo = repository("repo-api", "api", "github.com/acme/api", "/moved/api", at);
        store.save_repository(repo).expect("update");
        let listed = store.list_repositories("proj-a").expect("repositories");
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].local_path, "/moved/api");

        // No node or task names either repository, so both go.
        assert!(
            store
                .delete_unused_repository("proj-a", "api")
                .expect("delete")
        );
        assert!(
            !store
                .delete_unused_repository("proj-a", "api")
                .expect("delete twice"),
            "deleting twice removed something"
        );
        let listed = store.list_repositories("proj-a").expect("repositories");
        assert_eq!(listed.len(), 1, "after delete: {listed:?}");
        assert_eq!(listed[0].name, "web");
    }

    #[test]
    fn repositories_reject_unknown_project() {
        let holder = TempStore::new();
        let at = now();
        let mut repo = repository("r", "api", "", "/x", at);
        repo.project_id = "ghost".to_string();
        let result = holder.store.save_repository(repo);
        assert!(
            result.is_err(),
            "foreign keys must reject a repo for an unknown project"
        );
    }
}
