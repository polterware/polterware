//! The `doc` command: the project's Knowledge docs.

use super::{
    canvas::read_source,
    core::{dial, resolve_project},
    flags::{FlagError, FlagSet},
    milestone::given,
    output::{CliError, CliResult},
};
use crate::domains::knowledge;
use crate::local_v1;
use crate::platform::coreclient::{CALL_TIMEOUT, Clients};

const USAGE: &str = "usage: polterware doc ls [--archived] [--scope <node>[,<node>]] [--project <id>]\n       polterware doc show <id|title> [--project <id>]\n       polterware doc new \"<title>\" --body <file|-> [--summary \"...\"] [--tag <tag>]... [--scope <node>[,<node>]] [--project <id>]\n       polterware doc save <id|title> [--title \"...\"] [--body <file|->] [--summary \"...\"] [--tag <tag>]... [--scope <node>[,<node>]] [--project <id>]\n       polterware doc archive|restore|rm <id|title> [--project <id>]   (rm only an archived doc)\n       (the body is Markdown; --scope names the Structure nodes the doc is about)";
const KNOWLEDGE: &str = "/polterware.local.v1.KnowledgeService/";
const WORK: &str = "/polterware.local.v1.WorkService/";

fn parse_err(err: FlagError) -> CliError {
    CliError::message(err.message())
}

fn flags(name: &'static str) -> FlagSet {
    let mut f = FlagSet::new(name);
    f.string_flag(
        "project",
        "",
        "project id (default: project containing the cwd)",
    );
    f
}

fn field_flags(f: &mut FlagSet) {
    f.string_flag("body", "", "file holding the Markdown body, or - for stdin");
    f.string_flag("summary", "", "one line on what the doc says");
    f.list_flag("tag", "a tag; repeat or separate with commas", true);
    f.list_flag(
        "scope",
        "Structure node path, slug, or id; repeat or separate with commas (default: whole project)",
        true,
    );
}

fn change(summary: String) -> Option<local_v1::ChangeEnvelope> {
    Some(local_v1::ChangeEnvelope {
        summary,
        rationale: "Written from the CLI".to_string(),
        ..Default::default()
    })
}

pub fn run_doc(db: &str, args: &[String]) -> CliResult {
    let Some(sub) = args.first() else {
        return Err(CliError::message(USAGE));
    };
    let rest = &args[1..];
    match sub.as_str() {
        "ls" | "list" => ls(db, rest),
        "show" => show(db, rest),
        "new" => new(db, rest),
        "save" => save(db, rest),
        "archive" => archive(db, rest, true),
        "restore" => archive(db, rest, false),
        "rm" => rm(db, rest),
        _ => Err(CliError::message(USAGE)),
    }
}

fn list(
    c: &Clients,
    project: &str,
    archived: bool,
    scope: &[String],
) -> Result<Vec<local_v1::KnowledgeItem>, CliError> {
    let r: local_v1::ListKnowledgeResponse = c.unary(
        &format!("{KNOWLEDGE}ListKnowledge"),
        &local_v1::ListKnowledgeRequest {
            project_id: project.to_string(),
            scope_nodes: scope.to_vec(),
            archived,
            ..Default::default()
        },
        CALL_TIMEOUT,
    )?;
    Ok(r.items)
}

/// Finds a doc by id, then by its exact title; an ambiguous title is refused.
fn find(
    items: &[local_v1::KnowledgeItem],
    reference: &str,
) -> Result<local_v1::KnowledgeItem, CliError> {
    let needle = reference.trim();
    if let Some(item) = items.iter().find(|i| i.id == needle) {
        return Ok(item.clone());
    }
    let named: Vec<&local_v1::KnowledgeItem> = items.iter().filter(|i| i.title == needle).collect();
    match named.as_slice() {
        [one] => Ok((*one).clone()),
        [] => Err(CliError::message(format!(
            "unknown doc {needle:?}: list them with polterware doc ls"
        ))),
        _ => Err(CliError::message(format!(
            "{} docs are titled {needle:?}: name one by id",
            named.len()
        ))),
    }
}

/// Finds a doc among the active ones, then the archived ones.
fn find_any(
    c: &Clients,
    project: &str,
    reference: &str,
) -> Result<local_v1::KnowledgeItem, CliError> {
    match find(&list(c, project, false, &[])?, reference) {
        Ok(item) => Ok(item),
        Err(first) => find(&list(c, project, true, &[])?, reference).map_err(|_| first),
    }
}

fn node_paths(c: &Clients, project: &str, ids: &[String]) -> Result<Vec<String>, CliError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let r: local_v1::ListStructureResponse = c.unary(
        &format!("{WORK}ListStructure"),
        &local_v1::ListStructureRequest {
            project_id: project.to_string(),
            ..Default::default()
        },
        CALL_TIMEOUT,
    )?;
    Ok(ids
        .iter()
        .map(|id| {
            r.nodes
                .iter()
                .find(|n| &n.id == id)
                .map_or_else(|| id.clone(), |n| n.path.clone())
        })
        .collect())
}

fn format_list(items: &[local_v1::KnowledgeItem]) -> String {
    if items.is_empty() {
        return "no docs\n".to_string();
    }
    let mut out = String::new();
    for item in items {
        out.push_str(&format!("{}  {:?}", item.id, item.title));
        if !item.tags.is_empty() {
            out.push_str(&format!("  [{}]", item.tags.join(", ")));
        }
        out.push('\n');
        if !item.summary.is_empty() {
            out.push_str(&format!("  {}\n", item.summary));
        }
    }
    out
}

fn format_doc(item: &local_v1::KnowledgeItem, scope: &[String]) -> String {
    let mut out = format!("# {}\n", item.title);
    let mut meta = vec![format!("id: {}", item.id)];
    if !item.tags.is_empty() {
        meta.push(format!("tags: {}", item.tags.join(", ")));
    }
    if scope.is_empty() {
        meta.push("scope: whole project".to_string());
    } else {
        meta.push(format!("scope: {}", scope.join(", ")));
    }
    if !item.archived_at.is_empty() {
        meta.push("archived".to_string());
    }
    out.push_str(&meta.join(" · "));
    out.push('\n');
    if !item.summary.is_empty() {
        out.push_str(&format!("{}\n", item.summary));
    }
    if !item.body.trim().is_empty() {
        out.push('\n');
        out.push_str(item.body.trim_end());
        out.push('\n');
    }
    out
}

fn ls(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("doc ls");
    f.bool_flag("archived", false, "list only archived docs");
    f.list_flag("scope", "Structure nodes", true);
    f.parse_interspersed(args).map_err(parse_err)?;
    if !f.rest().is_empty() {
        return Err(CliError::message(USAGE));
    }
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let items = list(&c, &p.id, f.bool_value("archived"), f.list_value("scope"))?;
    print!("{}", format_list(&items));
    Ok(())
}

fn show(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("doc show");
    f.parse_interspersed(args).map_err(parse_err)?;
    if f.rest().len() != 1 {
        return Err(CliError::message(USAGE));
    }
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let item = find_any(&c, &p.id, &f.rest()[0])?;
    let scope = node_paths(&c, &p.id, &item.scope_node_ids)?;
    print!("{}", format_doc(&item, &scope));
    Ok(())
}

/// Saves the item, then sets its scope when `--scope` was given.
fn write(
    c: &Clients,
    project: &str,
    item: local_v1::KnowledgeItem,
    summary: String,
    scope: Option<&[String]>,
) -> Result<local_v1::KnowledgeItem, CliError> {
    let r: local_v1::SaveKnowledgeResponse = c.unary(
        &format!("{KNOWLEDGE}SaveKnowledge"),
        &local_v1::SaveKnowledgeRequest {
            item: Some(item),
            change: change(summary),
        },
        CALL_TIMEOUT,
    )?;
    let saved = r
        .item
        .ok_or_else(|| CliError::message("the core returned no doc"))?;
    if let Some(nodes) = scope {
        let _: local_v1::SetScopeResponse = c.unary(
            &format!("{WORK}SetScope"),
            &local_v1::SetScopeRequest {
                project_id: project.to_string(),
                kind: "knowledge".to_string(),
                id: saved.id.clone(),
                nodes: nodes.to_vec(),
            },
            CALL_TIMEOUT,
        )?;
    }
    Ok(saved)
}

fn new(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("doc new");
    field_flags(&mut f);
    f.parse_interspersed(args).map_err(parse_err)?;
    if f.rest().len() != 1 || f.rest()[0].trim().is_empty() || f.string_value("body").is_empty() {
        return Err(CliError::message(USAGE));
    }
    let title = f.rest()[0].trim().to_string();
    let body = read_source(f.string_value("body"))?;
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let item = local_v1::KnowledgeItem {
        id: knowledge::new_item_id(),
        project_id: p.id.clone(),
        title: title.clone(),
        body,
        kind: knowledge::KIND_DOC.to_string(),
        doc_type: knowledge::KIND_DOC.to_string(),
        summary: f.string_value("summary").to_string(),
        tags: f.list_value("tag").to_vec(),
        extra_json: "{}".to_string(),
        ..Default::default()
    };
    let scope = given(args, "scope").then(|| f.list_value("scope"));
    let saved = write(&c, &p.id, item, format!("Add doc: {title}"), scope)?;
    println!("+ doc {}: {}", saved.id, saved.title);
    Ok(())
}

fn save(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("doc save");
    field_flags(&mut f);
    f.string_flag("title", "", "the doc's title");
    f.parse_interspersed(args).map_err(parse_err)?;
    if f.rest().len() != 1 {
        return Err(CliError::message(USAGE));
    }
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let mut item = find(&list(&c, &p.id, false, &[])?, &f.rest()[0])?;
    // SaveKnowledge replaces the item; keep what the flags leave out.
    if given(args, "title") {
        item.title = f.string_value("title").trim().to_string();
    }
    if given(args, "body") {
        item.body = read_source(f.string_value("body"))?;
    }
    if given(args, "summary") {
        item.summary = f.string_value("summary").to_string();
    }
    if given(args, "tag") {
        item.tags = f.list_value("tag").to_vec();
    }
    let scope = given(args, "scope").then(|| f.list_value("scope"));
    let title = item.title.clone();
    let saved = write(&c, &p.id, item, format!("Edit doc: {title}"), scope)?;
    println!("~ doc {}: {}", saved.id, saved.title);
    Ok(())
}

fn archive(db: &str, args: &[String], archived: bool) -> CliResult {
    let mut f = flags(if archived {
        "doc archive"
    } else {
        "doc restore"
    });
    f.parse_interspersed(args).map_err(parse_err)?;
    if f.rest().len() != 1 {
        return Err(CliError::message(USAGE));
    }
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let item = find(&list(&c, &p.id, !archived, &[])?, &f.rest()[0])?;
    let verb = if archived { "Archive" } else { "Restore" };
    let _: local_v1::SetKnowledgeArchivedResponse = c.unary(
        &format!("{KNOWLEDGE}SetKnowledgeArchived"),
        &local_v1::SetKnowledgeArchivedRequest {
            id: item.id.clone(),
            archived,
            change: change(format!("{verb} doc: {}", item.title)),
        },
        CALL_TIMEOUT,
    )?;
    let done = if archived { "archived" } else { "restored" };
    println!("{done} doc {}: {}", item.id, item.title);
    Ok(())
}

fn rm(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("doc rm");
    f.parse_interspersed(args).map_err(parse_err)?;
    if f.rest().len() != 1 {
        return Err(CliError::message(USAGE));
    }
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let item = find(&list(&c, &p.id, true, &[])?, &f.rest()[0]).map_err(|_| {
        CliError::message(format!(
            "no archived doc {:?}: archive it first with polterware doc archive",
            f.rest()[0]
        ))
    })?;
    let _: local_v1::DeleteKnowledgeResponse = c.unary(
        &format!("{KNOWLEDGE}DeleteKnowledge"),
        &local_v1::DeleteKnowledgeRequest {
            id: item.id.clone(),
            change: change(format!("Delete doc: {}", item.title)),
        },
        CALL_TIMEOUT,
    )?;
    println!("- doc {}: {}", item.id, item.title);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(id: &str, title: &str) -> local_v1::KnowledgeItem {
        local_v1::KnowledgeItem {
            id: id.into(),
            title: title.into(),
            ..Default::default()
        }
    }

    #[test]
    fn find_by_id_or_title() {
        let items = vec![
            doc("k1", "Local core"),
            doc("k2", "Twin"),
            doc("k3", "Twin"),
        ];
        assert_eq!(find(&items, "k1").unwrap().title, "Local core");
        assert_eq!(find(&items, "Local core").unwrap().id, "k1");
        assert!(find(&items, "Twin").is_err());
        assert!(find(&items, "nope").is_err());
    }

    #[test]
    fn show_prints_meta_and_body() {
        let mut item = doc("k1", "Local core");
        item.tags = vec!["core".into()];
        item.summary = "How the core runs.".into();
        item.body = "One core per database.\n\n".into();
        assert_eq!(
            format_doc(&item, &["core/server".into()]),
            "# Local core\nid: k1 · tags: core · scope: core/server\nHow the core runs.\n\nOne core per database.\n"
        );
        assert_eq!(
            format_doc(&doc("k2", "Empty"), &[]),
            "# Empty\nid: k2 · scope: whole project\n"
        );
    }

    #[test]
    fn list_shows_tags_and_summary() {
        let mut item = doc("k1", "Local core");
        item.tags = vec!["core".into(), "ops".into()];
        item.summary = "How it runs.".into();
        assert_eq!(
            format_list(&[item]),
            "k1  \"Local core\"  [core, ops]\n  How it runs.\n"
        );
        assert_eq!(format_list(&[]), "no docs\n");
    }
}
