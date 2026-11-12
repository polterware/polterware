use std::collections::BTreeSet;

use rusqlite::Connection;

use super::migrate::MIGRATIONS;
use super::*;

fn migration_body(name: &str) -> &'static str {
    MIGRATIONS
        .iter()
        .find(|(file, _)| *file == name)
        .map(|(_, body)| *body)
        .unwrap_or_else(|| panic!("no embedded migration {name}"))
}

fn apply_migration(conn: &mut Connection, name: &str) {
    let body = migration_body(name);
    conn.execute_batch(body)
        .unwrap_or_else(|err| panic!("{name}: {err}"));
}

fn migrate_before(conn: &mut Connection, prefix: &str) {
    for (name, body) in MIGRATIONS {
        if *name >= prefix {
            continue;
        }
        conn.execute_batch(body)
            .unwrap_or_else(|err| panic!("{name}: {err}"));
    }
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |row| row.get(0))
        .unwrap_or_else(|err| panic!("{sql}: {err}"))
}

fn assert_no_foreign_key_violations(conn: &Connection) {
    let mut stmt = conn
        .prepare("PRAGMA foreign_key_check")
        .expect("prepare foreign_key_check");
    let mut rows = stmt.query([]).expect("run foreign_key_check");
    assert!(
        rows.next().expect("next").is_none(),
        "foreign_key_check reported violations"
    );
}

#[derive(Debug, PartialEq, Eq)]
struct RuleRow {
    id: String,
    project_id: String,
    slug: Option<String>,
    statement: String,
    why: String,
    status: String,
    replaced_by: Option<String>,
    position: i64,
}

fn load_rule(db: &Connection, id: &str) -> RuleRow {
    db.query_row(
        "SELECT id, project_id, slug, statement, why, status, replaced_by, position FROM rules WHERE id = ?",
        [id],
        |row| {
            Ok(RuleRow {
                id: row.get(0)?,
                project_id: row.get(1)?,
                slug: row.get(2)?,
                statement: row.get(3)?,
                why: row.get(4)?,
                status: row.get(5)?,
                replaced_by: row.get(6)?,
                position: row.get(7)?,
            })
        },
    )
    .unwrap_or_else(|err| panic!("load rule {id}: {err}"))
}

fn load_contexts(db: &Connection, entity_id: &str) -> BTreeSet<String> {
    let mut stmt = db
        .prepare("SELECT node_id FROM entity_contexts WHERE entity_id = ? ORDER BY node_id")
        .expect("prepare contexts");
    let rows = stmt
        .query_map([entity_id], |row| row.get::<_, String>(0))
        .expect("query contexts");
    rows.map(|r| r.expect("context node")).collect()
}

#[test]
fn rules_unify_migration_converts_decisions_principles_and_facts() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut db = open(tmp.path().join("unify.db")).expect("open");

    // 1. Build the database at the previous migration (0057).
    migrate_before(&mut db, "0058");

    // 2. Seed data before migration 0058:
    // - 1 Project
    // - 2 Structure nodes
    // - 1 Existing active rule (to test fact duplication against existing rule)
    // - 3 Decisions: accepted, challenged, superseded by another
    // - 1 Principle (unscoped decision)
    // - 2 Facts: one duplicating the existing active rule, one unique
    // - 1 Document (to verify doc rows in knowledge_items are preserved)
    let seed_statements = [
        "INSERT INTO projects(id, name, created_at, updated_at, status, archived_at)
         VALUES('p1', 'Rules Project', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 'active', '')",

        "INSERT INTO structure_nodes(id, project_id, level, slug, path, title, created_at, updated_at)
         VALUES('node-api', 'p1', 'component', 'api', 'backend/api', 'API', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",

        "INSERT INTO structure_nodes(id, project_id, level, slug, path, title, created_at, updated_at)
         VALUES('node-ui', 'p1', 'component', 'ui', 'frontend/ui', 'UI', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",

        // Existing active rule with context on node-api:
        "INSERT INTO rules(id, project_id, slug, statement, status, position, created_at, updated_at)
         VALUES('r-existing', 'p1', 'one-process', 'One process owns SQLite.', 'active', 0, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        "INSERT INTO entity_contexts(project_id, entity_id, node_id)
         VALUES('p1', 'r-existing', 'node-api')",

        // Decision 1: accepted
        r#"INSERT INTO knowledge_items(id, project_id, title, body, kind, status, extra_json, created_at, updated_at)
         VALUES('d-accepted', 'p1', 'Use SQLite for storage', 'SQLite is embedded and fast.', 'decision', 'accepted',
                '{"reviewTrigger":"multi-writer needed","statusReason":"benchmarks approved"}',
                '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z')"#,
        "INSERT INTO entity_contexts(project_id, entity_id, node_id)
         VALUES('p1', 'd-accepted', 'node-api')",

        // Decision 2: challenged
        r#"INSERT INTO knowledge_items(id, project_id, title, body, kind, status, extra_json, created_at, updated_at)
         VALUES('d-challenged', 'p1', 'Local HTTP for IPC', 'Use HTTP between desktop and core.', 'decision', 'challenged',
                '{"reviewTrigger":"IPC latency > 5ms","statusReason":"named pipes proposed"}',
                '2026-01-03T00:00:00Z', '2026-01-03T00:00:00Z')"#,
        "INSERT INTO entity_contexts(project_id, entity_id, node_id)
         VALUES('p1', 'd-challenged', 'node-ui')",

        // Decision 3: superseded by d-accepted
        r#"INSERT INTO knowledge_items(id, project_id, title, body, kind, status, extra_json, created_at, updated_at)
         VALUES('d-superseded', 'p1', 'Raw TCP sockets', 'Raw TCP connections for everything.', 'decision', 'superseded',
                '{"supersededBy":"d-accepted","reviewTrigger":"too brittle","statusReason":"HTTP adopted instead"}',
                '2026-01-04T00:00:00Z', '2026-01-04T00:00:00Z')"#,
        "INSERT INTO entity_contexts(project_id, entity_id, node_id)
         VALUES('p1', 'd-superseded', 'node-api')",

        // Principle: decision with no contexts (unscoped)
        r#"INSERT INTO knowledge_items(id, project_id, title, body, kind, status, extra_json, created_at, updated_at)
         VALUES('principle-local-first', 'p1', 'Local first architecture', 'All data stays on the machine.', 'decision', 'active',
                '{"reviewTrigger":"cloud backup added","statusReason":"core philosophy"}',
                '2026-01-05T00:00:00Z', '2026-01-05T00:00:00Z')"#,

        // Fact 1: duplicates r-existing (case-insensitive, no final period)
        "INSERT INTO canon_facts(id, project_id, statement, truth_class, source, actor, observed_at, created_at, updated_at)
         VALUES('fact-dup', 'p1', 'one process owns sqlite', 'declared', 'polterware', 'agent', '2026-01-06T00:00:00Z', '2026-01-06T00:00:00Z', '2026-01-06T00:00:00Z')",
        "INSERT INTO entity_contexts(project_id, entity_id, node_id)
         VALUES('p1', 'fact-dup', 'node-api')",

        // Fact 2: unique fact with context on node-ui
        "INSERT INTO canon_facts(id, project_id, slug, statement, truth_class, source, actor, observed_at, created_at, updated_at)
         VALUES('fact-unique', 'p1', 'backend-local', 'Backends run locally.', 'declared', 'polterware', 'agent', '2026-01-07T00:00:00Z', '2026-01-07T00:00:00Z', '2026-01-07T00:00:00Z')",
        "INSERT INTO entity_contexts(project_id, entity_id, node_id)
         VALUES('p1', 'fact-unique', 'node-ui')",

        // A regular doc item in knowledge_items: must remain intact
        "INSERT INTO knowledge_items(id, project_id, title, body, kind, doc_type, created_at, updated_at)
         VALUES('doc-arch', 'p1', 'Architecture Guide', 'System overview.', 'doc', 'doc', '2026-01-08T00:00:00Z', '2026-01-08T00:00:00Z')",
        "INSERT INTO entity_contexts(project_id, entity_id, node_id)
         VALUES('p1', 'doc-arch', 'node-api')",
    ];

    for stmt in seed_statements {
        db.execute(stmt, [])
            .unwrap_or_else(|err| panic!("execute {stmt}: {err}"));
    }

    // 3. Apply migration 0058_rules_unify.sql.
    apply_migration(&mut db, "0058_rules_unify.sql");

    // 4. Verify the database state:

    // 4.1. Rules count:
    // Expected in rules:
    // - r-existing (existing rule)
    // - d-accepted (migrated decision)
    // - d-challenged (migrated decision)
    // - d-superseded (migrated decision)
    // - principle-local-first (migrated principle)
    // - fact-unique (migrated unique fact)
    // (fact-dup was dropped because it duplicated r-existing)
    let total_rules = count(&db, "SELECT COUNT(*) FROM rules");
    assert_eq!(total_rules, 6, "expected 6 rules, got {total_rules}");

    // 4.2. Existing rule:
    let r_existing = load_rule(&db, "r-existing");
    assert_eq!(
        r_existing,
        RuleRow {
            id: "r-existing".to_string(),
            project_id: "p1".to_string(),
            slug: Some("one-process".to_string()),
            statement: "One process owns SQLite.".to_string(),
            why: String::new(),
            status: "active".to_string(),
            replaced_by: None,
            position: 0,
        }
    );
    assert_eq!(
        load_contexts(&db, "r-existing"),
        BTreeSet::from(["node-api".to_string()])
    );

    // 4.3. Accepted decision: title->statement, body->why with review trigger and status reason, status active
    let d_accepted = load_rule(&db, "d-accepted");
    assert_eq!(
        d_accepted,
        RuleRow {
            id: "d-accepted".to_string(),
            project_id: "p1".to_string(),
            slug: None,
            statement: "Use SQLite for storage".to_string(),
            why: "SQLite is embedded and fast.\n\nmulti-writer needed\n\nbenchmarks approved"
                .to_string(),
            status: "active".to_string(),
            replaced_by: None,
            position: 1,
        }
    );
    assert_eq!(
        load_contexts(&db, "d-accepted"),
        BTreeSet::from(["node-api".to_string()])
    );

    // 4.4. Challenged decision: status challenged -> under_review
    let d_challenged = load_rule(&db, "d-challenged");
    assert_eq!(
        d_challenged,
        RuleRow {
            id: "d-challenged".to_string(),
            project_id: "p1".to_string(),
            slug: None,
            statement: "Local HTTP for IPC".to_string(),
            why: "Use HTTP between desktop and core.\n\nIPC latency > 5ms\n\nnamed pipes proposed"
                .to_string(),
            status: "under_review".to_string(),
            replaced_by: None,
            position: 2,
        }
    );
    assert_eq!(
        load_contexts(&db, "d-challenged"),
        BTreeSet::from(["node-ui".to_string()])
    );

    // 4.5. Superseded decision: status superseded -> replaced with replaced_by pointing at d-accepted
    let d_superseded = load_rule(&db, "d-superseded");
    assert_eq!(
        d_superseded,
        RuleRow {
            id: "d-superseded".to_string(),
            project_id: "p1".to_string(),
            slug: None,
            statement: "Raw TCP sockets".to_string(),
            why: "Raw TCP connections for everything.\n\ntoo brittle\n\nHTTP adopted instead"
                .to_string(),
            status: "replaced".to_string(),
            replaced_by: Some("d-accepted".to_string()),
            position: 3,
        }
    );
    assert_eq!(
        load_contexts(&db, "d-superseded"),
        BTreeSet::from(["node-api".to_string()])
    );

    // 4.6. Principle: moved into an unscoped rule
    let principle = load_rule(&db, "principle-local-first");
    assert_eq!(
        principle,
        RuleRow {
            id: "principle-local-first".to_string(),
            project_id: "p1".to_string(),
            slug: None,
            statement: "Local first architecture".to_string(),
            why: "All data stays on the machine.\n\ncloud backup added\n\ncore philosophy"
                .to_string(),
            status: "active".to_string(),
            replaced_by: None,
            position: 4,
        }
    );
    assert!(
        load_contexts(&db, "principle-local-first").is_empty(),
        "principle must be an unscoped rule (no entity_contexts rows)"
    );

    // 4.7. Fact 1 (duplicate of r-existing): dropped completely
    let fact_dup_count = count(&db, "SELECT COUNT(*) FROM rules WHERE id = 'fact-dup'");
    assert_eq!(
        fact_dup_count, 0,
        "duplicate fact must not be migrated to rules"
    );
    assert!(
        load_contexts(&db, "fact-dup").is_empty(),
        "duplicate fact contexts must be removed from entity_contexts"
    );

    // 4.8. Fact 2 (unique): migrated into rule with slug, active status, empty why, contexts kept
    let fact_unique = load_rule(&db, "fact-unique");
    assert_eq!(
        fact_unique,
        RuleRow {
            id: "fact-unique".to_string(),
            project_id: "p1".to_string(),
            slug: Some("backend-local".to_string()),
            statement: "Backends run locally.".to_string(),
            why: String::new(),
            status: "active".to_string(),
            replaced_by: None,
            position: 5,
        }
    );
    assert_eq!(
        load_contexts(&db, "fact-unique"),
        BTreeSet::from(["node-ui".to_string()])
    );

    // 4.9. Verify cleanup: moved decisions and principles deleted from knowledge_items, facts deleted from canon_facts
    let decisions_left = count(
        &db,
        "SELECT COUNT(*) FROM knowledge_items WHERE kind = 'decision'",
    );
    assert_eq!(
        decisions_left, 0,
        "no decisions should remain in knowledge_items"
    );

    let facts_left = count(&db, "SELECT COUNT(*) FROM canon_facts");
    assert_eq!(facts_left, 0, "no facts should remain in canon_facts");

    // 4.10. Verify nothing else deleted: doc-arch is still in knowledge_items with its context
    let doc_count = count(
        &db,
        "SELECT COUNT(*) FROM knowledge_items WHERE id = 'doc-arch'",
    );
    assert_eq!(doc_count, 1, "knowledge_items docs must not be deleted");
    assert_eq!(
        load_contexts(&db, "doc-arch"),
        BTreeSet::from(["node-api".to_string()]),
        "doc contexts must remain"
    );

    // 4.11. Verify widened status CHECK constraint on rules:
    // proposed, active, under_review, replaced, removed are valid
    db.execute(
        "INSERT INTO rules(id, project_id, statement, status, position, created_at, updated_at)
         VALUES('r-prop', 'p1', 'Proposed rule', 'proposed', 10, 'now', 'now')",
        [],
    )
    .expect("insert proposed rule");

    db.execute(
        "INSERT INTO rules(id, project_id, statement, status, position, created_at, updated_at)
         VALUES('r-rem', 'p1', 'Removed rule', 'removed', 11, 'now', 'now')",
        [],
    )
    .expect("insert removed rule");

    let invalid_status_result = db.execute(
        "INSERT INTO rules(id, project_id, statement, status, position, created_at, updated_at)
         VALUES('r-invalid', 'p1', 'Invalid rule', 'unknown_status', 12, 'now', 'now')",
        [],
    );
    assert!(
        invalid_status_result.is_err(),
        "rules table CHECK constraint should reject unknown status"
    );

    // 4.12. Verify delete trigger on rules cleans up entity_contexts
    db.execute("DELETE FROM rules WHERE id = 'd-challenged'", [])
        .expect("delete rule");
    assert!(
        load_contexts(&db, "d-challenged").is_empty(),
        "entity_contexts_rule_delete trigger must clean up contexts when rule is deleted"
    );

    // 4.13. Foreign keys check:
    assert_no_foreign_key_violations(&db);
}

#[test]
fn rules_unify_migration_additional_status_and_paragraph_cases() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut db = open(tmp.path().join("edge_cases.db")).expect("open");

    migrate_before(&mut db, "0058");

    db.execute_batch(
        "INSERT INTO projects(id, name, created_at, updated_at, status, archived_at)
         VALUES('p1', 'P1', 'now', 'now', 'active', '');

         -- Decision with proposed status
         INSERT INTO knowledge_items(id, project_id, title, body, kind, status, extra_json, created_at, updated_at)
         VALUES('d-prop', 'p1', 'Proposed idea', 'A thought.', 'decision', 'proposed', '{}', 'now', 'now');

         -- Decision with unknown status -> maps to removed
         INSERT INTO knowledge_items(id, project_id, title, body, kind, status, extra_json, created_at, updated_at)
         VALUES('d-unk', 'p1', 'Retired idea', 'Old body.', 'decision', 'archived', '{}', 'now', 'now');

         -- Decision with empty body, but reviewTrigger and statusReason
         INSERT INTO knowledge_items(id, project_id, title, body, kind, status, extra_json, created_at, updated_at)
         VALUES('d-nobody', 'p1', 'No body decision', '', 'decision', 'active',
                '{\"reviewTrigger\":\"Trig\",\"statusReason\":\"Reason\"}', 'now', 'now');

         -- Decision with body only (no trigger, no reason)
         INSERT INTO knowledge_items(id, project_id, title, body, kind, status, extra_json, created_at, updated_at)
         VALUES('d-bodyonly', 'p1', 'Body only', 'Only body here.', 'decision', 'active', '{}', 'now', 'now');

         -- Active decision in the same batch that duplicates a fact statement
         INSERT INTO knowledge_items(id, project_id, title, body, kind, status, extra_json, created_at, updated_at)
         VALUES('d-active-match', 'p1', 'Strict typing everywhere.', 'Enforce types.', 'decision', 'active', '{}', 'now', 'now');

         -- Fact duplicating d-active-match case-insensitively and without trailing period
         INSERT INTO canon_facts(id, project_id, statement, truth_class, source, actor, observed_at, created_at, updated_at)
         VALUES('fact-match-dec', 'p1', 'strict typing everywhere', 'declared', 'polterware', 'agent', 'now', 'now', 'now');",
    )
    .expect("seed edge cases");

    apply_migration(&mut db, "0058_rules_unify.sql");

    let prop = load_rule(&db, "d-prop");
    assert_eq!(prop.status, "proposed");
    assert_eq!(prop.why, "A thought.");

    let unk = load_rule(&db, "d-unk");
    assert_eq!(unk.status, "removed");

    let nobody = load_rule(&db, "d-nobody");
    assert_eq!(nobody.why, "Trig\n\nReason");

    let bodyonly = load_rule(&db, "d-bodyonly");
    assert_eq!(bodyonly.why, "Only body here.");

    // fact-match-dec duplicated d-active-match, so it must be dropped
    let fact_count = count(
        &db,
        "SELECT COUNT(*) FROM rules WHERE id = 'fact-match-dec'",
    );
    assert_eq!(
        fact_count, 0,
        "fact duplicating active migrated decision must be dropped"
    );

    assert_no_foreign_key_violations(&db);
}
