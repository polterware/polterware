//! Inbox dispositions, stored.
//!
//! What the Inbox keeps: a person's dispositions on derived items (append-only
//! rows, latest per key).

use std::collections::HashMap;

use rusqlite::params;

use crate::domains::inbox;
use crate::platform::ids;

use super::{Result, format_rfc3339_nano, parse_rfc3339};

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
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

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
                disposition("conflict:api", "defer", "2026-01-01T10:00:00Z"),
            )
            .expect("first disposition");
        store
            .insert_inbox_disposition(
                "proj",
                disposition("conflict:api", "ignore", "2026-01-01T10:00:01.5Z"),
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
        assert_eq!(listed[0].key, "conflict:api");
        assert_eq!(listed[0].action, "ignore");
        assert_eq!(listed[1].key, "question:a");
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
