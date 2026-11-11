//! Port of `backend/providers/sqlite/entities.go`.
//!
//! Only the project settings `store.go` reads and writes are here; the
//! integration resource and sync-run rows land with their own run.

use rusqlite::params;

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
}
