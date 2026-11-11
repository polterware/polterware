//! Port of `backend/providers/sqlite/structure.go`.
//!
//! Only the node reader `scope.go` resolves a scope against is here; the
//! Structure write operations land with their own run.

use rusqlite::params;

use crate::domains::structure;

use super::Result;

impl super::Store {
    /// Returns every Structure node of the project, removed ones included, in
    /// position order, each with its deploy links and their latest check.
    pub fn load_structure_nodes(&self, project_id: &str) -> Result<Vec<structure::Node>> {
        let mut nodes: Vec<structure::Node> = Vec::new();
        let mut index = std::collections::HashMap::new();
        {
            let mut stmt = self.db.prepare(
                r#"SELECT id,project_id,COALESCE(parent_id,''),level,slug,path,title,intent,status,decisions_json,position,repo,folder,kind,tech,migration_from,migration_to
		FROM structure_nodes WHERE project_id=? ORDER BY position, created_at"#,
            )?;
            let mut rows = stmt.query(params![project_id])?;
            while let Some(row) = rows.next()? {
                let decisions: String = row.get(9)?;
                let node = structure::Node {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    parent_id: row.get(2)?,
                    level: row.get(3)?,
                    slug: row.get(4)?,
                    path: row.get(5)?,
                    title: row.get(6)?,
                    intent: row.get(7)?,
                    status: row.get(8)?,
                    decisions: serde_json::from_str::<Vec<String>>(&decisions).unwrap_or_default(),
                    deploys: Vec::new(),
                    position: row.get(10)?,
                    repo: row.get(11)?,
                    folder: row.get(12)?,
                    kind: row.get(13)?,
                    tech: row.get(14)?,
                    migration_from: row.get(15)?,
                    migration_to: row.get(16)?,
                };
                index.insert(node.id.clone(), nodes.len());
                nodes.push(node);
            }
        }
        if nodes.is_empty() {
            return Ok(nodes);
        }
        self.each_row(
            r#"SELECT d.id,d.node_id,d.environment_id,COALESCE(e.name,''),d.provider,d.resource,d.target_id,d.target_url,
			COALESCE(c.state,''),COALESCE(c.summary,''),COALESCE(c.url,''),COALESCE(c.checked_at,'')
		FROM node_deployments d LEFT JOIN environments e ON e.id=d.environment_id LEFT JOIN deploy_checks c ON c.deployment_id=d.id
		WHERE d.project_id=? ORDER BY e.name, d.environment_id"#,
            params![project_id],
            |row| {
                let node_id: String = row.get(1)?;
                let mut deploy = structure::Deployment {
                    id: row.get(0)?,
                    environment_id: row.get(2)?,
                    environment: row.get(3)?,
                    provider: row.get(4)?,
                    resource: row.get(5)?,
                    target_id: row.get(6)?,
                    url: row.get(7)?,
                    check: None,
                };
                let check = structure::DeployCheck {
                    state: row.get(8)?,
                    summary: row.get(9)?,
                    url: row.get(10)?,
                    checked_at: row.get(11)?,
                };
                if !check.state.is_empty() {
                    deploy.check = Some(check);
                }
                if let Some(&position) = index.get(&node_id) {
                    nodes[position].deploys.push(deploy);
                }
                Ok(())
            },
        )?;
        Ok(nodes)
    }
}
