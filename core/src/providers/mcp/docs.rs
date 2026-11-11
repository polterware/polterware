//! Port of `backend/providers/mcp/docs.go`: the doc write, whose Markdown body
//! does not fit Polter text.

use serde_json::{Map, Value};

use crate::compositions::wedge;
use crate::domains::agentrun;
use crate::domains::knowledge;
use crate::domains::polter;
use crate::providers::sqlite::{self, SCOPE_KNOWLEDGE, Store};

use super::helpers::{string_from, string_slice_from};
use super::{Error, Result, TOOL_DOC_UPSERT};

impl super::Server {
    /// Creates or updates one doc as an audited write in this session's run.
    /// On update, omitted fields keep their value.
    pub(crate) fn doc_upsert(
        &mut self,
        store: &Store,
        args: &Map<String, Value>,
    ) -> Result<String> {
        let project_id = string_from(args.get("project_id"));
        if project_id.is_empty() {
            return Err(Error::message("project_id is required"));
        }
        let summary = string_from(args.get("summary"));
        let rationale = string_from(args.get("rationale"));
        if let Err(error) = agentrun::validate_change(&summary, &rationale) {
            return Err(Error::message(format!(
                "{error}: pass summary and rationale"
            )));
        }
        let id = string_from(args.get("id")).trim().to_string();
        let mut item: knowledge::Item;
        if !id.is_empty() {
            let existing = store.get_knowledge(&id);
            let existing = match existing {
                Ok(existing)
                    if existing.project_id == project_id
                        && existing.kind == knowledge::KIND_DOC =>
                {
                    existing
                }
                _ => return Err(unknown_doc(&id)),
            };
            if existing.archived() {
                return Err(Error::message(
                    knowledge::KnowledgeError::Archived.to_string(),
                ));
            }
            item = existing;
        } else {
            let created = knowledge::new_item(
                &knowledge::new_item_id(),
                &project_id,
                &string_from(args.get("title")),
                &string_from(args.get("body")),
                knowledge::KIND_DOC,
                string_slice_from(args.get("tags")),
            )
            .map_err(|error| Error::message(format!("a new doc needs a title: {error}")))?;
            item = created;
            item.owner = agentrun::ACTOR_AGENT.to_string();
        }
        if let Some(Value::String(title)) = args.get("title")
            && !title.trim().is_empty()
        {
            item.title = title.trim().to_string();
        }
        if let Some(Value::String(body)) = args.get("body") {
            item.body = body.clone();
        }
        if args.contains_key("tags") {
            item.tags = string_slice_from(args.get("tags"));
        }
        // Docs have no types; an older doc_type argument may only not name the
        // retired source.
        if let Some(Value::String(doc_type)) = args.get("doc_type") {
            knowledge::check_doc_type(doc_type)
                .map_err(|error| Error::message(error.to_string()))?;
        }
        item.doc_type = knowledge::DOC_TYPE_DOC.to_string();

        let mut scope_nodes: Vec<String> = Vec::new();
        let mut set_scope = false;
        if let Some(Value::String(scope)) = args.get("scope") {
            set_scope = true;
            let refs = wedge::scope_refs("", std::slice::from_ref(scope));
            if !refs.is_empty() {
                let resolved =
                    wedge::resolve_scope_refs(store, &project_id, &refs, true).map_err(|_| {
                        Error::message(format!(
                            "unknown scope {}: use Structure node paths, comma separated",
                            polter::go_quote(scope)
                        ))
                    })?;
                scope_nodes = resolved.node_ids;
            }
        }
        let mut run_id = string_from(args.get("agent_run_id"));
        if run_id.is_empty() {
            run_id = self
                .session
                .run_for(store, &project_id, &summary)
                .map_err(|error| Error::message(error.to_string()))?;
        }
        let item_id = item.id.clone();
        let title = item.title.clone();
        wedge::execute_audited_write(
            store,
            wedge::ChangeEnvelope {
                summary,
                rationale,
                agent_run_id: run_id.clone(),
                project_id: project_id.clone(),
                ..wedge::ChangeEnvelope::default()
            },
            agentrun::ACTOR_KIND_AGENT,
            TOOL_DOC_UPSERT,
            |_run| {
                store.save_knowledge(item.clone())?;
                if set_scope {
                    store.set_scope(&project_id, SCOPE_KNOWLEDGE, &item.id, &scope_nodes)?;
                }
                Ok((vec![item_id.clone()], ()))
            },
        )
        .map_err(|error| match &error {
            sqlite::Error::Knowledge(knowledge::KnowledgeError::Invalid) => {
                Error::message(format!("invalid doc: {error}"))
            }
            other => Error::message(other.to_string()),
        })?;
        let verb = if id.is_empty() { "created" } else { "updated" };
        Ok(format!(
            "{verb} doc {item_id} {} (agent_run_id {run_id})\n",
            polter::go_quote(&title)
        ))
    }
}

/// The message a doc write answers for an id that is not one of the project's
/// docs.
fn unknown_doc(id: &str) -> Error {
    Error::message(format!(
        "unknown doc {}: search <terms> finds docs, or omit id to create one",
        polter::go_quote(id)
    ))
}
