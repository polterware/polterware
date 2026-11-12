//! Milestones, stored.
//!
//! Milestones group the cards that ship together. Every write records a
//! `change_ops` row in the same transaction, and archiving a milestone moves
//! its cards back to the Backlog with one audited card op each.

use rusqlite::{Connection, params};
use serde_json::{Map, Value};
use time::OffsetDateTime;

use crate::domains::work;
use crate::platform::ids;

use super::work::{go_json_fields, go_json_strings};
use super::{Error, Result, format_rfc3339, parse_rfc3339};

impl super::Store {
    /// Returns every milestone of the project in position order, archived ones
    /// included (their `archived_at` is set).
    pub fn list_milestones(&self, project_id: &str) -> Result<Vec<work::Milestone>> {
        query_milestones(&self.db, project_id)
    }

    /// Appends one milestone and records a `change_ops` create.
    pub fn create_milestone(
        &self,
        project_id: &str,
        agent_run_id: &str,
        batch_id: &str,
        item: work::Milestone,
    ) -> Result<work::Milestone> {
        let tx = self.db.unchecked_transaction()?;
        let existing = query_milestones(&tx, project_id)?;
        let mut item = item;
        item.project_id = project_id.to_string();
        let mut prepared = work::prepare_milestone_create(item, &existing)?;
        if prepared.id.is_empty() {
            prepared.id = ids::new_ulid();
        }
        let criteria = go_json_strings(&non_nil_strings(&prepared.exit_criteria));
        let now = format_rfc3339(OffsetDateTime::now_utc());
        if let Err(err) = tx.execute(
            r#"INSERT INTO milestones(id,project_id,slug,name,goal,exit_criteria_json,target_date,state,position,created_at,updated_at)
		VALUES(?,?,?,?,?,?,?,?,?,?,?)"#,
            params![
                prepared.id,
                project_id,
                prepared.slug,
                prepared.name,
                prepared.goal,
                criteria,
                prepared.target_date,
                prepared.state,
                prepared.position,
                now,
                now
            ],
        ) {
            if let Some(constraint) = milestone_constraint_error(&err) {
                return Err(constraint);
            }
            return Err(err.into());
        }
        let mut fields = Map::new();
        fields.insert("slug".to_string(), Value::from(prepared.slug.clone()));
        fields.insert("name".to_string(), Value::from(prepared.name.clone()));
        fields.insert("goal".to_string(), Value::from(prepared.goal.clone()));
        fields.insert(
            "exit_criteria".to_string(),
            serde_json::to_value(non_nil_strings(&prepared.exit_criteria)).unwrap_or_default(),
        );
        fields.insert(
            "target_date".to_string(),
            Value::from(prepared.target_date.clone()),
        );
        fields.insert("state".to_string(), Value::from(prepared.state.clone()));
        fields.insert("position".to_string(), Value::from(prepared.position));
        insert_milestone_op(
            &tx,
            project_id,
            agent_run_id,
            batch_id,
            &prepared.id,
            work::OP_CREATE,
            &fields,
            &now,
        )?;
        tx.commit()?;
        prepared.created_at = parse_rfc3339(&now);
        prepared.updated_at = prepared.created_at;
        Ok(prepared)
    }

    /// Replaces a milestone's name, goal, exit criteria, target date, and
    /// state. The slug stays. A no-op writes nothing.
    pub fn update_milestone(
        &self,
        project_id: &str,
        agent_run_id: &str,
        batch_id: &str,
        item: work::Milestone,
    ) -> Result<work::Milestone> {
        let tx = self.db.unchecked_transaction()?;
        let existing = query_milestones(&tx, project_id)?;
        let (next, changed) = work::prepare_milestone_update(item, &existing)?;
        if !changed {
            return Ok(next);
        }
        let mut current = work::Milestone::default();
        for other in &existing {
            if other.id == next.id {
                current = other.clone();
            }
        }
        let mut fields = Map::new();
        if next.name != current.name {
            fields.insert("name".to_string(), Value::from(next.name.clone()));
        }
        if next.goal != current.goal {
            fields.insert("goal".to_string(), Value::from(next.goal.clone()));
        }
        if next.exit_criteria.join("\n") != current.exit_criteria.join("\n") {
            fields.insert(
                "exit_criteria".to_string(),
                serde_json::to_value(non_nil_strings(&next.exit_criteria)).unwrap_or_default(),
            );
        }
        if next.target_date != current.target_date {
            fields.insert(
                "target_date".to_string(),
                Value::from(next.target_date.clone()),
            );
        }
        if next.state != current.state {
            fields.insert("state".to_string(), Value::from(next.state.clone()));
        }
        let criteria = go_json_strings(&non_nil_strings(&next.exit_criteria));
        let now = format_rfc3339(OffsetDateTime::now_utc());
        if let Err(err) = tx.execute(
            r#"UPDATE milestones SET name=?, goal=?, exit_criteria_json=?, target_date=?, state=?, updated_at=? WHERE id=? AND project_id=?"#,
            params![
                next.name,
                next.goal,
                criteria,
                next.target_date,
                next.state,
                now,
                next.id,
                project_id
            ],
        ) {
            if let Some(constraint) = milestone_constraint_error(&err) {
                return Err(constraint);
            }
            return Err(err.into());
        }
        insert_milestone_op(
            &tx,
            project_id,
            agent_run_id,
            batch_id,
            &next.id,
            work::OP_UPDATE,
            &fields,
            &now,
        )?;
        tx.commit()?;
        let mut next = next;
        next.updated_at = parse_rfc3339(&now);
        Ok(next)
    }

    /// Archives or restores a milestone. Its cards keep their milestone, so a
    /// restore brings them back with it.
    pub fn set_milestone_archived(
        &self,
        project_id: &str,
        agent_run_id: &str,
        batch_id: &str,
        id: &str,
        archived: bool,
    ) -> Result<work::Milestone> {
        let tx = self.db.unchecked_transaction()?;
        let existing = query_milestones(&tx, project_id)?;
        let Some(target) = existing.iter().find(|item| item.id == id).cloned() else {
            return Err(work::Error::NotFound.into());
        };
        if target.archived_at.is_empty() != archived {
            return Ok(target);
        }
        let now = format_rfc3339(OffsetDateTime::now_utc());
        let stamp = if archived { now.clone() } else { String::new() };
        tx.execute(
            r#"UPDATE milestones SET archived_at=?, updated_at=? WHERE id=? AND project_id=?"#,
            params![stamp, now, id, project_id],
        )?;
        let mut fields = Map::new();
        fields.insert("archived_at".to_string(), Value::from(stamp.clone()));
        insert_milestone_op(
            &tx,
            project_id,
            agent_run_id,
            batch_id,
            id,
            work::OP_UPDATE,
            &fields,
            &now,
        )?;
        if archived {
            move_milestone_cards_to_backlog(&tx, project_id, agent_run_id, batch_id, id, &now)?;
        }
        tx.commit()?;
        let mut target = target;
        target.archived_at = stamp;
        Ok(target)
    }

    /// Sets positions from `ordered_ids`, which must list each listed
    /// (unarchived) milestone once. An order that changes nothing writes no
    /// rows.
    pub fn reorder_milestones(
        &self,
        project_id: &str,
        agent_run_id: &str,
        batch_id: &str,
        ordered_ids: &[String],
    ) -> Result<Vec<work::Milestone>> {
        let tx = self.db.unchecked_transaction()?;
        let all = query_milestones(&tx, project_id)?;
        let listed: Vec<work::Milestone> = all
            .iter()
            .filter(|item| item.archived_at.is_empty())
            .cloned()
            .collect();
        let next = work::reorder_milestones(&listed, ordered_ids)?;
        if !work::milestone_positions_changed(&listed, &next) {
            return Ok(all);
        }
        let now = format_rfc3339(OffsetDateTime::now_utc());
        for item in &next {
            let current = listed
                .iter()
                .find(|other| other.id == item.id)
                .map(|other| other.position)
                .unwrap_or(0);
            if current == item.position {
                continue;
            }
            tx.execute(
                r#"UPDATE milestones SET position=?, updated_at=? WHERE id=? AND project_id=?"#,
                params![item.position, now, item.id, project_id],
            )?;
            let mut fields = Map::new();
            fields.insert("position".to_string(), Value::from(item.position));
            insert_milestone_op(
                &tx,
                project_id,
                agent_run_id,
                batch_id,
                &item.id,
                work::OP_UPDATE,
                &fields,
                &now,
            )?;
        }
        tx.commit()?;
        query_milestones(&self.db, project_id)
    }
}

/// Reads the project's milestones in position order, from a connection or a
/// transaction.
fn query_milestones(q: &Connection, project_id: &str) -> Result<Vec<work::Milestone>> {
    let mut stmt = q.prepare(
        r#"SELECT id,project_id,slug,name,goal,exit_criteria_json,target_date,state,position,archived_at,created_at,updated_at
		FROM milestones WHERE project_id=? ORDER BY position, id"#,
    )?;
    let mut rows = stmt.query(params![project_id])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        let criteria: String = row.get(5)?;
        let created: String = row.get(10)?;
        let updated: String = row.get(11)?;
        out.push(work::Milestone {
            id: row.get(0)?,
            project_id: row.get(1)?,
            slug: row.get(2)?,
            name: row.get(3)?,
            goal: row.get(4)?,
            exit_criteria: serde_json::from_str::<Vec<String>>(&criteria).unwrap_or_default(),
            target_date: row.get(6)?,
            state: row.get(7)?,
            position: row.get(8)?,
            archived_at: row.get(9)?,
            created_at: parse_rfc3339(&created),
            updated_at: parse_rfc3339(&updated),
        });
    }
    Ok(out)
}

/// Clears an archived milestone from its cards, one audited card op each, so no
/// card is left in a milestone nobody can open.
fn move_milestone_cards_to_backlog(
    tx: &Connection,
    project_id: &str,
    agent_run_id: &str,
    batch_id: &str,
    milestone_id: &str,
    now: &str,
) -> Result<()> {
    let mut card_ids: Vec<String> = Vec::new();
    {
        let mut stmt = tx.prepare(
            r#"SELECT id FROM work_cards WHERE project_id=? AND milestone_id=? ORDER BY rowid"#,
        )?;
        let mut rows = stmt.query(params![project_id, milestone_id])?;
        while let Some(row) = rows.next()? {
            card_ids.push(row.get(0)?);
        }
    }
    for card_id in card_ids {
        tx.execute(
            r#"UPDATE work_cards SET milestone_id=NULL, updated_at=? WHERE id=?"#,
            params![now, card_id],
        )?;
        tx.execute(
            r#"INSERT INTO change_ops(id,project_id,agent_run_id,batch_id,entity,entity_id,op,data_json,node_id,created_at)
			VALUES(?,?,?,?,?,?,?,?,?,?)"#,
            params![
                ids::new_ulid(),
                project_id,
                agent_run_id,
                batch_id,
                work::ENTITY_CARD,
                card_id,
                work::OP_UPDATE,
                r#"{"milestone":""}"#,
                "",
                now
            ],
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn insert_milestone_op(
    tx: &Connection,
    project_id: &str,
    agent_run_id: &str,
    batch_id: &str,
    entity_id: &str,
    kind: &str,
    fields: &Map<String, Value>,
    now: &str,
) -> Result<()> {
    let data = go_json_fields(fields);
    tx.execute(
        r#"INSERT INTO change_ops(id,project_id,agent_run_id,batch_id,entity,entity_id,op,data_json,node_id,created_at)
		VALUES(?,?,?,?,?,?,?,?,?,?)"#,
        params![
            ids::new_ulid(),
            project_id,
            agent_run_id,
            batch_id,
            work::ENTITY_MILESTONE,
            entity_id,
            kind,
            data,
            "",
            now
        ],
    )?;
    Ok(())
}

/// The name is the one unique key a person can hit: the slug is derived.
fn milestone_constraint_error(err: &rusqlite::Error) -> Option<Error> {
    if err.to_string().contains("milestones.name") {
        return Some(work::Error::NameTaken.into());
    }
    None
}

fn non_nil_strings(values: &[String]) -> Vec<String> {
    values.to_vec()
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, json};
    use time::OffsetDateTime;

    use crate::domains::work;
    use crate::providers::sqlite::Error;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    fn card_op(id: &str, slug: &str, stage: &str, milestone: &str) -> work::Op {
        let mut fields = Map::new();
        fields.insert("slug".to_string(), json!(slug));
        fields.insert("title".to_string(), json!(slug.to_uppercase()));
        fields.insert("stage".to_string(), json!(stage));
        fields.insert("status".to_string(), json!(work::CARD_ACTIVE));
        fields.insert("milestone".to_string(), json!(milestone));
        work::Op {
            entity: work::ENTITY_CARD.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: id.to_string(),
            card_id: id.to_string(),
            card_slug: slug.to_string(),
            label: slug.to_string(),
            fields,
            ..work::Op::default()
        }
    }

    fn card_milestones(snapshot: &work::Snapshot) -> std::collections::HashMap<String, String> {
        let mut out = std::collections::HashMap::new();
        for card in &snapshot.cards {
            out.insert(card.slug.clone(), card.milestone.clone());
        }
        out
    }

    #[test]
    fn milestone_store_archive_moves_cards_to_backlog() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", now());
        let beta = store
            .create_milestone(
                "proj",
                "",
                "batch-1",
                work::Milestone {
                    slug: "beta".to_string(),
                    name: "Beta".to_string(),
                    state: work::MILESTONE_ACTIVE.to_string(),
                    exit_criteria: vec!["ships".to_string()],
                    ..work::Milestone::default()
                },
            )
            .expect("create milestone");
        assert_eq!(beta.slug, "beta", "the slug comes from the name");
        assert_eq!(beta.position, 0);

        store
            .apply_work_ops(
                "proj",
                "",
                "batch-2",
                &[
                    card_op("card-a", "a", work::STAGE_BUILD, &beta.id),
                    card_op("card-b", "b", work::STAGE_SHAPE, ""),
                ],
            )
            .expect("cards");
        let snapshot = store.load_work_snapshot("proj").expect("snapshot");
        let got = card_milestones(&snapshot);
        assert_eq!(got.get("a").map(String::as_str), Some("beta"));
        assert_eq!(got.get("b").map(String::as_str), Some(""));

        let archived = store
            .set_milestone_archived("proj", "", "batch-3", &beta.id, true)
            .expect("archive");
        assert!(!archived.archived_at.is_empty(), "archived = {archived:?}");
        let snapshot = store.load_work_snapshot("proj").expect("snapshot");
        let got = card_milestones(&snapshot);
        assert_eq!(
            got.get("a").map(String::as_str),
            Some(""),
            "an archived milestone keeps no cards"
        );
        assert_eq!(got.get("b").map(String::as_str), Some(""));
        let ops: i64 = store
            .db
            .query_row(
                r#"SELECT COUNT(*) FROM change_ops WHERE batch_id='batch-3' AND entity='card' AND entity_id='card-a' AND data_json='{"milestone":""}'"#,
                [],
                |row| row.get(0),
            )
            .expect("card move ops");
        assert_eq!(ops, 1);

        let restored = store
            .set_milestone_archived("proj", "", "batch-4", &beta.id, false)
            .expect("restore");
        assert!(restored.archived_at.is_empty(), "restored = {restored:?}");
        let err = store
            .set_milestone_archived("proj", "", "batch-5", "missing", true)
            .expect_err("missing milestone");
        assert!(
            matches!(err, Error::Work(work::Error::NotFound)),
            "err = {err:?}"
        );
        assert_eq!(err.to_string(), "milestone not found");
    }

    #[test]
    fn milestone_create_and_update_keep_the_slug() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", now());
        let beta = store
            .create_milestone(
                "proj",
                "",
                "batch-1",
                work::Milestone {
                    name: "Beta".to_string(),
                    ..work::Milestone::default()
                },
            )
            .expect("create");
        let err = store
            .create_milestone(
                "proj",
                "",
                "batch-2",
                work::Milestone {
                    name: " beta ".to_string(),
                    ..work::Milestone::default()
                },
            )
            .expect_err("duplicate name");
        assert_eq!(
            err.to_string(),
            "milestone name is already used in this project"
        );

        let updated = store
            .update_milestone(
                "proj",
                "",
                "batch-3",
                work::Milestone {
                    id: beta.id.clone(),
                    name: "Public beta".to_string(),
                    state: work::MILESTONE_DONE.to_string(),
                    ..work::Milestone::default()
                },
            )
            .expect("update");
        assert_eq!(updated.slug, "beta", "the slug stays");
        assert_eq!(updated.name, "Public beta");
        assert_eq!(updated.state, work::MILESTONE_DONE);

        let listed = store.list_milestones("proj").expect("milestones");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Public beta");
    }

    #[test]
    fn milestone_reorder_sets_positions() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", now());
        let mut ids = Vec::new();
        for name in ["One", "Two"] {
            let item = store
                .create_milestone(
                    "proj",
                    "",
                    "batch-1",
                    work::Milestone {
                        name: name.to_string(),
                        ..work::Milestone::default()
                    },
                )
                .expect("create");
            ids.push(item.id);
        }
        let ordered = vec![ids[1].clone(), ids[0].clone()];
        let listed = store
            .reorder_milestones("proj", "", "batch-2", &ordered)
            .expect("reorder");
        assert_eq!(listed[0].id, ids[1]);
        assert_eq!(listed[0].position, 0);
        assert_eq!(listed[1].position, 1);

        let err = store
            .reorder_milestones("proj", "", "batch-3", &ordered[..1])
            .expect_err("an order must list every milestone");
        assert_eq!(
            err.to_string(),
            "milestone order must list each milestone of the project once"
        );
    }
}
