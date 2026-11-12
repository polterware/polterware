//! The per-hook switches, stored.
//!
//! The agent hooks a person can turn off. A hook with no row is on.

use std::collections::HashMap;

use rusqlite::params;
use time::OffsetDateTime;

use super::{Result, format_rfc3339};

impl super::Store {
    /// Returns the stored switches; a hook with no row is on.
    pub fn hook_switches(&self) -> Result<HashMap<String, bool>> {
        let mut stmt = self
            .db
            .prepare(r#"SELECT name, enabled FROM agent_hook_switches"#)?;
        let rows = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let enabled: i64 = row.get(1)?;
            Ok((name, enabled != 0))
        })?;
        let mut out = HashMap::new();
        for row in rows {
            let (name, enabled) = row?;
            out.insert(name, enabled);
        }
        Ok(out)
    }

    /// Turns one hook on or off.
    pub fn set_hook_switch(&self, name: &str, enabled: bool) -> Result<()> {
        let value = i64::from(enabled);
        self.db.execute(
            r#"INSERT INTO agent_hook_switches(name, enabled, updated_at) VALUES(?,?,?)
		 ON CONFLICT(name) DO UPDATE SET enabled=excluded.enabled, updated_at=excluded.updated_at"#,
            params![name, value, format_rfc3339(OffsetDateTime::now_utc())],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::providers::sqlite::test_support::TempStore;

    #[test]
    fn hook_switches_default_on_and_round_trip() {
        let holder = TempStore::new();
        let store = &holder.store;
        let switches = store.hook_switches().expect("switches");
        assert!(
            switches.is_empty(),
            "a fresh store must have no rows (hooks default on): {switches:?}"
        );
        store.set_hook_switch("session-start", false).expect("off");
        store.set_hook_switch("stop", true).expect("on");
        // Updating an existing switch must not create a second row.
        store.set_hook_switch("session-start", true).expect("on");
        store.set_hook_switch("session-start", false).expect("off");
        let switches = store.hook_switches().expect("switches");
        assert_eq!(switches.len(), 2, "switches = {switches:?}");
        assert_eq!(switches.get("session-start"), Some(&false));
        assert_eq!(switches.get("stop"), Some(&true));
    }
}
