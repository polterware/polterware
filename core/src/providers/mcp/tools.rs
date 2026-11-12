//! The four tools, their schemas, and
//! the map from a retired name to its replacement.

use serde_json::{Value, json};

use crate::domains::polter;

use super::{Error, Result};

// The MCP surface is four tools: reads and Polter writes, the format, and
// docs, whose Markdown bodies do not fit Polter text.
pub const TOOL_POLTER_QUERY: &str = "polter.query";
pub const TOOL_POLTER_APPLY: &str = "polter.apply";
pub const TOOL_POLTER_FORMAT: &str = "polter.format";
pub const TOOL_DOC_UPSERT: &str = "doc.upsert";

/// Maps each removed tool to what replaces it, so an agent with a stale tool
/// list gets a pointer instead of "unknown tool".
pub(crate) fn retired_tools() -> &'static [(&'static str, &'static str)] {
    &[
        (
            "context.get",
            r#"polter.query "brief <slug|path|file>" or "search <terms>""#,
        ),
        ("entity.get", r#"polter.query "show <id>""#),
        ("evidence.get", "nothing: evidence is retired"),
        ("drift.check", r#"polter.query "commits <slug>""#),
        (
            "agent_run.start",
            "nothing: polter.apply and doc.upsert join this session's run on their own",
        ),
        (
            "agent_run.append",
            "nothing: polter.apply and doc.upsert join this session's run on their own",
        ),
        (
            "agent_run.complete",
            "nothing: the session run completes when the session closes",
        ),
        ("agent_run.get", r#"polter.query "show <run id>""#),
        ("agent_run.list", "the Activity page in the Desktop"),
        (
            "fact.upsert",
            "polter.apply with a #### rule <slug>: Statement block (polter.format topic rules)",
        ),
        (
            "evidence.upsert",
            "nothing: evidence is retired; a Rule is a statement the code must keep",
        ),
        (
            "evidence.link",
            "nothing: evidence is retired; a Rule is a statement the code must keep",
        ),
        (
            "knowledge.upsert",
            "doc.upsert for docs, or polter.apply with a #### rule block",
        ),
        ("knowledge.link", "doc.upsert with scope: <Structure path>"),
    ]
}

/// The replacement a retired tool name answers with.
fn retired_replacement(name: &str) -> Option<&'static str> {
    retired_tools()
        .iter()
        .find(|(retired, _)| *retired == name)
        .map(|(_, replacement)| *replacement)
}

fn scope_argument_schema() -> Value {
    json!({
        "description": "Structure node path, slug, or id, or a list of them. The read keeps the union of each node, its descendants, and its ancestors. Omit for the whole project.",
        "oneOf": [
            {"type": "string"},
            {"type": "array", "items": {"type": "string"}},
        ],
    })
}

/// The tools `tools/list` answers with, in the order Go declares them.
pub(crate) fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": TOOL_POLTER_QUERY,
            "description": "Read project state as compact text. Read \"brief <slug|path|file>\" before working on a card, node, or file; \"search <terms>\" finds anything. The query field lists the other reads.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_id": {"type": "string"},
                    "query": {"type": "string", "description": "\"brief <slug|path|file> [budget:N]\", \"search <terms>\", \"show <id>\", \"commits <slug>\", \"conflict <slug>\", \"conflicts\", \"cards\" (default), \"card <slug>\", \"status\", \"inbox\", \"handoff <slug>\", \"structure [<path>]\", \"rules [status]\", \"feature <slug> [shape|contract|plan]\", \"verify <slug|slug#T2>\", \"milestones\", \"roadmap\", or \"canvas [<name|id>]\""},
                    "scope": scope_argument_schema(),
                    "scope_exact": {"type": "boolean", "description": "With scope, leave out each node's ancestors."},
                },
                "required": ["project_id"],
            },
        }),
        json!({
            "name": TOOL_POLTER_APPLY,
            "description": "Write Work cards, Structure nodes, and Rules from Polter text, or a canvas from its Mermaid source (canvas names it; read polter.format topic canvas once), in one audited write that joins this session's run. Needs summary and rationale (frontmatter or arguments). dry_run returns the plan.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_id": {"type": "string"},
                    "text": {"type": "string", "description": "Polter format text, or the canvas Mermaid source when canvas is set"},
                    "canvas": {"type": "string", "description": "Canvas name or id: text is then its Mermaid source, and an unknown name creates the canvas. A parse failure answers with its line and column."},
                    "dry_run": {"type": "boolean"},
                    "summary": {"type": "string", "description": "Overrides the frontmatter summary"},
                    "rationale": {"type": "string", "description": "Overrides the frontmatter rationale"},
                    "agent_run_id": {"type": "string", "description": "Optional open AgentRun"},
                },
                "required": ["project_id", "text"],
            },
        }),
        json!({
            "name": TOOL_POLTER_FORMAT,
            "description": format!(
                "Polter format core (about {} tokens); read once before polter.apply. topic adds one part.",
                polter::SPEC_CORE_TOKENS
            ),
            "inputSchema": {"type": "object", "properties": {
                "topic": {"type": "string", "enum": polter::SPEC_TOPIC_NAMES},
            }},
        }),
        json!({
            "name": TOOL_DOC_UPSERT,
            "description": "Create or update a doc (Markdown body) in one audited write that joins this session's run. Pass id to update; omitted fields stay. scope lists the Structure paths the doc is about, comma separated (empty clears).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_id": {"type": "string"},
                    "id": {"type": "string"},
                    "title": {"type": "string"},
                    "body": {"type": "string"},
                    "tags": {"type": "array", "items": {"type": "string"}},
                    "scope": {"type": "string"},
                    "summary": {"type": "string"},
                    "rationale": {"type": "string"},
                },
                "required": ["project_id", "summary", "rationale"],
            },
        }),
    ]
}

impl super::Server {
    /// Answers one tool call.
    pub(crate) fn invoke_tool(
        &mut self,
        store: &crate::providers::sqlite::Store,
        name: &str,
        args: &serde_json::Map<String, Value>,
    ) -> Result<String> {
        match name {
            TOOL_POLTER_FORMAT => {
                polter::format_spec(&super::helpers::string_from(args.get("topic")))
                    .map_err(Error::message)
            }
            TOOL_POLTER_QUERY => self.polter_query(store, args),
            TOOL_POLTER_APPLY => self.polter_apply(store, args),
            TOOL_DOC_UPSERT => self.doc_upsert(store, args),
            _ => {
                if let Some(replacement) = retired_replacement(name) {
                    return Err(Error::message(format!(
                        "{} was removed: use {}",
                        name, replacement
                    )));
                }
                Err(Error::message(format!(
                    "unknown tool {}: use polter.query, polter.apply, polter.format, or doc.upsert",
                    polter::go_quote(name)
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_four_tools_keep_their_names_and_order() {
        let names: Vec<String> = tool_definitions()
            .iter()
            .map(|tool| {
                tool.get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string()
            })
            .collect();
        assert_eq!(
            names,
            vec![
                TOOL_POLTER_QUERY,
                TOOL_POLTER_APPLY,
                TOOL_POLTER_FORMAT,
                TOOL_DOC_UPSERT
            ]
        );
    }

    #[test]
    fn every_tool_describes_itself() {
        for tool in tool_definitions() {
            let description = tool
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("");
            assert!(
                !description.is_empty(),
                "tool without a description: {tool}"
            );
            let schema = tool.get("inputSchema").expect("inputSchema");
            assert_eq!(schema.get("type"), Some(&json!("object")));
        }
    }

    #[test]
    fn retired_names_point_to_their_replacement() {
        for (name, replacement) in retired_tools() {
            assert_eq!(retired_replacement(name), Some(*replacement));
        }
        assert_eq!(retired_replacement("search"), None);
        assert_eq!(retired_replacement("nothing"), None);
    }
}
