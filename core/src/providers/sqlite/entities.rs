//! Project settings, integration resources, and sync runs, stored.
//!
//! The project's settings row, the AGENTS.md sync switch over it, and the
//! integration resources and sync runs of a project's connections.

use rusqlite::params;

use crate::domains::projects;

use super::Result;

impl super::Store {
    pub fn save_project_settings(&self, settings: projects::Settings) -> Result<()> {
        self.db.execute(
            r#"INSERT INTO project_settings(project_id,description,repository) VALUES(?,?,?)
		ON CONFLICT(project_id) DO UPDATE SET description=excluded.description,repository=excluded.repository"#,
            params![
                settings.project_id,
                settings.description,
                settings.repository,
            ],
        )?;
        Ok(())
    }

    pub fn get_project_settings(&self, project_id: &str) -> Result<projects::Settings> {
        self.db.query_row(
            r#"SELECT project_id,description,repository,agents_sync FROM project_settings WHERE project_id=?"#,
            params![project_id],
            |row| {
                Ok(projects::Settings {
                    project_id: row.get(0)?,
                    description: row.get(1)?,
                    repository: row.get(2)?,
                    agents_sync: row.get(3)?,
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
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    use crate::providers::sqlite::test_support::{TempStore, save_project};

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
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
}
