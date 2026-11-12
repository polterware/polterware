//! Canon entities, relationships, and facts, stored.
//!
//! Canon in the store: entities projected from their ref, the relationships
//! between them, the links from knowledge items to entities, the full-text
//! index over knowledge, and the search that reads it.

use std::collections::HashSet;

use rusqlite::types::{ToSql, Value};
use rusqlite::{Row, params, params_from_iter};
use time::OffsetDateTime;

use crate::domains::canon;
use crate::domains::knowledge;

use super::{Error, Result, format_rfc3339};

impl super::Store {
    /// Writes one entity, replacing the row of its ref.
    pub fn upsert_canon_entity(&self, entity: &canon::Entity) -> Result<()> {
        self.upsert_canon_entity_with(entity, false)
    }

    /// Writes one entity only when a projected value changed, so a re-store of
    /// the same projection leaves its timestamps alone.
    pub fn upsert_canon_entity_if_changed(&self, entity: &canon::Entity) -> Result<()> {
        self.upsert_canon_entity_with(entity, true)
    }

    fn upsert_canon_entity_with(
        &self,
        entity: &canon::Entity,
        only_if_changed: bool,
    ) -> Result<()> {
        let mut query = String::from(
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
        );
        if only_if_changed {
            query.push_str(
                " WHERE canon_entities.title IS NOT excluded.title\n\t\t\tOR canon_entities.truth_class IS NOT excluded.truth_class\n\t\t\tOR canon_entities.source IS NOT excluded.source\n\t\t\tOR canon_entities.actor IS NOT excluded.actor",
            );
        }
        self.db.execute(
            &query,
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

    /// Returns the project's entities, filtered by kind when `kind` is not
    /// empty, in title order.
    pub fn list_canon_entities(&self, project_id: &str, kind: &str) -> Result<Vec<canon::Entity>> {
        let mut query = String::from(
            "SELECT id,project_id,kind,ref_kind,ref_id,title,truth_class,source,actor,observed_at,created_at,updated_at\n\t\tFROM canon_entities WHERE project_id=?",
        );
        let mut args: Vec<Value> = vec![project_id.to_string().into()];
        if !kind.is_empty() {
            query.push_str(" AND kind=?");
            args.push(kind.to_string().into());
        }
        query.push_str(" ORDER BY title");
        let mut stmt = self.db.prepare(&query)?;
        let mut rows = stmt.query(params_from_iter(args))?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push(scan_canon_entity(row)?);
        }
        Ok(result)
    }

    pub fn get_canon_entity(&self, id: &str) -> Result<canon::Entity> {
        self.db
            .query_row(
                "SELECT id,project_id,kind,ref_kind,ref_id,title,truth_class,source,actor,observed_at,created_at,updated_at\n\t\tFROM canon_entities WHERE id=?",
                params![id],
                scan_canon_entity,
            )
            .map_err(Into::into)
    }

    pub fn get_canon_entity_by_ref(
        &self,
        project_id: &str,
        ref_kind: &str,
        ref_id: &str,
    ) -> Result<canon::Entity> {
        self.db
            .query_row(
                "SELECT id,project_id,kind,ref_kind,ref_id,title,truth_class,source,actor,observed_at,created_at,updated_at\n\t\tFROM canon_entities WHERE project_id=? AND ref_kind=? AND ref_id=?",
                params![project_id, ref_kind, ref_id],
                scan_canon_entity,
            )
            .map_err(Into::into)
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

    pub fn upsert_canon_relationship(&self, rel: &canon::Relationship) -> Result<()> {
        self.upsert_canon_relationship_with(rel, false)
    }

    /// Writes one relationship only when its provenance changed.
    pub fn upsert_canon_relationship_if_changed(&self, rel: &canon::Relationship) -> Result<()> {
        self.upsert_canon_relationship_with(rel, true)
    }

    fn upsert_canon_relationship_with(
        &self,
        rel: &canon::Relationship,
        only_if_changed: bool,
    ) -> Result<()> {
        let mut query = String::from(
            r#"INSERT INTO canon_relationships(
		id,project_id,subject_entity_id,predicate,object_entity_id,truth_class,source,actor,observed_at,created_at
	) VALUES(?,?,?,?,?,?,?,?,?,?)
	ON CONFLICT(project_id, subject_entity_id, predicate, object_entity_id) DO UPDATE SET
		truth_class=excluded.truth_class,
		source=excluded.source,
		actor=excluded.actor,
		observed_at=excluded.observed_at"#,
        );
        if only_if_changed {
            query.push_str(
                " WHERE canon_relationships.truth_class IS NOT excluded.truth_class\n\t\t\tOR canon_relationships.source IS NOT excluded.source\n\t\t\tOR canon_relationships.actor IS NOT excluded.actor",
            );
        }
        self.db.execute(
            &query,
            params![
                rel.id,
                rel.project_id,
                rel.subject_entity_id,
                rel.predicate,
                rel.object_entity_id,
                rel.truth_class,
                rel.source,
                rel.actor,
                format_rfc3339(rel.observed_at),
                format_rfc3339(rel.created_at),
            ],
        )?;
        Ok(())
    }

    /// Returns the project's relationships touching `entity_id`, or all of them
    /// when it is empty, oldest first.
    pub fn list_canon_relationships(
        &self,
        project_id: &str,
        entity_id: &str,
    ) -> Result<Vec<canon::Relationship>> {
        let mut query = String::from(
            "SELECT id,project_id,subject_entity_id,predicate,object_entity_id,truth_class,source,actor,observed_at,created_at\n\t\tFROM canon_relationships WHERE project_id=?",
        );
        let mut args: Vec<Value> = vec![project_id.to_string().into()];
        if !entity_id.is_empty() {
            query.push_str(" AND (subject_entity_id=? OR object_entity_id=?)");
            args.push(entity_id.to_string().into());
            args.push(entity_id.to_string().into());
        }
        query.push_str(" ORDER BY created_at");
        let mut stmt = self.db.prepare(&query)?;
        let mut rows = stmt.query(params_from_iter(args))?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push(scan_canon_relationship(row)?);
        }
        Ok(result)
    }

    /// Removes the relationships of one source that are no longer in
    /// `keep_ids`.
    pub fn delete_canon_relationships_missing(
        &self,
        project_id: &str,
        source: &str,
        keep_ids: &[String],
    ) -> Result<()> {
        if keep_ids.is_empty() {
            self.db.execute(
                r#"DELETE FROM canon_relationships WHERE project_id=? AND source=?"#,
                params![project_id, source],
            )?;
            return Ok(());
        }
        let placeholders = vec!["?"; keep_ids.len()].join(",");
        let query = format!(
            r#"DELETE FROM canon_relationships WHERE project_id=? AND source=? AND id NOT IN ({placeholders})"#
        );
        let mut args: Vec<&dyn ToSql> = Vec::with_capacity(keep_ids.len() + 2);
        args.push(&project_id);
        args.push(&source);
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

    pub fn save_knowledge_link(&self, link: &knowledge::EntityLink) -> Result<()> {
        self.db.execute(
            r#"INSERT INTO knowledge_entity_links(
		id,project_id,knowledge_id,entity_id,predicate,truth_class,source,actor,observed_at,created_at
	) VALUES(?,?,?,?,?,?,?,?,?,?)
	ON CONFLICT(knowledge_id, entity_id, predicate) DO UPDATE SET
		truth_class=excluded.truth_class,
		source=excluded.source,
		actor=excluded.actor,
		observed_at=excluded.observed_at"#,
            params![
                link.id,
                link.project_id,
                link.knowledge_id,
                link.entity_id,
                link.predicate,
                link.truth_class,
                link.source,
                link.actor,
                format_rfc3339(link.observed_at),
                format_rfc3339(link.created_at),
            ],
        )?;
        Ok(())
    }

    pub fn get_knowledge_link(&self, id: &str) -> Result<knowledge::EntityLink> {
        self.db
            .query_row(
                r#"SELECT id,project_id,knowledge_id,entity_id,predicate,truth_class,source,actor,observed_at,created_at
		 FROM knowledge_entity_links WHERE id=?"#,
                params![id],
                scan_knowledge_link,
            )
            .map_err(Into::into)
    }

    /// Returns the project's knowledge links, filtered by item and entity, oldest
    /// first.
    pub fn list_knowledge_links(
        &self,
        project_id: &str,
        knowledge_id: &str,
        entity_id: &str,
    ) -> Result<Vec<knowledge::EntityLink>> {
        let mut query = String::from(
            "SELECT id,project_id,knowledge_id,entity_id,predicate,truth_class,source,actor,observed_at,created_at\n\t\tFROM knowledge_entity_links WHERE project_id=?",
        );
        let mut args: Vec<Value> = vec![project_id.to_string().into()];
        if !knowledge_id.is_empty() {
            query.push_str(" AND knowledge_id=?");
            args.push(knowledge_id.to_string().into());
        }
        if !entity_id.is_empty() {
            query.push_str(" AND entity_id=?");
            args.push(entity_id.to_string().into());
        }
        query.push_str(" ORDER BY created_at");
        let mut stmt = self.db.prepare(&query)?;
        let mut rows = stmt.query(params_from_iter(args))?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push(scan_knowledge_link(row)?);
        }
        Ok(result)
    }

    pub fn delete_knowledge_link(&self, id: &str) -> Result<()> {
        self.db.execute(
            r#"DELETE FROM knowledge_entity_links WHERE id=?"#,
            params![id],
        )?;
        Ok(())
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

    /// Searches the project's knowledge: the exact id first, then the full-text
    /// index, newest relevance first. Archived items never surface.
    pub fn search_knowledge(
        &self,
        project_id: &str,
        query: &str,
        kind: &str,
        limit: i32,
    ) -> Result<Vec<knowledge::SearchHit>> {
        let limit = if limit <= 0 || limit > 50 { 20 } else { limit };
        let query = query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }

        // Exact id match first.
        let mut hits: Vec<knowledge::SearchHit> = Vec::new();
        if let Ok(item) = self.get_active_knowledge(query)
            && item.project_id == project_id
            && (kind.is_empty() || item.kind == kind)
        {
            hits.push(knowledge::SearchHit {
                score: 100.0,
                snippet: item.summary.clone(),
                item,
                truth_class: "declared".to_string(),
                source: "knowledge".to_string(),
            });
        }

        let fts_query = sanitize_fts_query(query);
        if fts_query.is_empty() {
            return Ok(hits);
        }
        let mut sql_query = String::from(
            "SELECT knowledge_id, bm25(knowledge_fts) AS rank\n\t\tFROM knowledge_fts\n\t\tWHERE knowledge_fts MATCH ? AND project_id=?",
        );
        let mut args: Vec<Value> = vec![fts_query.into(), project_id.to_string().into()];
        if !kind.is_empty() {
            sql_query.push_str(" AND kind=?");
            args.push(kind.to_string().into());
        }
        sql_query.push_str(" ORDER BY rank LIMIT ?");
        args.push(i64::from(limit).into());
        let mut stmt = self.db.prepare(&sql_query)?;
        let mut rows = stmt.query(params_from_iter(args))?;
        let mut seen: HashSet<String> = hits.iter().map(|hit| hit.item.id.clone()).collect();
        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let rank: f64 = row.get(1)?;
            if seen.contains(&id) {
                continue;
            }
            let Ok(item) = self.get_active_knowledge(&id) else {
                continue;
            };
            let mut snippet = item.summary.clone();
            if snippet.is_empty() {
                snippet = item.title.clone();
            }
            hits.push(knowledge::SearchHit {
                item,
                score: -rank,
                snippet,
                truth_class: "declared".to_string(),
                source: "knowledge_fts".to_string(),
            });
            seen.insert(id);
        }
        Ok(hits)
    }

    /// Returns an item that is not archived; an archived one reads as missing,
    /// so search and context never surface it.
    pub(crate) fn get_active_knowledge(&self, id: &str) -> Result<knowledge::Item> {
        let item = self.get_knowledge(id)?;
        if item.archived() {
            return Err(Error::Sqlite(rusqlite::Error::QueryReturnedNoRows));
        }
        Ok(item)
    }
}

pub(crate) fn scan_canon_entity(row: &Row<'_>) -> rusqlite::Result<canon::Entity> {
    let observed: String = row.get(9)?;
    let created: String = row.get(10)?;
    let updated: String = row.get(11)?;
    Ok(canon::Entity {
        id: row.get(0)?,
        project_id: row.get(1)?,
        kind: row.get(2)?,
        ref_kind: row.get(3)?,
        ref_id: row.get(4)?,
        title: row.get(5)?,
        truth_class: row.get(6)?,
        source: row.get(7)?,
        actor: row.get(8)?,
        observed_at: super::parse_rfc3339(&observed),
        created_at: super::parse_rfc3339(&created),
        updated_at: super::parse_rfc3339(&updated),
    })
}

pub(crate) fn scan_canon_relationship(row: &Row<'_>) -> rusqlite::Result<canon::Relationship> {
    let observed: String = row.get(8)?;
    let created: String = row.get(9)?;
    Ok(canon::Relationship {
        id: row.get(0)?,
        project_id: row.get(1)?,
        subject_entity_id: row.get(2)?,
        predicate: row.get(3)?,
        object_entity_id: row.get(4)?,
        truth_class: row.get(5)?,
        source: row.get(6)?,
        actor: row.get(7)?,
        observed_at: super::parse_rfc3339(&observed),
        created_at: super::parse_rfc3339(&created),
    })
}

fn scan_knowledge_link(row: &Row<'_>) -> rusqlite::Result<knowledge::EntityLink> {
    let observed: String = row.get(8)?;
    let created: String = row.get(9)?;
    Ok(knowledge::EntityLink {
        id: row.get(0)?,
        project_id: row.get(1)?,
        knowledge_id: row.get(2)?,
        entity_id: row.get(3)?,
        predicate: row.get(4)?,
        truth_class: row.get(5)?,
        source: row.get(6)?,
        actor: row.get(7)?,
        observed_at: super::parse_rfc3339(&observed),
        created_at: super::parse_rfc3339(&created),
    })
}

/// Keeps the letters, digits, `_` and `-` of each term and makes every one a
/// prefix query, so punctuation a person types never breaks the index.
fn sanitize_fts_query(query: &str) -> String {
    let mut cleaned: Vec<String> = Vec::new();
    for part in query.split_whitespace() {
        let part: String = part
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect();
        if part.is_empty() {
            continue;
        }
        cleaned.push(format!("{part}*"));
    }
    cleaned.join(" ")
}

#[cfg(test)]
mod tests {
    use rusqlite::params;
    use time::OffsetDateTime;

    use crate::domains::canon;
    use crate::domains::knowledge;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    fn entity(id: &str, project_id: &str, title: &str, at: OffsetDateTime) -> canon::Entity {
        canon::Entity {
            id: id.to_string(),
            project_id: project_id.to_string(),
            kind: canon::KIND_COMPONENT.to_string(),
            ref_kind: canon::REF_ARCHITECTURE_NODE.to_string(),
            ref_id: format!("ref-{id}"),
            title: title.to_string(),
            truth_class: canon::TRUTH_DECLARED.to_string(),
            source: "test".to_string(),
            actor: "tester".to_string(),
            observed_at: at,
            created_at: at,
            updated_at: at,
        }
    }

    fn environment_entity(id: &str, project_id: &str, at: OffsetDateTime) -> canon::Entity {
        canon::Entity {
            kind: canon::KIND_ENVIRONMENT.to_string(),
            ref_kind: canon::REF_ENVIRONMENT.to_string(),
            ..entity(id, project_id, id, at)
        }
    }

    fn doc(
        id: &str,
        project_id: &str,
        title: &str,
        body: &str,
        at: OffsetDateTime,
    ) -> knowledge::Item {
        knowledge::Item {
            id: id.to_string(),
            project_id: project_id.to_string(),
            title: title.to_string(),
            body: body.to_string(),
            kind: knowledge::KIND_DOC.to_string(),
            doc_type: knowledge::DOC_TYPE_DOC.to_string(),
            summary: String::new(),
            created_at: at,
            updated_at: at,
            ..knowledge::Item::default()
        }
    }

    #[test]
    fn canon_entities_upsert_list_and_get() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        store
            .upsert_canon_entity(&entity("e1", "p", "Zeta", at))
            .expect("upsert zeta");
        store
            .upsert_canon_entity(&entity("e2", "p", "Alpha", at))
            .expect("upsert alpha");

        let listed = store.list_canon_entities("p", "").expect("entities");
        let titles: Vec<&str> = listed.iter().map(|item| item.title.as_str()).collect();
        assert_eq!(titles, vec!["Alpha", "Zeta"], "entities are in title order");
        assert!(
            store
                .list_canon_entities("p", canon::KIND_ENVIRONMENT)
                .expect("environments")
                .is_empty(),
            "a kind filter must drop the components"
        );

        let got = store.get_canon_entity("e1").expect("by id");
        assert_eq!(got.title, "Zeta");
        let got = store
            .get_canon_entity_by_ref("p", canon::REF_ARCHITECTURE_NODE, "ref-e1")
            .expect("by ref");
        assert_eq!(got.id, "e1");
        assert!(
            store
                .get_canon_entity_by_ref("p", canon::REF_ENVIRONMENT, "ref-e1")
                .expect_err("no environment ref")
                .is_no_rows()
        );

        // The same ref upserts: the title changes, the row count stays.
        let mut changed = entity("e1", "p", "Omega", at);
        changed.updated_at = at + time::Duration::seconds(1);
        store.upsert_canon_entity(&changed).expect("re-upsert");
        let listed = store.list_canon_entities("p", "").expect("entities");
        assert_eq!(listed.len(), 2, "upsert must not add a row: {listed:?}");
        assert_eq!(store.get_canon_entity("e1").expect("by id").title, "Omega");

        // Only a changed projection rewrites the row.
        let mut same = changed.clone();
        same.observed_at = at + time::Duration::seconds(9);
        same.updated_at = at + time::Duration::seconds(9);
        store
            .upsert_canon_entity_if_changed(&same)
            .expect("unchanged");
        assert_eq!(
            store.get_canon_entity("e1").expect("by id").updated_at,
            changed.updated_at,
            "an unchanged projection must keep its timestamps"
        );
        same.title = "Sigma".to_string();
        store
            .upsert_canon_entity_if_changed(&same)
            .expect("changed");
        assert_eq!(store.get_canon_entity("e1").expect("by id").title, "Sigma");
    }

    #[test]
    fn delete_canon_entities_missing_refs_keeps_the_listed() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        for id in ["e1", "e2", "e3"] {
            store
                .upsert_canon_entity(&entity(id, "p", id, at))
                .expect("upsert");
        }
        store
            .delete_canon_entities_missing_refs(
                "p",
                canon::REF_ARCHITECTURE_NODE,
                &["ref-e1".to_string(), "ref-e2".to_string()],
            )
            .expect("prune");
        let left: Vec<String> = store
            .list_canon_entities("p", "")
            .expect("entities")
            .into_iter()
            .map(|item| item.id)
            .collect();
        assert_eq!(left, vec!["e1".to_string(), "e2".to_string()]);
        store
            .delete_canon_entities_missing_refs("p", canon::REF_ARCHITECTURE_NODE, &[])
            .expect("clear");
        assert!(
            store
                .list_canon_entities("p", "")
                .expect("entities")
                .is_empty(),
            "an empty keep list clears the ref kind"
        );
    }

    #[test]
    fn canon_relationships_upsert_list_and_delete() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        for id in ["e1", "e2"] {
            store
                .upsert_canon_entity(&entity(id, "p", id, at))
                .expect("upsert");
        }
        let mut rel = canon::new_relationship(
            "p",
            "e1",
            canon::PRED_DOCUMENTS,
            "e2",
            canon::TRUTH_DECLARED,
            "test",
            "tester",
        )
        .expect("relationship");
        rel.observed_at = at;
        rel.created_at = at;
        store.upsert_canon_relationship(&rel).expect("upsert");

        assert_eq!(
            store.list_canon_relationships("p", "").expect("rels").len(),
            1
        );
        assert_eq!(
            store
                .list_canon_relationships("p", "e1")
                .expect("subject")
                .len(),
            1
        );
        assert_eq!(
            store
                .list_canon_relationships("p", "e2")
                .expect("object")
                .len(),
            1,
            "the object side of the relationship matches too"
        );
        assert!(
            store
                .list_canon_relationships("p", "e3")
                .expect("unknown")
                .is_empty()
        );
        let got = &store.list_canon_relationships("p", "").expect("rels")[0];
        assert_eq!(got.predicate, canon::PRED_DOCUMENTS);
        assert_eq!(got.subject_entity_id, "e1");
        assert_eq!(got.object_entity_id, "e2");

        // Re-provenance overwrites; an unchanged one leaves the stamp alone.
        let mut later = rel.clone();
        later.observed_at = at + time::Duration::seconds(5);
        store
            .upsert_canon_relationship_if_changed(&later)
            .expect("unchanged");
        assert_eq!(
            store.list_canon_relationships("p", "").expect("rels")[0].observed_at,
            rel.observed_at
        );
        later.actor = "erick".to_string();
        store
            .upsert_canon_relationship_if_changed(&later)
            .expect("changed");
        assert_eq!(
            store.list_canon_relationships("p", "").expect("rels")[0].actor,
            "erick"
        );

        store
            .delete_canon_relationships_missing("p", "test", &[])
            .expect("clear");
        assert!(
            store
                .list_canon_relationships("p", "")
                .expect("rels")
                .is_empty()
        );
    }

    #[test]
    fn knowledge_links_round_trip() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        store
            .upsert_canon_entity(&entity("e1", "p", "API", at))
            .expect("entity");
        store
            .save_knowledge(doc("doc-1", "p", "Doc", "Body", at))
            .expect("doc");
        let mut link =
            knowledge::new_entity_link("p", "doc-1", "e1", canon::PRED_DOCUMENTS, "", "", "")
                .expect("link");
        link.observed_at = at;
        link.created_at = at;
        store.save_knowledge_link(&link).expect("save");

        let got = store.get_knowledge_link(&link.id).expect("get");
        assert_eq!(got, link);
        assert_eq!(
            store
                .list_knowledge_links("p", "doc-1", "")
                .expect("by item")
                .len(),
            1
        );
        assert_eq!(
            store
                .list_knowledge_links("p", "", "e1")
                .expect("by entity")
                .len(),
            1
        );
        assert!(
            store
                .list_knowledge_links("p", "doc-2", "")
                .expect("other item")
                .is_empty()
        );
        store.delete_knowledge_link(&link.id).expect("delete");
        assert!(
            store
                .get_knowledge_link(&link.id)
                .expect_err("gone")
                .is_no_rows()
        );
    }

    #[test]
    fn search_knowledge_matches_ids_and_terms() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        store
            .save_knowledge(doc(
                "docdatabase",
                "p",
                "Database schema",
                "The database runs Postgres",
                at,
            ))
            .expect("doc");
        store
            .save_knowledge(doc("docother", "p", "Billing", "Invoices", at))
            .expect("other doc");

        // A hyphen in a term is kept, and FTS5 reads it as a column filter, so
        // the ids here have none.
        let by_id = store
            .search_knowledge("p", "docdatabase", "", 10)
            .expect("exact id");
        assert_eq!(by_id.len(), 1, "hits = {by_id:?}");
        assert_eq!(by_id[0].item.id, "docdatabase");
        assert_eq!(by_id[0].score, 100.0);
        assert_eq!(by_id[0].source, "knowledge");

        let by_term = store
            .search_knowledge("p", "database", "", 10)
            .expect("fts");
        assert_eq!(by_term.len(), 1, "hits = {by_term:?}");
        assert_eq!(by_term[0].item.id, "docdatabase");
        assert_eq!(by_term[0].source, "knowledge_fts");
        assert!(by_term[0].score > 0.0, "score = {}", by_term[0].score);

        // Punctuation is stripped per term, and the term stays a prefix query.
        for query in ["!database@", "datab", "database*"] {
            let hits = store.search_knowledge("p", query, "", 10).expect(query);
            assert_eq!(hits.len(), 1, "query {query:?} hits = {hits:?}");
        }
        assert!(
            store
                .search_knowledge("p", "!!!", "", 10)
                .expect("no terms")
                .is_empty(),
            "a query with no usable term finds nothing"
        );
        assert!(
            store
                .search_knowledge("p", "   ", "", 10)
                .expect("blank")
                .is_empty()
        );
        assert!(
            store
                .search_knowledge("p", "database", "evidence", 10)
                .expect("kind filter")
                .is_empty(),
            "the kind filter applies to the index"
        );
        assert!(
            store
                .search_knowledge("other", "database", "", 10)
                .expect("other project")
                .is_empty(),
            "search is scoped to the project"
        );
    }

    #[test]
    fn search_hides_archived_items() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        let item = doc("doc-1", "p", "Deploys", "The deploy target", at);
        let title = item.title.clone();
        store.save_knowledge(item).expect("doc");
        assert_eq!(
            store
                .search_knowledge("p", &title, "", 20)
                .expect("before")
                .len(),
            1
        );

        store
            .set_knowledge_archived("doc-1", true, at)
            .expect("archive");
        let hits = store.search_knowledge("p", &title, "", 20).expect("hits");
        assert!(
            hits.iter().all(|hit| hit.item.id != "doc-1"),
            "an archived item must leave search: {hits:?}"
        );
    }

    #[test]
    fn sync_project_model_projects_environments() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        for (id, name) in [("local", "Local"), ("staging", "Staging")] {
            store
                .save_environment(crate::domains::projects::Environment {
                    id: id.to_string(),
                    project_id: "p".to_string(),
                    name: name.to_string(),
                    kind: "local".to_string(),
                    ..crate::domains::projects::Environment::default()
                })
                .expect("environment");
        }
        let count = store.sync_project_model("p").expect("sync");
        assert_eq!(count, 2);
        let envs = store
            .list_canon_entities("p", canon::KIND_ENVIRONMENT)
            .expect("entities");
        let titles: Vec<&str> = envs.iter().map(|entity| entity.title.as_str()).collect();
        assert_eq!(titles, vec!["Local", "Staging"]);

        store.delete_environment("local", "p").expect("delete");
        let envs = store
            .list_canon_entities("p", canon::KIND_ENVIRONMENT)
            .expect("entities");
        assert_eq!(envs.len(), 1, "the deleted environment left Canon");
        assert_eq!(envs[0].ref_id, "staging");
        assert_eq!(
            store
                .db
                .query_row(
                    "SELECT COUNT(*) FROM canon_entities WHERE project_id=?",
                    params!["p"],
                    |row| row.get::<_, i64>(0)
                )
                .expect("count"),
            1,
            "a project with no environment keeps no environment entity"
        );
    }

    #[test]
    fn sanitize_fts_query_keeps_words_and_wildcards() {
        assert_eq!(sanitize_fts_query("Go modules"), "Go* modules*");
        assert_eq!(sanitize_fts_query("  a-b_c.d  "), "a-b_cd*");
        assert_eq!(sanitize_fts_query("!!!"), "");
    }

    #[test]
    fn an_environment_entity_lists_by_its_kind() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        store
            .upsert_canon_entity(&environment_entity("e1", "p", at))
            .expect("environment entity");
        let listed = store
            .list_canon_entities("p", canon::KIND_ENVIRONMENT)
            .expect("entities");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].ref_kind, canon::REF_ENVIRONMENT);
    }
}
