//! Port of `backend/providers/sqlite/canon.go`.
//!
//! Only the pieces `store.go` calls are here: the knowledge search index and
//! the Canon projection of a project's environments. The rest of the file
//! (entities, relationships, knowledge links, search) lands with its own run.

use rusqlite::{ToSql, params, params_from_iter};
use time::OffsetDateTime;

use crate::domains::canon;
use crate::domains::knowledge;

use super::{Result, format_rfc3339};

impl super::Store {
    /// Writes one entity, replacing the row of its ref.
    pub fn upsert_canon_entity(&self, entity: &canon::Entity) -> Result<()> {
        self.db.execute(
            r#"INSERT INTO canon_entities(
		id,project_id,kind,ref_kind,ref_id,title,truth_class,source,actor,observed_at,created_at,updated_at
	) VALUES(?,?,?,?,?,?,?,?,?,?,?,?)
	ON CONFLICT(project_id, ref_kind, ref_id) DO UPDATE SET
		title=excluded.title,
		truth_class=excluded.truth_class,
		source=excluded.source,
		actor=excluded.actor,
		observed_at=excluded.observed_at,
		updated_at=excluded.updated_at"#,
            params![
                entity.id,
                entity.project_id,
                entity.kind,
                entity.ref_kind,
                entity.ref_id,
                entity.title,
                entity.truth_class,
                entity.source,
                entity.actor,
                format_rfc3339(entity.observed_at),
                format_rfc3339(entity.created_at),
                format_rfc3339(entity.updated_at),
            ],
        )?;
        Ok(())
    }

    /// Removes the entities of one ref kind that are no longer in `keep_ids`.
    pub fn delete_canon_entities_missing_refs(
        &self,
        project_id: &str,
        ref_kind: &str,
        keep_ids: &[String],
    ) -> Result<()> {
        if keep_ids.is_empty() {
            self.db.execute(
                r#"DELETE FROM canon_entities WHERE project_id=? AND ref_kind=?"#,
                params![project_id, ref_kind],
            )?;
            return Ok(());
        }
        let placeholders = vec!["?"; keep_ids.len()].join(",");
        let query = format!(
            r#"DELETE FROM canon_entities WHERE project_id=? AND ref_kind=? AND ref_id NOT IN ({placeholders})"#
        );
        let mut args: Vec<&dyn ToSql> = Vec::with_capacity(keep_ids.len() + 2);
        args.push(&project_id);
        args.push(&ref_kind);
        for id in keep_ids {
            args.push(id);
        }
        self.db.execute(&query, params_from_iter(args))?;
        Ok(())
    }

    /// Projects environments into Canon.
    pub fn sync_project_model(&self, project_id: &str) -> Result<usize> {
        let now = OffsetDateTime::now_utc();
        let mut count = 0;
        let envs = self.list_environments(project_id)?;
        let mut keep_envs = Vec::with_capacity(envs.len());
        for env in &envs {
            let mut entity = canon::new_entity(
                project_id,
                canon::KIND_ENVIRONMENT,
                canon::REF_ENVIRONMENT,
                &env.id,
                &env.name,
                canon::TRUTH_DECLARED,
                "environments",
                "system",
            )?;
            entity.observed_at = now;
            entity.updated_at = now;
            self.upsert_canon_entity(&entity)?;
            keep_envs.push(env.id.clone());
            count += 1;
        }
        self.delete_canon_entities_missing_refs(project_id, canon::REF_ENVIRONMENT, &keep_envs)?;
        Ok(count)
    }

    /// Writes one item into the full-text index, replacing its previous row.
    pub fn index_knowledge_fts(&self, item: &knowledge::Item) -> Result<()> {
        let tags = serde_json::to_string(&item.tags).unwrap_or_default();
        let tag_text = tags.trim_matches(|c| c == '[' || c == ']').replace('"', "");
        self.db.execute(
            r#"DELETE FROM knowledge_fts WHERE knowledge_id=?"#,
            params![item.id],
        )?;
        self.db.execute(
            r#"INSERT INTO knowledge_fts(knowledge_id, project_id, kind, title, body, summary, tags) VALUES(?,?,?,?,?,?,?)"#,
            params![
                item.id,
                item.project_id,
                item.kind,
                item.title,
                item.body,
                item.summary,
                tag_text,
            ],
        )?;
        Ok(())
    }
}
