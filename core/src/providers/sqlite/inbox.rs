//! Port of `backend/providers/sqlite/inbox.go`.
//!
//! What the Inbox keeps: a person's dispositions on derived items (append-only
//! rows, latest per key) and the deploy links the Deploy and Setup items are
//! derived from.

use std::collections::HashMap;

use rusqlite::params;
use time::OffsetDateTime;
use time::UtcOffset;
use time::format_description::well_known::Rfc3339;

use crate::domains::inbox;
use crate::platform::ids;

use super::{Result, parse_rfc3339};

impl super::Store {
    /// Appends one disposition row. Rows are never updated.
    pub fn insert_inbox_disposition(
        &self,
        project_id: &str,
        disposition: inbox::Disposition,
    ) -> Result<()> {
        self.db.execute(
            r#"INSERT INTO inbox_dispositions(id,project_id,item_key,action,actor,note,created_at)
		VALUES(?,?,?,?,?,?,?)"#,
            params![
                ids::new_ulid(),
                project_id,
                disposition.key,
                disposition.action,
                disposition.actor,
                disposition.note,
                format_rfc3339_nano(disposition.created_at),
            ],
        )?;
        Ok(())
    }

    /// Returns the latest disposition per item key.
    pub fn list_inbox_dispositions(&self, project_id: &str) -> Result<Vec<inbox::Disposition>> {
        let mut latest: HashMap<String, usize> = HashMap::new();
        let mut out: Vec<inbox::Disposition> = Vec::new();
        self.each_row(
            r#"SELECT item_key,action,actor,note,created_at FROM inbox_dispositions
		WHERE project_id=? ORDER BY item_key, created_at, id"#,
            params![project_id],
            |row| {
                let created: String = row.get(4)?;
                let disposition = inbox::Disposition {
                    key: row.get(0)?,
                    action: row.get(1)?,
                    actor: row.get(2)?,
                    note: row.get(3)?,
                    created_at: parse_rfc3339(&created),
                };
                match latest.get(&disposition.key).copied() {
                    Some(position) => out[position] = disposition,
                    None => {
                        latest.insert(disposition.key.clone(), out.len());
                        out.push(disposition);
                    }
                }
                Ok(())
            },
        )?;
        Ok(out)
    }

    /// Returns every deploy link on an active Structure node.
    pub fn list_node_deploy_links(&self, project_id: &str) -> Result<Vec<inbox::DeployLink>> {
        let mut out = Vec::new();
        self.each_row(
            r#"SELECT d.id,d.node_id,n.path,n.title,d.environment_id,COALESCE(e.name,''),d.provider,d.resource,d.target_id,d.target_url,
			COALESCE(c.state,''),COALESCE(c.summary,''),COALESCE(c.url,''),COALESCE(c.checked_at,'')
		FROM node_deployments d
		JOIN structure_nodes n ON n.id=d.node_id
		LEFT JOIN environments e ON e.id=d.environment_id
		LEFT JOIN deploy_checks c ON c.deployment_id=d.id
		WHERE d.project_id=? AND n.status<>'removed'
		ORDER BY n.path, e.name, d.environment_id"#,
            params![project_id],
            |row| {
                out.push(inbox::DeployLink {
                    id: row.get(0)?,
                    node_id: row.get(1)?,
                    node_path: row.get(2)?,
                    node_title: row.get(3)?,
                    environment_id: row.get(4)?,
                    environment: row.get(5)?,
                    provider: row.get(6)?,
                    resource: row.get(7)?,
                    target_id: row.get(8)?,
                    url: row.get(9)?,
                    state: row.get(10)?,
                    summary: row.get(11)?,
                    state_url: row.get(12)?,
                    checked_at: row.get(13)?,
                    failed: false,
                });
                Ok(())
            },
        )?;
        Ok(out)
    }

    /// Stores a link's platform id and dashboard page, found by a check.
    pub fn set_deploy_target(&self, deployment_id: &str, target_id: &str, url: &str) -> Result<()> {
        self.db.execute(
            r#"UPDATE node_deployments SET target_id=?, target_url=? WHERE id=?"#,
            params![target_id, url, deployment_id],
        )?;
        Ok(())
    }

    /// Replaces a link's latest check.
    pub fn save_deploy_check(
        &self,
        deployment_id: &str,
        state: &str,
        summary: &str,
        url: &str,
        checked_at: &str,
    ) -> Result<()> {
        self.db.execute(
            r#"INSERT INTO deploy_checks(deployment_id,state,summary,url,checked_at) VALUES(?,?,?,?,?)
		ON CONFLICT(deployment_id) DO UPDATE SET state=excluded.state, summary=excluded.summary, url=excluded.url, checked_at=excluded.checked_at"#,
            params![deployment_id, state, summary, url, checked_at],
        )?;
        Ok(())
    }
}

/// Formats a stamp the way Go's `time.RFC3339Nano` does: UTC, with the
/// fraction kept only when it is not zero and without trailing zeros.
fn format_rfc3339_nano(value: OffsetDateTime) -> String {
    value
        .to_offset(UtcOffset::UTC)
        .format(&Rfc3339)
        .expect("RFC3339 formatting cannot fail")
}

#[cfg(test)]
mod tests {
    use rusqlite::params;

    use crate::domains::inbox;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    use super::*;

    fn disposition(key: &str, action: &str, at: &str) -> inbox::Disposition {
        inbox::Disposition {
            key: key.to_string(),
            action: action.to_string(),
            actor: "erick".to_string(),
            note: String::new(),
            created_at: parse_rfc3339(at),
        }
    }

    #[test]
    fn the_latest_disposition_per_key_wins() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", OffsetDateTime::UNIX_EPOCH);
        store
            .insert_inbox_disposition(
                "proj",
                disposition("deploy:api", "defer", "2026-01-01T10:00:00Z"),
            )
            .expect("first disposition");
        store
            .insert_inbox_disposition(
                "proj",
                disposition("deploy:api", "ignore", "2026-01-01T10:00:01.5Z"),
            )
            .expect("second disposition");
        store
            .insert_inbox_disposition(
                "proj",
                disposition("question:a", "defer", "2026-01-01T09:00:00Z"),
            )
            .expect("other key");

        let listed = store.list_inbox_dispositions("proj").expect("dispositions");
        assert_eq!(listed.len(), 2, "one row per key: {listed:?}");
        assert_eq!(listed[0].key, "deploy:api");
        assert_eq!(listed[0].action, "ignore");
        assert_eq!(listed[1].key, "question:a");
    }

    #[test]
    fn deploy_links_read_the_node_and_its_check() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", OffsetDateTime::UNIX_EPOCH);
        store
            .db
            .execute(
                "INSERT INTO environments(id,project_id,name,kind,protection,is_default,description) VALUES('env-1','proj','Production','production','protected',1,'')",
                [],
            )
            .expect("environment");
        store
            .db
            .execute(
                "INSERT INTO structure_nodes(id,project_id,parent_id,level,slug,path,title,intent,status,decisions_json,position,repo,folder,kind,tech,migration_from,migration_to,created_at,updated_at) VALUES('node-1','proj',NULL,'component','api','app/api','API','','active','[]',0,'','','service','Rust','','','2026-01-01T00:00:00Z','2026-01-01T00:00:00Z')",
                [],
            )
            .expect("node");
        store
            .db
            .execute(
                "INSERT INTO node_deployments(id,project_id,node_id,environment_id,provider,resource,target_id,target_url,created_at,updated_at) VALUES('deploy-1','proj','node-1','env-1','vercel','api','','','2026-01-01T00:00:00Z','2026-01-01T00:00:00Z')",
                [],
            )
            .expect("deployment");

        let links = store.list_node_deploy_links("proj").expect("links");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].node_path, "app/api");
        assert_eq!(links[0].environment, "Production");
        assert!(links[0].state.is_empty(), "an unchecked link has no state");

        store
            .save_deploy_check(
                "deploy-1",
                "failed",
                "Build failed",
                "https://x.test",
                "2026-01-01T00:00:00Z",
            )
            .expect("check");
        store
            .set_deploy_target("deploy-1", "prj_1", "https://vercel.test/prj_1")
            .expect("target");
        let links = store.list_node_deploy_links("proj").expect("links");
        assert_eq!(links[0].state, "failed");
        assert_eq!(links[0].summary, "Build failed");
        assert_eq!(links[0].target_id, "prj_1");
        assert_eq!(links[0].url, "https://vercel.test/prj_1");

        store
            .db
            .execute(
                "UPDATE structure_nodes SET status='removed' WHERE id='node-1'",
                params![],
            )
            .expect("remove node");
        let links = store.list_node_deploy_links("proj").expect("links");
        assert!(links.is_empty(), "a removed node keeps no deploy link");
    }

    #[test]
    fn dispositions_keep_the_nanoseconds_go_writes() {
        assert_eq!(
            format_rfc3339_nano(parse_rfc3339("2026-01-01T10:00:00Z")),
            "2026-01-01T10:00:00Z"
        );
        assert_eq!(
            format_rfc3339_nano(parse_rfc3339("2026-01-01T10:00:01.5Z")),
            "2026-01-01T10:00:01.5Z"
        );
        assert_eq!(
            format_rfc3339_nano(parse_rfc3339("2026-01-01T10:00:01.500000000Z")),
            "2026-01-01T10:00:01.5Z",
            "trailing zeros are dropped, as Go's RFC3339Nano does"
        );
    }
}
