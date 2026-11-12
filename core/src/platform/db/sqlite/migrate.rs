//! Schema migrations: the `migrations/*.sql` files, carried byte for byte.

use rusqlite::{Connection, params};
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use super::{Error, Result};

pub(super) const MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_first_version.sql",
        include_str!("migrations/0001_first_version.sql"),
    ),
    (
        "0002_knowledge_spec_kinds.sql",
        include_str!("migrations/0002_knowledge_spec_kinds.sql"),
    ),
    (
        "0003_canon_minimal.sql",
        include_str!("migrations/0003_canon_minimal.sql"),
    ),
    (
        "0004_knowledge_links_verdict_fts.sql",
        include_str!("migrations/0004_knowledge_links_verdict_fts.sql"),
    ),
    (
        "0005_knowledge_spec_process_kinds.sql",
        include_str!("migrations/0005_knowledge_spec_process_kinds.sql"),
    ),
    (
        "0006_agent_tool_calls.sql",
        include_str!("migrations/0006_agent_tool_calls.sql"),
    ),
    (
        "0007_node_ssh_profiles.sql",
        include_str!("migrations/0007_node_ssh_profiles.sql"),
    ),
    (
        "0008_knowledge_source_kind.sql",
        include_str!("migrations/0008_knowledge_source_kind.sql"),
    ),
    (
        "0009_canon_facts_evidence.sql",
        include_str!("migrations/0009_canon_facts_evidence.sql"),
    ),
    (
        "0010_kill_ops_terminal_nodes.sql",
        include_str!("migrations/0010_kill_ops_terminal_nodes.sql"),
    ),
    (
        "0011_canon_relationship_predicates.sql",
        include_str!("migrations/0011_canon_relationship_predicates.sql"),
    ),
    (
        "0012_agent_runs.sql",
        include_str!("migrations/0012_agent_runs.sql"),
    ),
    (
        "0013_agent_run_actor_kind.sql",
        include_str!("migrations/0013_agent_run_actor_kind.sql"),
    ),
    (
        "0014_work_model.sql",
        include_str!("migrations/0014_work_model.sql"),
    ),
    (
        "0015_work_handoffs.sql",
        include_str!("migrations/0015_work_handoffs.sql"),
    ),
    (
        "0016_project_repos.sql",
        include_str!("migrations/0016_project_repos.sql"),
    ),
    (
        "0017_work_commit_links.sql",
        include_str!("migrations/0017_work_commit_links.sql"),
    ),
    (
        "0018_work_claims.sql",
        include_str!("migrations/0018_work_claims.sql"),
    ),
    (
        "0019_inbox_dispositions.sql",
        include_str!("migrations/0019_inbox_dispositions.sql"),
    ),
    (
        "0020_atlas_nodes.sql",
        include_str!("migrations/0020_atlas_nodes.sql"),
    ),
    (
        "0021_atlas_repos.sql",
        include_str!("migrations/0021_atlas_repos.sql"),
    ),
    (
        "0022_decision_lifecycle.sql",
        include_str!("migrations/0022_decision_lifecycle.sql"),
    ),
    (
        "0023_architecture_evolution.sql",
        include_str!("migrations/0023_architecture_evolution.sql"),
    ),
    (
        "0024_agent_hook_switches.sql",
        include_str!("migrations/0024_agent_hook_switches.sql"),
    ),
    (
        "0025_atlas_uses.sql",
        include_str!("migrations/0025_atlas_uses.sql"),
    ),
    (
        "0026_knowledge_evidence_kind.sql",
        include_str!("migrations/0026_knowledge_evidence_kind.sql"),
    ),
    ("0027_scope.sql", include_str!("migrations/0027_scope.sql")),
    (
        "0028_dispatch.sql",
        include_str!("migrations/0028_dispatch.sql"),
    ),
    (
        "0029_canvas.sql",
        include_str!("migrations/0029_canvas.sql"),
    ),
    (
        "0030_boards.sql",
        include_str!("migrations/0030_boards.sql"),
    ),
    (
        "0031_deployments.sql",
        include_str!("migrations/0031_deployments.sql"),
    ),
    (
        "0032_polter_facts.sql",
        include_str!("migrations/0032_polter_facts.sql"),
    ),
    (
        "0033_drop_architecture.sql",
        include_str!("migrations/0033_drop_architecture.sql"),
    ),
    (
        "0034_canvas_single_type.sql",
        include_str!("migrations/0034_canvas_single_type.sql"),
    ),
    (
        "0035_saved_filters.sql",
        include_str!("migrations/0035_saved_filters.sql"),
    ),
    (
        "0036_archive.sql",
        include_str!("migrations/0036_archive.sql"),
    ),
    (
        "0037_structure_rename.sql",
        include_str!("migrations/0037_structure_rename.sql"),
    ),
    (
        "0038_decision_doc_kinds.sql",
        include_str!("migrations/0038_decision_doc_kinds.sql"),
    ),
    (
        "0039_structure_kind_tech.sql",
        include_str!("migrations/0039_structure_kind_tech.sql"),
    ),
    (
        "0040_milestones.sql",
        include_str!("migrations/0040_milestones.sql"),
    ),
    (
        "0041_agents_sync.sql",
        include_str!("migrations/0041_agents_sync.sql"),
    ),
    (
        "0042_hook_context_seen.sql",
        include_str!("migrations/0042_hook_context_seen.sql"),
    ),
    (
        "0043_drop_structure_uses.sql",
        include_str!("migrations/0043_drop_structure_uses.sql"),
    ),
    (
        "0044_archive_source_docs.sql",
        include_str!("migrations/0044_archive_source_docs.sql"),
    ),
    (
        "0045_docs_without_types.sql",
        include_str!("migrations/0045_docs_without_types.sql"),
    ),
    (
        "0046_entity_contexts.sql",
        include_str!("migrations/0046_entity_contexts.sql"),
    ),
    (
        "0047_rules_without_evidence.sql",
        include_str!("migrations/0047_rules_without_evidence.sql"),
    ),
    (
        "0048_card_commits.sql",
        include_str!("migrations/0048_card_commits.sql"),
    ),
    (
        "0049_conflicts.sql",
        include_str!("migrations/0049_conflicts.sql"),
    ),
    (
        "0050_repositories.sql",
        include_str!("migrations/0050_repositories.sql"),
    ),
    (
        "0051_no_system_level.sql",
        include_str!("migrations/0051_no_system_level.sql"),
    ),
    (
        "0052_domain_no_checkout.sql",
        include_str!("migrations/0052_domain_no_checkout.sql"),
    ),
    (
        "0053_deploy_targets.sql",
        include_str!("migrations/0053_deploy_targets.sql"),
    ),
    (
        "0054_dispatch_route_decisions.sql",
        include_str!("migrations/0054_dispatch_route_decisions.sql"),
    ),
    (
        "0055_dispatch_route_area.sql",
        include_str!("migrations/0055_dispatch_route_area.sql"),
    ),
    (
        "0056_dispatch_classifier.sql",
        include_str!("migrations/0056_dispatch_classifier.sql"),
    ),
    (
        "0057_drop_dispatch.sql",
        include_str!("migrations/0057_drop_dispatch.sql"),
    ),
    (
        "0058_rules_unify.sql",
        include_str!("migrations/0058_rules_unify.sql"),
    ),
    (
        "0059_card_spec.sql",
        include_str!("migrations/0059_card_spec.sql"),
    ),
    (
        "0060_card_confirmations.sql",
        include_str!("migrations/0060_card_confirmations.sql"),
    ),
];

const SCHEMA_MIGRATIONS: &str = "CREATE TABLE IF NOT EXISTS schema_migrations (\n\t\tversion INTEGER PRIMARY KEY,\n\t\tname TEXT NOT NULL,\n\t\tapplied_at TEXT NOT NULL\n\t)";

const RFC3339_SECONDS: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");

/// The tables created before the PPM to Canon rename, mapped to their current
/// names.
const LEGACY_CANON_TABLES: &[(&str, &str)] = &[
    ("ppm_entities", "canon_entities"),
    ("ppm_relationships", "canon_relationships"),
    ("ppm_facts", "canon_facts"),
    ("ppm_evidence", "canon_evidence"),
    ("ppm_fact_evidence", "canon_fact_evidence"),
];

/// Applied migration files, mapped to their names after the rename.
const LEGACY_MIGRATION_NAMES: &[(&str, &str)] = &[
    ("0003_ppm_minimal.sql", "0003_canon_minimal.sql"),
    (
        "0004_knowledge_links_adr_fts.sql",
        "0004_knowledge_links_verdict_fts.sql",
    ),
    (
        "0009_ppm_facts_evidence.sql",
        "0009_canon_facts_evidence.sql",
    ),
    (
        "0011_ppm_relationship_predicates.sql",
        "0011_canon_relationship_predicates.sql",
    ),
];

pub fn migrate(conn: &mut Connection) -> Result<()> {
    create_schema_migrations(conn)?;
    // Rebuild an incompatible legacy baseline before applying pending SQL.
    // Pending migrations may assume the current first-version tables exist.
    ensure_baseline_compatible(conn)?;
    rename_legacy_canon_tables(conn)
        .map_err(|err| Error::other(format!("rename legacy canon tables: {err}")))?;
    apply_pending_migrations(conn)
}

fn create_schema_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_MIGRATIONS)?;
    Ok(())
}

/// Carries a store from before the PPM to Canon rename forward in place. Tables
/// keep their rows, SQLite rewrites the foreign keys that point at them, and
/// their indexes are recreated under the current names. Decisions still stored
/// as kind 'adr' are converted by the pending migrations.
fn rename_legacy_canon_tables(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;
    let mut renamed = false;
    for (legacy, current) in LEGACY_CANON_TABLES {
        let legacy_count: i64 = tx.query_row(
            "SELECT COUNT(1) FROM sqlite_master WHERE type='table' AND name=?",
            [legacy],
            |row| row.get(0),
        )?;
        if legacy_count == 0 {
            continue;
        }
        let current_count: i64 = tx.query_row(
            "SELECT COUNT(1) FROM sqlite_master WHERE type='table' AND name=?",
            [current],
            |row| row.get(0),
        )?;
        if current_count > 0 {
            return Err(Error::other(format!("both {legacy} and {current} exist")));
        }
        tx.execute(&format!("ALTER TABLE {legacy} RENAME TO {current}"), [])?;
        renamed = true;
    }
    if !renamed {
        return Ok(());
    }
    let indexes: Vec<(String, String)> = {
        let mut stmt = tx.prepare(
            "SELECT name, sql FROM sqlite_master WHERE type='index' AND name LIKE 'idx_ppm_%' AND sql IS NOT NULL",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    for (name, statement) in indexes {
        tx.execute(&format!("DROP INDEX \"{name}\""), [])?;
        tx.execute_batch(&statement.replace("ppm_", "canon_"))?;
    }
    for (legacy, current) in LEGACY_MIGRATION_NAMES {
        tx.execute(
            "UPDATE schema_migrations SET name=? WHERE name=?",
            params![current, legacy],
        )?;
    }
    tx.commit()?;
    Ok(())
}

fn apply_pending_migrations(conn: &mut Connection) -> Result<()> {
    for (name, body) in MIGRATIONS {
        let version = parse_version(name)?;
        let exists: i64 = conn.query_row(
            "SELECT COUNT(1) FROM schema_migrations WHERE version=?",
            [version],
            |row| row.get(0),
        )?;
        if exists > 0 {
            continue;
        }
        let tx = conn.transaction()?;
        tx.execute_batch(body)
            .map_err(|err| Error::other(format!("apply {name}: {err}")))?;
        tx.execute(
            "INSERT INTO schema_migrations(version, name, applied_at) VALUES (?,?,?)",
            params![version, name, now_rfc3339()],
        )?;
        tx.commit()?;
    }
    Ok(())
}

/// Rebuilds the database when an older projects table is present without
/// archive columns. Polterware has a single clean baseline and does not migrate
/// legacy local databases in place.
fn ensure_baseline_compatible(conn: &mut Connection) -> Result<()> {
    if projects_baseline_compatible(conn)? {
        return Ok(());
    }
    drop_all_tables(conn)
        .map_err(|err| Error::other(format!("rebuild incompatible baseline: {err}")))?;
    create_schema_migrations(conn)?;
    apply_pending_migrations(conn)
}

pub(crate) fn projects_baseline_compatible(conn: &Connection) -> Result<bool> {
    let table_count: i64 = conn.query_row(
        "SELECT COUNT(1) FROM sqlite_master WHERE type='table' AND name='projects'",
        [],
        |row| row.get(0),
    )?;
    if table_count == 0 {
        return Ok(true);
    }
    let has_status = table_has_column(conn, "projects", "status")?;
    let has_archived_at = table_has_column(conn, "projects", "archived_at")?;
    Ok(has_status && has_archived_at)
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn drop_all_tables(conn: &mut Connection) -> Result<()> {
    let names: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    let tx = conn.transaction()?;
    tx.execute_batch("PRAGMA foreign_keys = OFF")?;
    for name in names {
        tx.execute_batch(&format!(
            "DROP TABLE IF EXISTS \"{}\"",
            name.replace('"', "\"\"")
        ))?;
    }
    tx.execute_batch("PRAGMA foreign_keys = ON")?;
    tx.commit()?;
    Ok(())
}

/// Returns the embedded migration numbers in apply order. A reserved number may
/// leave a gap until that migration file exists; the applied set must still
/// match this list.
pub fn migration_versions() -> Result<Vec<i64>> {
    let mut versions = Vec::with_capacity(MIGRATIONS.len());
    for (name, _) in MIGRATIONS {
        versions.push(parse_version(name)?);
    }
    versions.sort_unstable();
    Ok(versions)
}

fn parse_version(name: &str) -> Result<i64> {
    let Some((digits, _)) = name.split_once('_') else {
        return Err(Error::other(format!(
            "migration name {name}: unexpected EOF"
        )));
    };
    digits
        .parse::<i64>()
        .map_err(|_| Error::other(format!("migration name {name}: expected integer")))
}

pub fn applied_versions(conn: &Connection) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare("SELECT version FROM schema_migrations ORDER BY version")?;
    let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&RFC3339_SECONDS)
        .unwrap_or_default()
}
