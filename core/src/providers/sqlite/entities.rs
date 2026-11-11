//! Port of `backend/providers/sqlite/entities.go`.
//!
//! The project's settings row, the AGENTS.md sync switch over it, and the
//! integration resources and sync runs of a project's connections.

use rusqlite::params;

use crate::domains::integrations;
use crate::domains::projects;

use super::{Result, nullable};

impl super::Store {
    pub fn save_project_settings(&self, settings: projects::Settings) -> Result<()> {
        self.db.execute(
            r#"INSERT INTO project_settings(project_id,description,default_environment_id,repository) VALUES(?,?,?,?)
		ON CONFLICT(project_id) DO UPDATE SET description=excluded.description,default_environment_id=excluded.default_environment_id,repository=excluded.repository"#,
            params![
                settings.project_id,
                settings.description,
                nullable(&settings.default_environment_id),
                settings.repository,
            ],
        )?;
        Ok(())
    }

    pub fn get_project_settings(&self, project_id: &str) -> Result<projects::Settings> {
        self.db.query_row(
            r#"SELECT project_id,description,COALESCE(default_environment_id,''),repository,agents_sync FROM project_settings WHERE project_id=?"#,
            params![project_id],
            |row| {
                Ok(projects::Settings {
                    project_id: row.get(0)?,
                    description: row.get(1)?,
                    default_environment_id: row.get(2)?,
                    repository: row.get(3)?,
                    agents_sync: row.get(4)?,
                })
            },
        )
        .map_err(Into::into)
    }

    /// Turns AGENTS.md sync on or off for a project, creating its settings row
    /// when it has none.
    pub fn set_agents_sync(&self, project_id: &str, on: bool) -> Result<()> {
        self.db.execute(
            r#"INSERT INTO project_settings(project_id,agents_sync) VALUES(?,?)
		ON CONFLICT(project_id) DO UPDATE SET agents_sync=excluded.agents_sync"#,
            params![project_id, on],
        )?;
        Ok(())
    }

    /// Lists the projects with AGENTS.md sync on.
    pub fn agents_sync_projects(&self) -> Result<Vec<String>> {
        let mut stmt = self.db.prepare(
            r#"SELECT ps.project_id FROM project_settings ps JOIN projects p ON p.id = ps.project_id
		WHERE ps.agents_sync = 1 AND p.status = 'active' ORDER BY ps.project_id"#,
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn save_integration_resource(&self, resource: integrations::Resource) -> Result<()> {
        self.db.execute(
            r#"INSERT INTO integration_resources(id,connection_id,name,kind,external_id,environment_id,status) VALUES(?,?,?,?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET connection_id=excluded.connection_id,name=excluded.name,kind=excluded.kind,external_id=excluded.external_id,environment_id=excluded.environment_id,status=excluded.status"#,
            params![
                resource.id,
                resource.connection_id,
                resource.name,
                resource.kind,
                resource.external_id,
                nullable(&resource.environment_id),
                resource.status,
            ],
        )?;
        Ok(())
    }

    pub fn delete_integration_resources_by_connection(&self, connection_id: &str) -> Result<()> {
        self.db.execute(
            r#"DELETE FROM integration_resources WHERE connection_id=?"#,
            params![connection_id],
        )?;
        Ok(())
    }

    pub fn count_integration_resources_by_connection(&self, connection_id: &str) -> Result<usize> {
        let count: i64 = self.db.query_row(
            r#"SELECT COUNT(*) FROM integration_resources WHERE connection_id=?"#,
            params![connection_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Returns the project's resources by name, across its connections.
    pub fn list_integration_resources(
        &self,
        project_id: &str,
    ) -> Result<Vec<integrations::Resource>> {
        let mut stmt = self.db.prepare(
            r#"SELECT r.id,r.connection_id,r.name,r.kind,r.external_id,COALESCE(r.environment_id,''),r.status
		FROM integration_resources r
		JOIN integration_connections c ON c.id = r.connection_id
		WHERE c.project_id=? ORDER BY r.name"#,
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok(integrations::Resource {
                id: row.get(0)?,
                connection_id: row.get(1)?,
                name: row.get(2)?,
                kind: row.get(3)?,
                external_id: row.get(4)?,
                environment_id: row.get(5)?,
                status: row.get(6)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn save_integration_sync_run(&self, run: integrations::SyncRun) -> Result<()> {
        self.db.execute(
            r#"INSERT INTO integration_sync_runs(id,connection_id,started_at,finished_at,result,imported,changed,message) VALUES(?,?,?,?,?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET connection_id=excluded.connection_id,started_at=excluded.started_at,finished_at=excluded.finished_at,result=excluded.result,imported=excluded.imported,changed=excluded.changed,message=excluded.message"#,
            params![
                run.id,
                run.connection_id,
                run.started_at,
                run.finished_at,
                run.result,
                run.imported,
                run.changed,
                run.message,
            ],
        )?;
        Ok(())
    }

    /// Returns the project's sync runs, newest start first.
    pub fn list_integration_sync_runs(
        &self,
        project_id: &str,
    ) -> Result<Vec<integrations::SyncRun>> {
        let mut stmt = self.db.prepare(
            r#"SELECT r.id,r.connection_id,r.started_at,r.finished_at,r.result,r.imported,r.changed,r.message
		FROM integration_sync_runs r
		JOIN integration_connections c ON c.id = r.connection_id
		WHERE c.project_id=? ORDER BY r.started_at DESC"#,
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok(integrations::SyncRun {
                id: row.get(0)?,
                connection_id: row.get(1)?,
                started_at: row.get(2)?,
                finished_at: row.get(3)?,
                result: row.get(4)?,
                imported: row.get(5)?,
                changed: row.get(6)?,
                message: row.get(7)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    use crate::domains::integrations;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    fn connection(id: &str, project_id: &str) -> integrations::Connection {
        integrations::Connection {
            id: id.to_string(),
            project_id: project_id.to_string(),
            provider_id: "vercel".to_string(),
            status: "connected".to_string(),
            label: "Vercel".to_string(),
            ..integrations::Connection::default()
        }
    }

    #[test]
    fn agents_sync_round_trips_and_lists_active_projects() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        for id in ["proj-a", "proj-b"] {
            save_project(store, id, at);
        }
        assert!(
            store.agents_sync_projects().expect("projects").is_empty(),
            "a fresh project does not sync AGENTS.md"
        );
        store.set_agents_sync("proj-a", true).expect("on");
        store.set_agents_sync("proj-b", true).expect("on");
        assert_eq!(
            store.agents_sync_projects().expect("projects"),
            vec!["proj-a".to_string(), "proj-b".to_string()]
        );
        let settings = store.get_project_settings("proj-a").expect("settings");
        assert!(settings.agents_sync, "the switch is stored in the row");
        assert_eq!(settings.project_id, "proj-a");
        assert_eq!(
            settings.description, "",
            "creating the old row keeps its defaults"
        );

        store.set_agents_sync("proj-b", false).expect("off");
        store.archive_project("proj-a").expect("archive");
        assert!(
            store.agents_sync_projects().expect("projects").is_empty(),
            "an archived project stops syncing"
        );
    }

    #[test]
    fn integration_resources_list_by_project_and_delete_by_connection() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        store
            .save_environment(crate::domains::projects::Environment {
                id: "env-1".to_string(),
                project_id: "p".to_string(),
                name: "env-1".to_string(),
                kind: "local".to_string(),
                ..crate::domains::projects::Environment::default()
            })
            .expect("environment");
        store
            .save_connection(connection("c1", "p"))
            .expect("connection one");
        store
            .save_connection(connection("c2", "p"))
            .expect("connection two");
        store
            .save_integration_resource(integrations::Resource {
                id: "r-web".to_string(),
                connection_id: "c1".to_string(),
                name: "web".to_string(),
                kind: "site".to_string(),
                external_id: "prj_1".to_string(),
                environment_id: String::new(),
                status: "active".to_string(),
            })
            .expect("web resource");
        store
            .save_integration_resource(integrations::Resource {
                id: "r-api".to_string(),
                connection_id: "c2".to_string(),
                name: "api".to_string(),
                kind: "service".to_string(),
                external_id: "srv_1".to_string(),
                environment_id: "env-1".to_string(),
                status: "active".to_string(),
            })
            .expect("api resource");

        let listed = store.list_integration_resources("p").expect("resources");
        assert_eq!(
            listed
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
            vec!["api", "web"],
            "resources come back by name"
        );
        assert_eq!(listed[0].environment_id, "env-1");
        assert_eq!(
            listed[1].environment_id, "",
            "an empty environment reads as empty"
        );
        assert!(
            store
                .list_integration_resources("other")
                .expect("none")
                .is_empty()
        );

        assert_eq!(
            store
                .count_integration_resources_by_connection("c1")
                .expect("count"),
            1
        );
        store
            .delete_integration_resources_by_connection("c1")
            .expect("delete");
        assert_eq!(
            store
                .count_integration_resources_by_connection("c1")
                .expect("count"),
            0
        );
        assert_eq!(
            store
                .list_integration_resources("p")
                .expect("resources")
                .len(),
            1
        );
    }

    #[test]
    fn integration_sync_runs_come_back_newest_first() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        store
            .save_connection(connection("c1", "p"))
            .expect("connection");
        for (id, started) in [
            ("run-1", "2026-01-01T00:00:00Z"),
            ("run-2", "2026-02-01T00:00:00Z"),
        ] {
            store
                .save_integration_sync_run(integrations::SyncRun {
                    id: id.to_string(),
                    connection_id: "c1".to_string(),
                    started_at: started.to_string(),
                    finished_at: started.to_string(),
                    result: "ok".to_string(),
                    imported: 2,
                    changed: 1,
                    message: String::new(),
                })
                .expect("run");
        }
        let runs = store.list_integration_sync_runs("p").expect("runs");
        assert_eq!(
            runs.iter().map(|run| run.id.as_str()).collect::<Vec<_>>(),
            vec!["run-2", "run-1"]
        );
        assert_eq!(runs[0].imported, 2);
        assert_eq!(runs[0].changed, 1);
        assert!(
            store
                .list_integration_sync_runs("other")
                .expect("no runs")
                .is_empty()
        );
    }
}
