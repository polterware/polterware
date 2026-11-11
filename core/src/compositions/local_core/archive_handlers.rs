//! Port of `backend/compositions/local-core/archive_handlers.go`.
//!
//! Archiving and restoring one knowledge item or one canvas. A Decision is
//! refused: it moves through its lifecycle instead. Go's `mapArchiveErr` and
//! `archivedStamp` live beside their first callers in the port, in
//! [`super::audited_write::map_archive_err`] and
//! [`super::server::archived_stamp`].

use std::sync::Arc;

use time::OffsetDateTime;

use crate::compositions::wedge;
use crate::domains::agentrun;
use crate::domains::knowledge;
use crate::local_v1;
use crate::platform::ids;

use super::audited_write::{change_envelope_from_proto, map_archive_err, map_audited_err};
use super::canvas_handlers::map_canvas_write_err;
use super::connect::{Code, ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid, knowledge_proto};

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.CanvasService/SetCanvasArchived",
            set_canvas_archived,
        ),
        Route::unary(
            "/polterware.local.v1.KnowledgeService/SetKnowledgeArchived",
            set_knowledge_archived,
        ),
    ]
}

/// Archives or restores one knowledge item. Decisions are refused: they move
/// through their lifecycle instead.
pub fn set_knowledge_archived(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SetKnowledgeArchivedRequest>,
) -> Result<local_v1::SetKnowledgeArchivedResponse, ConnectError> {
    let msg = request.message;
    if msg.id.is_empty() {
        return Err(invalid("knowledge id is required"));
    }
    let mut env = change_envelope_from_proto(msg.change.as_ref()).map_err(map_audited_err)?;
    let existing = match core.store().get_knowledge(&msg.id) {
        Ok(item) => item,
        Err(err) if err.is_no_rows() => {
            return Err(ConnectError::new(Code::NotFound, "unknown knowledge item"));
        }
        Err(err) => return Err(internal(err)),
    };
    if existing.kind == knowledge::KIND_DECISION {
        return Err(map_archive_err(knowledge::KnowledgeError::DecisionArchive));
    }
    env.project_id = existing.project_id.clone();
    let tool = if msg.archived {
        "desktop.knowledge.archive"
    } else {
        "desktop.knowledge.restore"
    };
    let written = {
        let store = core.store();
        wedge::execute_audited_write(&store, env, agentrun::ACTOR_KIND_HUMAN, tool, |_run| {
            let saved = store.set_knowledge_archived(
                &existing.id,
                msg.archived,
                OffsetDateTime::now_utc(),
            )?;
            Ok((vec![saved.id.clone()], saved))
        })
    }
    .map_err(map_archive_err)?;
    Ok(local_v1::SetKnowledgeArchivedResponse {
        item: Some(knowledge_proto(&written.result)),
    })
}

/// Archives or restores one canvas.
pub fn set_canvas_archived(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SetCanvasArchivedRequest>,
) -> Result<local_v1::SetCanvasArchivedResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() || msg.id.is_empty() {
        return Err(invalid("project id and canvas id are required"));
    }
    let mut env = change_envelope_from_proto(msg.change.as_ref()).map_err(map_audited_err)?;
    let existing = match core.store().get_canvas(&msg.project_id, &msg.id) {
        Ok(item) => item,
        Err(err) if err.is_no_rows() => {
            return Err(ConnectError::new(Code::NotFound, "canvas not found"));
        }
        Err(err) => return Err(internal(err)),
    };
    env.project_id = existing.project_id.clone();
    env.batch_id = ids::new_ulid();
    let batch_id = env.batch_id.clone();
    let tool = if msg.archived {
        "desktop.canvas.archive"
    } else {
        "desktop.canvas.restore"
    };
    let written = {
        let store = core.store();
        wedge::execute_audited_write(&store, env, agentrun::ACTOR_KIND_HUMAN, tool, |run| {
            let saved = store.set_canvas_archived(
                &existing.project_id,
                &existing.id,
                msg.archived,
                OffsetDateTime::now_utc(),
                &run.id,
                &batch_id,
            )?;
            Ok((vec![saved.id.clone()], saved))
        })
    }
    .map_err(map_canvas_write_err)?;
    Ok(local_v1::SetCanvasArchivedResponse {
        canvas: Some(super::canvas_handlers::canvas_proto(&written.result)),
    })
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderMap;
    use time::OffsetDateTime;

    use crate::compositions::local_core::test_support::Harness;
    use crate::domains::projects;

    use super::*;

    fn request<T>(message: T) -> ConnectRequest<T> {
        ConnectRequest {
            headers: HeaderMap::new(),
            message,
        }
    }

    fn change() -> local_v1::ChangeEnvelope {
        local_v1::ChangeEnvelope {
            summary: "Archive".to_string(),
            rationale: "No longer current".to_string(),
            ..local_v1::ChangeEnvelope::default()
        }
    }

    fn failed(err: &ConnectError) -> bool {
        err.code == Code::FailedPrecondition
    }

    #[test]
    fn knowledge_archive_rules() {
        let harness = Harness::start();
        let core = Arc::clone(&harness.core);
        let now = OffsetDateTime::now_utc();
        {
            let store = core.store();
            store
                .save_project(projects::Project {
                    id: "proj-archive".to_string(),
                    name: "Archive".to_string(),
                    created_at: now,
                    updated_at: now,
                    ..projects::Project::default()
                })
                .expect("project");
            for (id, kind) in [
                ("doc-1", knowledge::KIND_DOC),
                ("dec-1", knowledge::KIND_DECISION),
            ] {
                let saved = knowledge::new_item(
                    id,
                    "proj-archive",
                    &format!("Item {id}"),
                    "body",
                    kind,
                    Vec::new(),
                )
                .expect("knowledge item");
                store.save_knowledge(saved).expect("save knowledge");
            }
        }
        let change = change();

        let err = set_knowledge_archived(
            Arc::clone(&core),
            request(local_v1::SetKnowledgeArchivedRequest {
                id: "dec-1".to_string(),
                archived: true,
                change: Some(change.clone()),
            }),
        )
        .expect_err("archive of a decision");
        assert!(failed(&err), "{err}");

        let err = super::super::server::delete_knowledge(
            Arc::clone(&core),
            request(local_v1::DeleteKnowledgeRequest {
                id: "dec-1".to_string(),
                change: Some(change.clone()),
            }),
        )
        .expect_err("delete of a decision");
        assert!(failed(&err), "{err}");

        let err = super::super::server::delete_knowledge(
            Arc::clone(&core),
            request(local_v1::DeleteKnowledgeRequest {
                id: "doc-1".to_string(),
                change: Some(change.clone()),
            }),
        )
        .expect_err("delete of an active item");
        assert!(failed(&err), "{err}");

        let archived = set_knowledge_archived(
            Arc::clone(&core),
            request(local_v1::SetKnowledgeArchivedRequest {
                id: "doc-1".to_string(),
                archived: true,
                change: Some(change.clone()),
            }),
        )
        .expect("archive");
        assert!(!archived.item.expect("item").archived_at.is_empty());

        let err = super::super::server::save_knowledge(
            Arc::clone(&core),
            request(local_v1::SaveKnowledgeRequest {
                item: Some(local_v1::KnowledgeItem {
                    id: "doc-1".to_string(),
                    project_id: "proj-archive".to_string(),
                    title: "Edited".to_string(),
                    kind: knowledge::KIND_DOC.to_string(),
                    ..local_v1::KnowledgeItem::default()
                }),
                change: Some(change.clone()),
            }),
        )
        .expect_err("edit of an archived item");
        assert!(failed(&err), "{err}");

        let listed = super::super::server::list_knowledge(
            Arc::clone(&core),
            request(local_v1::ListKnowledgeRequest {
                project_id: "proj-archive".to_string(),
                archived: true,
                ..local_v1::ListKnowledgeRequest::default()
            }),
        )
        .expect("archived list");
        assert_eq!(listed.items.len(), 1);
        assert_eq!(listed.items[0].id, "doc-1");

        super::super::server::delete_knowledge(
            Arc::clone(&core),
            request(local_v1::DeleteKnowledgeRequest {
                id: "doc-1".to_string(),
                change: Some(change),
            }),
        )
        .expect("delete");
    }
}
