//! Port of `backend/providers/sqlite/canon_facts.go`.
//!
//! Canon facts in the store: a statement with its truth class and provenance,
//! optionally about one entity.

use rusqlite::{Connection, Row, params};

use crate::domains::canon;

use super::{Result, format_rfc3339};

impl super::Store {
    pub fn upsert_canon_fact(&self, fact: &canon::Fact) -> Result<()> {
        upsert_canon_fact(&self.db, fact)
    }

    /// Returns the project's facts, filtered by entity when it is not empty:
    /// newest observation first.
    pub fn list_canon_facts(&self, project_id: &str, entity_id: &str) -> Result<Vec<canon::Fact>> {
        let mut query = String::from(
            "SELECT id,project_id,statement,truth_class,source,actor,observed_at,entity_id,created_at,updated_at\n\t\tFROM canon_facts WHERE project_id=?",
        );
        if !entity_id.is_empty() {
            query.push_str(" AND entity_id=?");
        }
        query.push_str(" ORDER BY observed_at DESC, created_at DESC");
        let mut stmt = self.db.prepare(&query)?;
        let mut rows = if entity_id.is_empty() {
            stmt.query(params![project_id])?
        } else {
            stmt.query(params![project_id, entity_id])?
        };
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push(scan_canon_fact(row)?);
        }
        Ok(result)
    }

    pub fn get_canon_fact(&self, id: &str) -> Result<canon::Fact> {
        self.db
            .query_row(
                "SELECT id,project_id,statement,truth_class,source,actor,observed_at,entity_id,created_at,updated_at\n\t\tFROM canon_facts WHERE id=?",
                params![id],
                scan_canon_fact,
            )
            .map_err(Into::into)
    }
}

/// Writes a fact, replacing the row of its id. The Work transaction calls it so
/// facts and audit ops commit together.
pub(crate) fn upsert_canon_fact(db: &Connection, fact: &canon::Fact) -> Result<()> {
    let entity_id = if fact.entity_id.is_empty() {
        None
    } else {
        Some(&fact.entity_id)
    };
    db.execute(
        r#"INSERT INTO canon_facts(
		id,project_id,statement,truth_class,source,actor,observed_at,entity_id,created_at,updated_at
	) VALUES(?,?,?,?,?,?,?,?,?,?)
	ON CONFLICT(id) DO UPDATE SET
		statement=excluded.statement,
		truth_class=excluded.truth_class,
		source=excluded.source,
		actor=excluded.actor,
		observed_at=excluded.observed_at,
		entity_id=excluded.entity_id,
		updated_at=excluded.updated_at"#,
        params![
            fact.id,
            fact.project_id,
            fact.statement,
            fact.truth_class,
            fact.source,
            fact.actor,
            format_rfc3339(fact.observed_at),
            entity_id,
            format_rfc3339(fact.created_at),
            format_rfc3339(fact.updated_at),
        ],
    )?;
    Ok(())
}

pub(crate) fn scan_canon_fact(row: &Row<'_>) -> rusqlite::Result<canon::Fact> {
    let observed: String = row.get(6)?;
    let entity_id: Option<String> = row.get(7)?;
    let created: String = row.get(8)?;
    let updated: String = row.get(9)?;
    Ok(canon::Fact {
        id: row.get(0)?,
        project_id: row.get(1)?,
        statement: row.get(2)?,
        truth_class: row.get(3)?,
        source: row.get(4)?,
        actor: row.get(5)?,
        observed_at: super::parse_rfc3339(&observed),
        entity_id: entity_id.unwrap_or_default(),
        created_at: super::parse_rfc3339(&created),
        updated_at: super::parse_rfc3339(&updated),
    })
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    use crate::domains::canon;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    fn save_entity(store: &super::super::Store, id: &str, at: OffsetDateTime) {
        store
            .upsert_canon_entity(&canon::Entity {
                id: id.to_string(),
                project_id: "p".to_string(),
                kind: canon::KIND_COMPONENT.to_string(),
                ref_kind: canon::REF_ARCHITECTURE_NODE.to_string(),
                ref_id: id.to_string(),
                title: id.to_string(),
                truth_class: canon::TRUTH_DECLARED.to_string(),
                source: "test".to_string(),
                actor: "tester".to_string(),
                observed_at: at,
                created_at: at,
                updated_at: at,
            })
            .expect("entity");
    }

    fn fact(id: &str, project_id: &str, statement: &str, at: OffsetDateTime) -> canon::Fact {
        canon::Fact {
            id: id.to_string(),
            project_id: project_id.to_string(),
            statement: statement.to_string(),
            truth_class: canon::TRUTH_DECLARED.to_string(),
            source: "polterware".to_string(),
            actor: "system".to_string(),
            observed_at: at,
            created_at: at,
            updated_at: at,
            ..canon::Fact::default()
        }
    }

    #[test]
    fn canon_facts_round_trip() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        save_entity(store, "ent-1", at);
        store
            .upsert_canon_fact(&fact("fact-1", "p", "Uses Go modules", at))
            .expect("upsert");
        let mut second = fact(
            "fact-2",
            "p",
            "Deploys through Vercel",
            at + time::Duration::seconds(1),
        );
        second.entity_id = "ent-1".to_string();
        store.upsert_canon_fact(&second).expect("upsert second");

        let listed = store.list_canon_facts("p", "").expect("facts");
        let ids: Vec<&str> = listed.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(ids, vec!["fact-2", "fact-1"], "newest observation first");
        assert_eq!(listed[0].entity_id, "ent-1");
        assert_eq!(listed[1].entity_id, "", "a missing entity reads as empty");

        let got = store.get_canon_fact("fact-1").expect("by id");
        assert_eq!(got, fact("fact-1", "p", "Uses Go modules", at));
        assert!(
            store
                .get_canon_fact("missing")
                .expect_err("unknown")
                .is_no_rows()
        );

        let scoped = store.list_canon_facts("p", "ent-1").expect("by entity");
        assert_eq!(scoped.len(), 1);
        assert_eq!(scoped[0].id, "fact-2");
        assert!(
            store
                .list_canon_facts("p", "ent-2")
                .expect("other entity")
                .is_empty()
        );

        // Upsert keeps one row per id and updates the statement in place.
        let mut changed = fact("fact-1", "p", "Uses Cargo", at);
        changed.updated_at = at + time::Duration::seconds(5);
        store.upsert_canon_fact(&changed).expect("update");
        assert_eq!(
            store.list_canon_facts("p", "").expect("facts").len(),
            2,
            "upsert must not add a row"
        );
        assert_eq!(
            store.get_canon_fact("fact-1").expect("by id").statement,
            "Uses Cargo"
        );
    }

    #[test]
    fn canon_facts_drop_the_entity_when_emptied() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        save_entity(store, "ent-1", at);
        let mut item = fact("fact-1", "p", "One core", at);
        item.entity_id = "ent-1".to_string();
        store.upsert_canon_fact(&item).expect("with entity");
        assert_eq!(
            store.get_canon_fact("fact-1").expect("by id").entity_id,
            "ent-1"
        );
        item.entity_id = String::new();
        store.upsert_canon_fact(&item).expect("without entity");
        assert_eq!(
            store.get_canon_fact("fact-1").expect("by id").entity_id,
            "",
            "an empty entity must be stored as NULL"
        );
    }
}
