use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, params};

use super::migrate::MIGRATIONS;
use super::*;

/// What the change recorder received: written tables and project ids.
type Recordings = Arc<Mutex<Vec<(Vec<String>, Vec<String>)>>>;

/// Serializes the tests that install the process-wide change recorder, so one
/// does not clear another's while it runs.
static RECORDER_LOCK: Mutex<()> = Mutex::new(());

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

/// Applies the embedded migrations whose names sort before `prefix`.
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

#[test]
fn open_sets_the_pragmas() {
    let conn = open(":memory:").expect("open");
    let foreign_keys: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .expect("foreign_keys");
    assert_eq!(foreign_keys, 1);
    let busy_timeout: i64 = conn
        .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
        .expect("busy_timeout");
    assert_eq!(busy_timeout, 5000);
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .expect("journal_mode");
    assert_eq!(
        journal_mode, "memory",
        "an in-memory database cannot use WAL"
    );
}

#[test]
fn open_creates_a_private_directory() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let nested = tmp.path().join("a").join("b");
    let _conn = open(nested.join("local.db")).expect("open");
    let mode = fs::metadata(&nested)
        .expect("metadata")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o700, "mode = {mode:o}");
}

#[test]
fn open_migrate_versions() {
    let mut db = open(":memory:").expect("open");
    migrate(&mut db).expect("migrate");
    let vers = applied_versions(&db).expect("applied");
    let want = migration_versions().expect("embedded");
    assert_eq!(vers, want, "applied {vers:?}, embedded {want:?}");
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/platform/db/sqlite/migrations");
    let files = fs::read_dir(&dir).expect("migrations dir");
    let on_disk = files
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "sql"))
        .count();
    assert_eq!(
        on_disk,
        want.len(),
        "disk migrations {on_disk}, embedded {}",
        want.len()
    );
    // idempotent
    migrate(&mut db).expect("migrate again");
}

#[test]
fn migrate_rebuilds_incompatible_projects_baseline() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let path = tmp.path().join("legacy.db");
    let mut db = open(&path).expect("open");

    db.execute_batch(
        r#"
		CREATE TABLE schema_migrations (
			version INTEGER PRIMARY KEY,
			name TEXT NOT NULL,
			applied_at TEXT NOT NULL
		);
		INSERT INTO schema_migrations(version, name, applied_at) VALUES (1, '0001_first_version.sql', '2026-01-01T00:00:00Z');
		CREATE TABLE projects (
			id TEXT PRIMARY KEY,
			name TEXT NOT NULL,
			root_path TEXT NOT NULL,
			created_at TEXT NOT NULL,
			updated_at TEXT NOT NULL
		);
		INSERT INTO projects(id,name,root_path,created_at,updated_at) VALUES ('p1','Legacy','/tmp','2026-01-01T00:00:00Z','2026-01-01T00:00:00Z');
	"#,
    )
    .expect("seed legacy database");

    migrate(&mut db).expect("migrate");

    assert!(
        super::migrate::projects_baseline_compatible(&db).expect("baseline compatibility"),
        "expected rebuilt projects baseline with status/archived_at"
    );

    let projects = count(&db, "SELECT COUNT(1) FROM projects");
    assert_eq!(
        projects, 0,
        "expected legacy rows wiped during rebuild, got {projects}"
    );

    db.execute(
        "INSERT INTO projects(id,name,created_at,updated_at,status,archived_at) VALUES ('p2','Ok','2026-01-01T00:00:00Z','2026-01-01T00:00:00Z','active','')",
        [],
    )
    .expect("insert into the rebuilt baseline");
}

#[test]
fn polter_facts_migration_preserves_all_audit_entities_and_node_id() {
    let mut db = open(":memory:").expect("open");
    migrate_before(&mut db, "0032");
    db.execute(
        "INSERT INTO projects(id,name,root_path,created_at,updated_at) VALUES('p','Project','/tmp','now','now')",
        [],
    )
    .expect("insert project");
    let entities = [
        "card",
        "task",
        "question",
        "invariant",
        "handoff",
        "claim",
        "node",
        "decision",
        "canvas",
        "board",
    ];
    for entity in entities {
        db.execute(
            r#"INSERT INTO change_ops(id,project_id,batch_id,entity,entity_id,op,data_json,node_id,created_at) VALUES(?,'p','batch',?,?,'create','{"saved":true}','node-kept','now')"#,
            params![entity, entity, entity],
        )
        .expect("insert change op");
    }
    apply_migration(&mut db, "0032_polter_facts.sql");
    for entity in entities {
        let (node, data): (String, String) = db
            .query_row(
                "SELECT node_id,data_json FROM change_ops WHERE id=? AND entity=?",
                params![entity, entity],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or_else(|err| panic!("{entity}: {err}"));
        assert_eq!(
            (node.as_str(), data.as_str()),
            ("node-kept", r#"{"saved":true}"#),
            "{entity}"
        );
        db.execute(
            "INSERT INTO change_ops(id,project_id,batch_id,entity,entity_id,op,created_at) VALUES(?,'p','after',?,?,'update','now')",
            params![format!("{entity}-after"), entity, entity],
        )
        .unwrap_or_else(|err| panic!("entity {entity} rejected after migration: {err}"));
    }
    db.execute(
        "INSERT INTO change_ops(id,project_id,batch_id,entity,entity_id,op,created_at) VALUES('fact','p','after','fact','fact-id','create','now')",
        [],
    )
    .expect("insert fact change op");
}

#[test]
fn structure_rename_migration_keeps_rows_and_foreign_keys() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut db = open(tmp.path().join("rename.db")).expect("open");
    migrate_before(&mut db, "0037");
    let seed = [
        "INSERT INTO projects(id,name,root_path,created_at,updated_at) VALUES('p','Project','/tmp','now','now')",
        "INSERT INTO atlas_nodes(id,project_id,parent_id,level,slug,path,title,created_at,updated_at) VALUES('sys','p',NULL,'system','app','app','App','now','now')",
        "INSERT INTO atlas_nodes(id,project_id,parent_id,level,slug,path,title,created_at,updated_at) VALUES('cmp','p','sys','component','api','app/api','API','now','now')",
        "INSERT INTO atlas_invariants(id,node_id,text,created_at,updated_at) VALUES('inv','cmp','Stays stateless','now','now')",
        "INSERT INTO scope_assignments(project_id,item_key,node_id,updated_at) VALUES('p','drift:x','cmp','now')",
    ];
    for statement in seed {
        db.execute(statement, [])
            .unwrap_or_else(|err| panic!("{statement}: {err}"));
    }
    apply_migration(&mut db, "0037_structure_rename.sql");
    for (table, want) in [
        ("structure_nodes", 2),
        ("structure_invariants", 1),
        ("scope_assignments", 1),
    ] {
        let got = count(&db, &format!("SELECT COUNT(*) FROM {table}"));
        assert_eq!(got, want, "{table}: rows={got} want {want}");
    }
    assert_no_foreign_key_violations(&db);
    let stale = count(
        &db,
        "SELECT COUNT(*) FROM sqlite_master WHERE name LIKE '%atlas%' OR sql LIKE '%atlas%'",
    );
    assert_eq!(stale, 0, "schema still names atlas: {stale}");
    let referencing = count(
        &db,
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND (sql LIKE '%REFERENCES \"structure_nodes\"%' OR sql LIKE '%REFERENCES structure_nodes%')",
    );
    assert!(
        referencing >= 6,
        "tables referencing structure_nodes: {referencing}"
    );
    assert!(
        db.execute(
            "INSERT INTO scope_assignments(project_id,item_key,node_id,updated_at) VALUES('p','drift:y','missing','now')",
            [],
        )
        .is_err(),
        "a scope assignment to an unknown node was accepted"
    );
    db.execute("DELETE FROM structure_nodes WHERE id='sys'", [])
        .expect("delete the root node");
    let left = count(
        &db,
        "SELECT (SELECT COUNT(*) FROM structure_nodes) + (SELECT COUNT(*) FROM structure_invariants) + (SELECT COUNT(*) FROM scope_assignments)",
    );
    assert_eq!(left, 0, "cascade after rename left {left} rows");
}

#[test]
fn decision_doc_kinds_migration() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut db = open(tmp.path().join("kinds.db")).expect("open");
    migrate_before(&mut db, "0038");
    let seed = [
        "INSERT INTO projects(id,name,root_path,created_at,updated_at) VALUES('p','Project','/tmp','now','now')",
        r#"INSERT INTO knowledge_items(id,project_id,title,body,kind,tags_json,extra_json,created_at,updated_at) VALUES('d1','p','Verdict 0003: Local store','b','verdict','["verdict","storage"]','{"scope":"principle","reviewTrigger":"x"}','now','now')"#,
        r#"INSERT INTO knowledge_items(id,project_id,title,body,kind,tags_json,extra_json,created_at,updated_at) VALUES('d2','p','ADR 0004: Old name','b','verdict','["adr"]','{}','now','now')"#,
        r#"INSERT INTO knowledge_items(id,project_id,title,body,kind,tags_json,created_at,updated_at) VALUES('k1','p','Guide','b','evidence','["doc","ops"]','now','now')"#,
        r#"INSERT INTO knowledge_items(id,project_id,title,body,kind,tags_json,created_at,updated_at) VALUES('k2','p','Failover','b','evidence','["runbook"]','now','now')"#,
        r#"INSERT INTO knowledge_items(id,project_id,title,body,kind,tags_json,created_at,updated_at,archived_at) VALUES('k3','p','go.mod','','evidence','["source"]','now','now','2026-01-01T00:00:00Z')"#,
        r#"INSERT INTO knowledge_items(id,project_id,title,body,kind,tags_json,created_at,updated_at) VALUES('k4','p','Untagged','b','evidence','[]','now','now')"#,
        r#"INSERT INTO knowledge_items(id,project_id,title,body,kind,tags_json,created_at,updated_at) VALUES('src-import-1','p','go.mod','','evidence','null','now','now')"#,
        "INSERT INTO canon_entities(id,project_id,kind,ref_kind,ref_id,title,observed_at,created_at,updated_at) VALUES('e','p','component','architecture_node','n','API','now','now','now')",
        "INSERT INTO knowledge_entity_links(id,project_id,knowledge_id,entity_id,predicate,observed_at,created_at) VALUES('l','p','k1','e','DOCUMENTS','now','now')",
        r#"INSERT INTO work_cards(id,project_id,slug,title,decisions_json,created_at,updated_at) VALUES('c','p','card','Card','["Verdict-3","d1","ADR 0004","Decision-0009"]','now','now')"#,
        r#"INSERT INTO structure_nodes(id,project_id,level,slug,path,title,decisions_json,created_at,updated_at) VALUES('n','p','system','app','app','App','["verdict_12"]','now','now')"#,
    ];
    for statement in seed {
        db.execute(statement, [])
            .unwrap_or_else(|err| panic!("{statement}: {err}"));
    }
    apply_migration(&mut db, "0038_decision_doc_kinds.sql");

    #[derive(Debug, PartialEq, Eq)]
    struct Row {
        kind: String,
        doc_type: String,
        title: String,
        tags: String,
        extra: String,
    }

    let want: BTreeMap<&str, Row> = BTreeMap::from([
        (
            "d1",
            Row {
                kind: "decision".to_string(),
                doc_type: String::new(),
                title: "Decision 0003: Local store".to_string(),
                tags: r#"["storage"]"#.to_string(),
                extra: r#"{"reviewTrigger":"x","level":"principle"}"#.to_string(),
            },
        ),
        (
            "d2",
            Row {
                kind: "decision".to_string(),
                doc_type: String::new(),
                title: "Decision 0004: Old name".to_string(),
                tags: "[]".to_string(),
                extra: "{}".to_string(),
            },
        ),
        (
            "k1",
            Row {
                kind: "doc".to_string(),
                doc_type: "doc".to_string(),
                title: "Guide".to_string(),
                tags: r#"["ops"]"#.to_string(),
                extra: "{}".to_string(),
            },
        ),
        (
            "k2",
            Row {
                kind: "doc".to_string(),
                doc_type: "runbook".to_string(),
                title: "Failover".to_string(),
                tags: "[]".to_string(),
                extra: "{}".to_string(),
            },
        ),
        (
            "k3",
            Row {
                kind: "doc".to_string(),
                doc_type: "source".to_string(),
                title: "go.mod".to_string(),
                tags: "[]".to_string(),
                extra: "{}".to_string(),
            },
        ),
        (
            "k4",
            Row {
                kind: "doc".to_string(),
                doc_type: "doc".to_string(),
                title: "Untagged".to_string(),
                tags: "[]".to_string(),
                extra: "{}".to_string(),
            },
        ),
        // Older imports wrote JSON null tags; the importer id marks them as
        // sources.
        (
            "src-import-1",
            Row {
                kind: "doc".to_string(),
                doc_type: "source".to_string(),
                title: "go.mod".to_string(),
                tags: "[]".to_string(),
                extra: "{}".to_string(),
            },
        ),
    ]);
    for (id, want) in &want {
        let got = db
            .query_row(
                "SELECT kind,doc_type,title,tags_json,extra_json FROM knowledge_items WHERE id=?",
                [id],
                |row| {
                    Ok(Row {
                        kind: row.get(0)?,
                        doc_type: row.get(1)?,
                        title: row.get(2)?,
                        tags: row.get(3)?,
                        extra: row.get(4)?,
                    })
                },
            )
            .unwrap_or_else(|err| panic!("{id}: {err}"));
        assert_eq!(&got, want, "{id}");
    }
    let links = count(&db, "SELECT COUNT(*) FROM knowledge_entity_links");
    assert_eq!(links, 1, "links = {links}");
    let fts = count(&db, "SELECT COUNT(*) FROM knowledge_fts");
    assert_eq!(fts, 6, "fts rows = {fts} (archived items stay out)");
    let card: String = db
        .query_row(
            "SELECT decisions_json FROM work_cards WHERE id='c'",
            [],
            |row| row.get(0),
        )
        .expect("card decisions");
    assert_eq!(
        card,
        r#"["Decision-0003","d1","Decision-0004","Decision-0009"]"#
    );
    let node: String = db
        .query_row(
            "SELECT decisions_json FROM structure_nodes WHERE id='n'",
            [],
            |row| row.get(0),
        )
        .expect("node decisions");
    assert_eq!(node, r#"["Decision-0012"]"#);
    assert!(
        db.execute(
            "INSERT INTO knowledge_items(id,project_id,title,body,kind,doc_type,created_at,updated_at) VALUES('x','p','Bad','b','decision','runbook','now','now')",
            [],
        )
        .is_err(),
        "a decision with a doc type was accepted"
    );
    assert_no_foreign_key_violations(&db);
}

#[test]
fn structure_kind_tech_migration() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut db = open(tmp.path().join("kind-tech.db")).expect("open");
    migrate_before(&mut db, "0039");
    for statement in [
        "INSERT INTO projects(id,name,root_path,created_at,updated_at) VALUES('p','Project','/tmp','now','now')",
        "INSERT INTO structure_nodes(id,project_id,level,slug,path,title,created_at,updated_at) VALUES('n1','p','component','db','db','DB','now','now')",
    ] {
        db.execute(statement, [])
            .unwrap_or_else(|err| panic!("{statement}: {err}"));
    }
    apply_migration(&mut db, "0039_structure_kind_tech.sql");
    let (kind, tech, nodes): (String, String, i64) = db
        .query_row(
            "SELECT kind, tech, (SELECT COUNT(*) FROM structure_nodes) FROM structure_nodes WHERE id='n1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("structure node");
    assert_eq!((kind.as_str(), tech.as_str(), nodes), ("", "", 1));
    db.execute(
        "UPDATE structure_nodes SET kind='database', tech='Postgres' WHERE id='n1'",
        [],
    )
    .expect("update kind and tech");
    assert_no_foreign_key_violations(&db);
}

#[test]
fn card_spec_migration_adds_the_spec_sections() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut db = open(tmp.path().join("card_spec.db")).expect("open");
    migrate_before(&mut db, "0059");
    for statement in [
        "INSERT INTO projects(id, name, created_at, updated_at, status, archived_at) VALUES('p', 'Project', 'now', 'now', 'active', '')",
        "INSERT INTO work_cards(id,project_id,slug,title,created_at,updated_at) VALUES('c1','p','signup','Signup','now','now')",
    ] {
        db.execute(statement, [])
            .unwrap_or_else(|err| panic!("{statement}: {err}"));
    }
    apply_migration(&mut db, "0059_card_spec.sql");
    let (behavior, edge_cases, plan): (String, String, String) = db
        .query_row(
            "SELECT behavior, edge_cases, plan FROM work_cards WHERE id='c1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("the spec columns");
    assert!(
        behavior.is_empty() && edge_cases.is_empty() && plan.is_empty(),
        "existing cards start with empty sections"
    );
    db.execute(
        "UPDATE work_cards SET behavior='Imports commits.', edge_cases='Empty repository.', plan='Read HEAD.' WHERE id='c1'",
        [],
    )
    .expect("write the sections");
    let behavior: String = db
        .query_row("SELECT behavior FROM work_cards WHERE id='c1'", [], |row| {
            row.get(0)
        })
        .expect("the behavior");
    assert_eq!(behavior, "Imports commits.", "the section keeps its text");
    assert_no_foreign_key_violations(&db);
}

#[test]
fn card_confirmations_migration_creates_the_table() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut db = open(tmp.path().join("confirmations.db")).expect("open");
    migrate_before(&mut db, "0060");
    for statement in [
        "INSERT INTO projects(id, name, created_at, updated_at, status, archived_at) VALUES('p', 'Project', 'now', 'now', 'active', '')",
        "INSERT INTO work_cards(id,project_id,slug,title,created_at,updated_at) VALUES('c1','p','signup','Signup','now','now')",
    ] {
        db.execute(statement, [])
            .unwrap_or_else(|err| panic!("{statement}: {err}"));
    }
    apply_migration(&mut db, "0060_card_confirmations.sql");
    db.execute(
        "INSERT INTO card_confirmations(card_id,kind,target_id,confirmed_by,confirmed_at) VALUES('c1','invariant','inv-1','codex','now')",
        [],
    )
    .expect("a confirmation");
    let kind: String = db
        .query_row(
            "SELECT kind FROM card_confirmations WHERE card_id='c1'",
            [],
            |row| row.get(0),
        )
        .expect("the confirmation");
    assert_eq!(kind, "invariant");
    assert!(
        db.execute(
            "INSERT INTO card_confirmations(card_id,kind,target_id,confirmed_by,confirmed_at) VALUES('c1','invariant','inv-2','codex','now')",
            [],
        )
        .is_ok(),
        "a card holds several confirmations"
    );
    assert!(
        db.execute(
            "INSERT INTO card_confirmations(card_id,kind,target_id,confirmed_by,confirmed_at) VALUES('c1','other','inv-1','codex','now')",
            [],
        )
        .is_err(),
        "only invariant and rule are confirmable"
    );
    assert!(
        db.execute(
            "INSERT INTO change_ops(id,project_id,batch_id,entity,entity_id,op,created_at) VALUES('o1','p','b','confirmation','inv-1','create','now')",
            [],
        )
        .is_ok(),
        "change_ops records confirmation writes"
    );
    assert_no_foreign_key_violations(&db);
}

#[test]
fn milestones_migration() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut db = open(tmp.path().join("milestones.db")).expect("open");
    migrate_before(&mut db, "0040");
    for statement in [
        "INSERT INTO projects(id,name,root_path,created_at,updated_at) VALUES('p','Project','/tmp','now','now')",
        r#"INSERT INTO work_boards(id,project_id,name,position,config_json,created_at,updated_at) VALUES('b1','p','Public Beta',0,'{"version":1}','now','now')"#,
        r#"INSERT INTO work_boards(id,project_id,name,position,config_json,created_at,updated_at) VALUES('b2','p','public beta',1,'{"version":1}','now','now')"#,
        "INSERT INTO work_cards(id,project_id,slug,title,created_at,updated_at) VALUES('c1','p','signup','Signup','now','now')",
        "INSERT INTO change_ops(id,project_id,batch_id,entity,entity_id,op,created_at) VALUES('o1','p','batch','board','b1','create','now')",
    ] {
        db.execute(statement, [])
            .unwrap_or_else(|err| panic!("{statement}: {err}"));
    }
    apply_migration(&mut db, "0040_milestones.sql");
    let got: Vec<String> = {
        let mut stmt = db
            .prepare("SELECT id, slug, state, position FROM milestones ORDER BY position")
            .expect("prepare milestones");
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .expect("query milestones");
        rows.map(|row| {
            let (id, slug, state, position) = row.expect("milestone row");
            format!("{id}={slug}/{state}/{position}")
        })
        .collect()
    };
    assert_eq!(
        got.join(" "),
        "b1=public-beta/active/0 b2=public-beta-b2/active/1",
        "milestones = {got:?}"
    );
    let entity: String = db
        .query_row("SELECT entity FROM change_ops WHERE id='o1'", [], |row| {
            row.get(0)
        })
        .expect("board history");
    assert_eq!(entity, "milestone");
    db.execute("UPDATE work_cards SET milestone_id='b1' WHERE id='c1'", [])
        .expect("assign the milestone");
    assert!(
        db.execute(
            "UPDATE work_cards SET milestone_id='missing' WHERE id='c1'",
            []
        )
        .is_err(),
        "milestone_id must reference a milestone"
    );
    db.execute("DELETE FROM milestones WHERE id='b1'", [])
        .expect("delete the milestone");
    let milestone: Option<String> = db
        .query_row(
            "SELECT milestone_id FROM work_cards WHERE id='c1'",
            [],
            |row| row.get(0),
        )
        .expect("card milestone");
    assert_eq!(
        milestone, None,
        "a deleted milestone leaves the card in the Backlog"
    );
    let tables = count(
        &db,
        "SELECT COUNT(*) FROM sqlite_master WHERE name='work_boards'",
    );
    assert_eq!(tables, 0, "work_boards should be gone");
}

#[test]
fn agents_sync_migration_keeps_settings_and_defaults_off() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut db = open(tmp.path().join("agents-sync.db")).expect("open");
    migrate_before(&mut db, "0041");
    for statement in [
        "INSERT INTO projects(id,name,root_path,created_at,updated_at) VALUES('p','Project','/tmp','now','now')",
        "INSERT INTO project_settings(project_id,description,repository) VALUES('p','Shop','api')",
    ] {
        db.execute(statement, [])
            .unwrap_or_else(|err| panic!("{statement}: {err}"));
    }
    apply_migration(&mut db, "0041_agents_sync.sql");
    let (description, repository, sync): (String, String, i64) = db
        .query_row(
            "SELECT description, repository, agents_sync FROM project_settings WHERE project_id='p'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("project settings");
    assert_eq!(
        (description.as_str(), repository.as_str(), sync),
        ("Shop", "api", 0)
    );
}

#[test]
fn hook_context_seen_migration_cascades_with_the_project() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut db = open(tmp.path().join("hook-context.db")).expect("open");
    migrate_before(&mut db, "0042");
    apply_migration(&mut db, "0042_hook_context_seen.sql");
    for statement in [
        "INSERT INTO projects(id,name,root_path,created_at,updated_at) VALUES('p','Project','/tmp','now','now')",
        "INSERT INTO hook_context_seen(session_id,project_id,node_id,brief_hash,seen_at) VALUES('s','p','n','h','now')",
        "DELETE FROM projects WHERE id='p'",
    ] {
        db.execute(statement, [])
            .unwrap_or_else(|err| panic!("{statement}: {err}"));
    }
    let rows = count(&db, "SELECT COUNT(*) FROM hook_context_seen");
    assert_eq!(rows, 0, "rows left after the project went: {rows}");
}

#[test]
fn entity_contexts_migration() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut db = open(tmp.path().join("contexts.db")).expect("open");
    migrate_before(&mut db, "0046");
    let seed = [
        "INSERT INTO projects(id,name,root_path,created_at,updated_at) VALUES('p','Project','/tmp','now','now')",
        "INSERT INTO structure_nodes(id,project_id,level,slug,path,title,created_at,updated_at) VALUES('app','p','system','app','app','App','now','now')",
        "INSERT INTO structure_nodes(id,project_id,parent_id,level,slug,path,title,created_at,updated_at) VALUES('api','p','app','component','api','app/api','API','now','now')",
        // A principle loses its scope; a plain scoped decision keeps it as a
        // context.
        r#"INSERT INTO knowledge_items(id,project_id,title,body,kind,extra_json,created_at,updated_at,scope_node_id) VALUES('principle','p','Local first','b','decision','{"level":"principle","reviewTrigger":"x"}','now','now','app')"#,
        r#"INSERT INTO knowledge_items(id,project_id,title,body,kind,extra_json,created_at,updated_at,scope_node_id) VALUES('scoped','p','Cache','b','decision','{}','now','now','api')"#,
        r#"INSERT INTO knowledge_items(id,project_id,title,body,kind,extra_json,created_at,updated_at) VALUES('unscoped','p','Ship weekly','b','decision','{"level":"decision"}','now','now')"#,
        r#"INSERT INTO knowledge_items(id,project_id,title,body,kind,doc_type,created_at,updated_at,scope_node_id) VALUES('doc','p','Guide','b','doc','doc','now','now','app')"#,
        "INSERT INTO canon_facts(id,project_id,statement,observed_at,created_at,updated_at,scope_node_id) VALUES('fact','p','One core','now','now','now','api')",
        "INSERT INTO canon_evidence(id,project_id,kind,summary,observed_at,created_at,updated_at,scope_node_id) VALUES('ev','p','file','go.mod','now','now','now','app')",
        "INSERT INTO scope_assignments(project_id,item_key,node_id,updated_at) VALUES('p','drift:x','api','now')",
    ];
    for statement in seed {
        db.execute(statement, [])
            .unwrap_or_else(|err| panic!("{statement}: {err}"));
    }
    apply_migration(&mut db, "0046_entity_contexts.sql");

    let mut contexts = BTreeMap::new();
    {
        let mut stmt = db
            .prepare("SELECT entity_id,node_id FROM entity_contexts WHERE project_id='p'")
            .expect("prepare contexts");
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .expect("query contexts");
        for row in rows {
            let (entity, node) = row.expect("context row");
            contexts.insert(entity, node);
        }
    }
    let want: BTreeMap<&str, &str> = BTreeMap::from([
        ("scoped", "api"),
        ("doc", "app"),
        ("fact", "api"),
        ("ev", "app"),
        ("drift:x", "api"),
    ]);
    let want: BTreeMap<String, String> = want
        .into_iter()
        .map(|(entity, node)| (entity.to_string(), node.to_string()))
        .collect();
    assert_eq!(contexts, want);
    let levels = count(
        &db,
        "SELECT COUNT(*) FROM knowledge_items WHERE json_extract(extra_json,'$.level') IS NOT NULL",
    );
    assert_eq!(levels, 0, "decisions keep a level: {levels}");
    let trigger: String = db
        .query_row(
            "SELECT json_extract(extra_json,'$.reviewTrigger') FROM knowledge_items WHERE id='principle'",
            [],
            |row| row.get(0),
        )
        .expect("review trigger");
    assert_eq!(trigger, "x", "other lifecycle data was lost");
    let scoped = count(
        &db,
        "SELECT (SELECT COUNT(*) FROM knowledge_items WHERE scope_node_id IS NOT NULL) + (SELECT COUNT(*) FROM canon_facts WHERE scope_node_id IS NOT NULL) + (SELECT COUNT(*) FROM canon_evidence WHERE scope_node_id IS NOT NULL)",
    );
    assert_eq!(scoped, 0, "retired columns still hold {scoped} scopes");
    let tables = count(
        &db,
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='scope_assignments'",
    );
    assert_eq!(tables, 0, "scope_assignments is still there");
    // Contexts go with their entity and with their node.
    db.execute("DELETE FROM knowledge_items WHERE id='doc'", [])
        .expect("delete the doc");
    db.execute("DELETE FROM structure_nodes WHERE id='api'", [])
        .expect("delete the node");
    let left = count(&db, "SELECT COUNT(*) FROM entity_contexts");
    assert_eq!(left, 1, "contexts left = {left}, want only the evidence's");
}

#[test]
fn pending_tx_records_committed_tables_and_projects() {
    let _guard = RECORDER_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut db = open(":memory:").expect("open");
    migrate(&mut db).expect("migrate");
    load_project_columns(&db).expect("load project columns");
    assert_eq!(project_column_index("projects"), Some(0));
    assert_eq!(project_column_index("work_cards"), Some(1));
    assert_eq!(project_column_index("missing_table"), None);
    assert_eq!(
        row_project_id("work_cards", |index| {
            assert_eq!(index, 1);
            Some("proj-row".to_string())
        }),
        "proj-row"
    );
    assert_eq!(
        row_project_id("missing_table", |_| Some("x".to_string())),
        ""
    );

    let seen: Recordings = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    set_change_recorder(Some(Arc::new(move |tables, projects| {
        sink.lock()
            .expect("recorder sink")
            .push((tables.to_vec(), projects.to_vec()));
    })));

    let mut tx = PendingTx::new();
    tx.pre_update("main", "work_cards", Some("proj-a"));
    tx.pre_update("main", "work_tasks", Some("proj-a"));
    tx.pre_update("main", "work_tasks", None);
    tx.pre_update("main", "work_tasks", Some(""));
    tx.pre_update("main", "schema_migrations", Some("ignored"));
    tx.pre_update("temp", "work_cards", Some("ignored"));
    assert!(tx.commit());
    assert!(!tx.commit(), "a commit without writes calls nothing");

    let mut rolled_back = PendingTx::new();
    rolled_back.pre_update("main", "projects", Some("proj-z"));
    rolled_back.reset();
    assert!(!rolled_back.commit());

    set_change_recorder(None);
    let seen = seen.lock().expect("recorder sink");
    let recorded = (
        vec!["work_cards".to_string(), "work_tasks".to_string()],
        vec!["proj-a".to_string()],
    );
    assert!(seen.contains(&recorded), "recording missing: {seen:?}");
}

#[test]
fn pending_tx_records_nothing_without_a_recorder() {
    let _guard = RECORDER_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut db = open(":memory:").expect("open");
    migrate(&mut db).expect("migrate");
    set_change_recorder(None);
    let mut tx = PendingTx::new();
    tx.pre_update("main", "work_cards", Some("proj-a"));
    assert!(!tx.commit());
}

#[test]
fn open_registers_the_change_hooks() {
    let _guard = RECORDER_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut db = open(":memory:").expect("open");
    migrate(&mut db).expect("migrate");
    load_project_columns(&db).expect("load project columns");
    db.execute_batch(
        "INSERT INTO projects(id,name,created_at,updated_at,status,archived_at)
            VALUES('proj-hook','Hook','2026-01-01T00:00:00Z','2026-01-01T00:00:00Z','active','');
         INSERT INTO projects(id,name,created_at,updated_at,status,archived_at)
            VALUES('proj-rollback','Rollback','2026-01-01T00:00:00Z','2026-01-01T00:00:00Z','active','');",
    )
    .expect("projects");

    let seen: Recordings = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    set_change_recorder(Some(Arc::new(move |tables, projects| {
        sink.lock()
            .expect("recorder sink")
            .push((tables.to_vec(), projects.to_vec()));
    })));

    db.execute(
        "INSERT INTO knowledge_items(id,project_id,title,body,kind,doc_type,created_at,updated_at)
            VALUES('row','proj-hook','Title','Body','doc','doc','2026-01-01T00:00:00Z','2026-01-01T00:00:00Z')",
        [],
    )
    .expect("insert");
    // A rolled-back transaction records nothing.
    {
        let tx = db.unchecked_transaction().expect("begin");
        tx.execute(
            "INSERT INTO knowledge_items(id,project_id,title,body,kind,doc_type,created_at,updated_at)
                VALUES('gone','proj-rollback','Title','Body','doc','doc','2026-01-01T00:00:00Z','2026-01-01T00:00:00Z')",
            [],
        )
        .expect("insert");
    }
    set_change_recorder(None);

    let seen = seen.lock().expect("recorder sink");
    assert!(
        seen.iter().any(
            |(tables, projects)| tables.iter().any(|table| table == "knowledge_items")
                && projects.iter().any(|project| project == "proj-hook")
        ),
        "the commit hook did not record the write: {seen:?}"
    );
    assert!(
        !seen
            .iter()
            .any(|(_, projects)| projects.iter().any(|project| project == "proj-rollback")),
        "a rolled back write was recorded: {seen:?}"
    );
}
