//! The demo seed's tests.

use std::collections::HashSet;

use crate::domains::knowledge;
use crate::providers::sqlite::{Store, test_support::TempStore};

use super::{EMPTY_PROJECT_ID, PROJECT_ID, defaults};

fn new_seeded() -> TempStore {
    let holder = TempStore::named("seed.db");
    defaults(&holder.store).expect("Defaults");
    holder
}

fn count(store: &Store, query: &str, args: &[&dyn rusqlite::ToSql]) -> i64 {
    store
        .db
        .query_row(
            query,
            rusqlite::params_from_iter(args.iter().copied()),
            |row| row.get(0),
        )
        .unwrap_or_else(|err| panic!("{query}: {err}"))
}

#[test]
fn defaults_creates_polter_and_empty_projects() {
    let holder = new_seeded();
    let store = &holder.store;
    for id in [PROJECT_ID, EMPTY_PROJECT_ID] {
        if let Err(err) = store.get_project(id) {
            panic!("project {id:?} missing: {err}");
        }
    }
    let envs = store
        .list_environments(PROJECT_ID)
        .expect("list environments");
    assert!(!envs.is_empty(), "seeded project needs environments");
    let defaults = envs.iter().filter(|env| env.is_default).count();
    assert_eq!(
        defaults, 1,
        "want exactly one default environment, got {defaults}"
    );
}

#[test]
fn defaults_is_idempotent() {
    let holder = new_seeded();
    let store = &holder.store;
    let tables = [
        "projects",
        "environments",
        "knowledge_items",
        "work_cards",
        "work_tasks",
        "rules",
    ];
    let before: Vec<i64> = tables
        .iter()
        .map(|table| count(store, &format!("SELECT COUNT(*) FROM {table}"), &[]))
        .collect();
    defaults(store).expect("second Defaults");
    for (table, before) in tables.iter().zip(before) {
        let after = count(store, &format!("SELECT COUNT(*) FROM {table}"), &[]);
        assert_eq!(after, before, "{table} changed on a second seed");
    }
}

#[test]
fn defaults_does_not_overwrite_an_existing_database() {
    let holder = TempStore::named("user.db");
    let store = &holder.store;
    store
        .db
        .execute(
            r#"INSERT INTO projects(id,name,created_at,updated_at,status,archived_at)
		VALUES('proj-user','Mine','2026-01-01T00:00:00Z','2026-01-01T00:00:00Z','active','')"#,
            [],
        )
        .expect("insert a user project");
    defaults(store).expect("Defaults");
    let projects = count(store, "SELECT COUNT(*) FROM projects", &[]);
    assert_eq!(
        projects, 1,
        "seed must leave a non-empty database alone, projects = {projects}"
    );
}

#[test]
fn seeded_rules_cover_the_lifecycle_states() {
    let holder = new_seeded();
    let store = &holder.store;
    let rules = store.list_rules(PROJECT_ID).expect("list rules");
    assert!(
        rules.len() >= 3,
        "want at least 3 seeded Rules, got {}",
        rules.len()
    );
    let states: HashSet<&str> = rules.iter().map(|rule| rule.status.as_str()).collect();
    assert!(
        states.contains("active") && states.contains("proposed"),
        "seed must show active and proposed Rules, got {states:?}"
    );
}

#[test]
fn seed_registers_no_source_docs() {
    let holder = new_seeded();
    let store = &holder.store;
    let items = store
        .list_knowledge(PROJECT_ID, knowledge::KIND_DOC)
        .expect("list docs");
    for item in items {
        assert_eq!(
            item.doc_type,
            knowledge::DOC_TYPE_DOC,
            "doc {} has type {:?}: docs have no types",
            item.id,
            item.doc_type
        );
    }
}

#[test]
fn empty_project_has_no_content() {
    let holder = new_seeded();
    let store = &holder.store;
    for table in ["knowledge_items", "work_cards", "rules"] {
        let rows = count(
            store,
            &format!("SELECT COUNT(*) FROM {table} WHERE project_id=?"),
            &[&EMPTY_PROJECT_ID],
        );
        assert_eq!(rows, 0, "empty project has {rows} rows in {table}");
    }
}
