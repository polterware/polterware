#[allow(unused_imports)] // The package-wide glob; every name this file uses is explicit.
use super::*;

use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::domains::polter;
use crate::domains::work;
use crate::providers::sqlite as storepkg;

use super::commits::card_commits;

/// Go's `2006-01-02 15:04` layout.
const GO_DATETIME: &[FormatItem<'static>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]");

/// Go's `15:04` layout.
const GO_CLOCK: &[FormatItem<'static>] = format_description!("[hour]:[minute]");

/// Renders one record by id as text: a card (by slug), a Rule (by slug, id, or
/// migrated Decision number), a doc with its body, a Canon entity, or an
/// AgentRun with its steps.
/// Called by the work.go query router, ported separately; unexported in Go, so
/// public here only for the package's glob re-export.
pub fn query_show(
    store: &storepkg::Store,
    snapshot: &work::Snapshot,
    id: &str,
) -> anyhow::Result<String> {
    let id = id.trim();
    if let Some(card) = snapshot.card(id) {
        return Ok(polter::encode_card(card));
    }
    if let Some(rule) = snapshot.rules.iter().find(|rule| {
        rule.id == id || polter::cites_decision(id, &rule.id, &rule.slug, &rule.statement)
    }) {
        return Ok(polter::encode_rule(
            rule,
            &snapshot.rules,
            &polter::nodes_citing_rule(&snapshot.nodes, rule),
        ));
    }
    if let Ok(item) = store.get_knowledge(id)
        && item.project_id == snapshot.project_id
        && !item.archived()
    {
        let scopes = store.scope_map(&snapshot.project_id, storepkg::SCOPE_KNOWLEDGE)?;
        let item_scopes = scopes.get(&item.id).cloned().unwrap_or_default();
        let mut text = format!("doc {} {}\n", item.id, polter::go_quote(&item.title));
        let mut paths: Vec<String> = snapshot
            .nodes
            .iter()
            .filter(|node| item_scopes.contains(&node.id))
            .map(|node| node.path.clone())
            .collect();
        if !paths.is_empty() {
            paths.sort();
            text.push_str("scope: ");
            text.push_str(&paths.join(", "));
            text.push('\n');
        }
        if !item.tags.is_empty() {
            text.push_str(&format!("tags: {}\n", item.tags.join(", ")));
        }
        return Ok(format!("{text}\n{}\n", item.body.trim_end_matches('\n')));
    }
    if let Ok(entity) = store.get_canon_entity(id)
        && entity.project_id == snapshot.project_id
    {
        return Ok(format!(
            "entity {} [{}] {}\nref: {}/{} · truth: {} · source: {}\n",
            entity.id,
            entity.kind,
            entity.title,
            entity.ref_kind,
            entity.ref_id,
            entity.truth_class,
            entity.source
        ));
    }
    if let Ok(record) = store.get_agent_run_record(id)
        && record.run.project_id == snapshot.project_id
    {
        let run = &record.run;
        let mut out = format!(
            "run {} [{}] {}\nagent: {} · actor: {} · started {}\n",
            run.id,
            run.status,
            run.task,
            run.agent_label,
            run.actor_kind,
            format_time(run.created_at, GO_DATETIME)
        );
        for step in &record.steps {
            out.push_str(&format!(
                "- {} {}: {}\n",
                format_time(step.created_at, GO_CLOCK),
                step.kind,
                step.summary
            ));
        }
        return Ok(out);
    }
    Err(anyhow::anyhow!(
        "unknown id {}: show takes a card slug, a Rule ref, a doc, entity, or run id (search <terms> finds them)",
        polter::go_quote(id)
    ))
}

/// Lists a card's commits, one line each: task, short sha, subject, and the
/// GitHub URL when there is one.
/// Called by the work.go query router, ported separately; unexported in Go, so
/// public here only for the package's glob re-export.
pub fn query_commits(
    store: &storepkg::Store,
    project_id: &str,
    slug: &str,
) -> anyhow::Result<String> {
    let commits = card_commits(store, project_id, slug)?;
    if commits.is_empty() {
        return Ok(format!(
            "no commits for {slug}: cite `{slug}#T2` in a commit subject, or run polterware task done {slug}#T2 --commit <sha>\n"
        ));
    }
    let mut out = format!("{} commits for {}\n", commits.len(), slug);
    for commit in &commits {
        let mut line = format!(
            "- T{} {} {}",
            commit.task_number,
            commit.short_sha(),
            commit.subject
        );
        if !commit.url.is_empty() {
            line.push(' ');
            line.push_str(&commit.url);
        }
        out.push_str(&line);
        out.push('\n');
    }
    Ok(out)
}

/// Formats a stored time the way Go's layout does. A format never fails there,
/// so a failure here answers with an empty string.
fn format_time(value: OffsetDateTime, format: &[FormatItem<'static>]) -> String {
    value.format(format).unwrap_or_default()
}
