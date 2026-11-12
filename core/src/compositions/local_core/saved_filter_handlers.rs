//! The saved filter handlers.
//!
//! Saved filters are UI preferences: these handlers write the store directly,
//! without an audited change envelope.

use std::sync::Arc;

use crate::domains::filters;
use crate::local_v1;
use crate::providers::sqlite;

use super::connect::{Code, ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid};

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.SavedFilterService/ListSavedFilters",
            list_saved_filters,
        ),
        Route::unary(
            "/polterware.local.v1.SavedFilterService/SaveSavedFilter",
            save_saved_filter,
        ),
        Route::unary(
            "/polterware.local.v1.SavedFilterService/DeleteSavedFilter",
            delete_saved_filter,
        ),
    ]
}

pub fn list_saved_filters(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListSavedFiltersRequest>,
) -> Result<local_v1::ListSavedFiltersResponse, ConnectError> {
    let msg = request.message;
    let surface = msg.surface.trim();
    filters::validate_surface(surface).map_err(map_saved_filter_err)?;
    let items = core
        .store()
        .list_saved_filters(&msg.project_id, surface)
        .map_err(internal)?;
    Ok(local_v1::ListSavedFiltersResponse {
        filters: items.iter().map(saved_filter_proto).collect(),
    })
}

pub fn save_saved_filter(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SaveSavedFilterRequest>,
) -> Result<local_v1::SaveSavedFilterResponse, ConnectError> {
    let msg = request.message;
    let saved = core
        .store()
        .save_saved_filter(filters::SavedFilter {
            project_id: msg.project_id,
            surface: msg.surface,
            name: msg.name,
            payload_json: msg.payload_json,
            ..filters::SavedFilter::default()
        })
        .map_err(map_saved_filter_err)?;
    Ok(local_v1::SaveSavedFilterResponse {
        filter: Some(saved_filter_proto(&saved)),
    })
}

pub fn delete_saved_filter(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::DeleteSavedFilterRequest>,
) -> Result<local_v1::DeleteSavedFilterResponse, ConnectError> {
    let id = request.message.id.trim();
    if id.is_empty() {
        return Err(invalid("saved filter id is required"));
    }
    core.store()
        .delete_saved_filter(id)
        .map_err(map_saved_filter_err)?;
    Ok(local_v1::DeleteSavedFilterResponse {})
}

/// Maps a saved filter failure: a refused field is `invalid_argument`, an
/// unknown filter or project is `not_found`, anything else `internal`.
pub fn map_saved_filter_err(err: impl Into<sqlite::Error>) -> ConnectError {
    let err = err.into();
    match &err {
        sqlite::Error::Filters(
            filters::SavedFilterError::SurfaceRequired
            | filters::SavedFilterError::SurfaceLength
            | filters::SavedFilterError::NameRequired
            | filters::SavedFilterError::NameLength
            | filters::SavedFilterError::Payload,
        ) => invalid(err.to_string()),
        sqlite::Error::Filters(
            filters::SavedFilterError::NotFound | filters::SavedFilterError::ProjectNotFound,
        ) => ConnectError::new(Code::NotFound, err.to_string()),
        _ => internal(err),
    }
}

pub fn saved_filter_proto(item: &filters::SavedFilter) -> local_v1::SavedFilter {
    local_v1::SavedFilter {
        id: item.id.clone(),
        project_id: item.project_id.clone(),
        surface: item.surface.clone(),
        name: item.name.clone(),
        payload_json: item.payload_json.clone(),
        position: item.position,
        created_at: sqlite::format_rfc3339(item.created_at),
        updated_at: sqlite::format_rfc3339(item.updated_at),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::compositions::local_core::test_support::{Harness, TEST_TOKEN};
    use crate::providers::seed;

    const SERVICE: &str = "/polterware.local.v1.SavedFilterService";

    fn post(harness: &Harness, method: &str, body: &str) -> (u16, Value) {
        let reply = harness.unary(&format!("{SERVICE}/{method}"), Some(TEST_TOKEN), body);
        (reply.status, reply.json())
    }

    #[test]
    fn saved_filter_service_over_rpc() {
        let harness = Harness::start();
        let project_id = seed::PROJECT_ID;

        let missing = harness.unary(
            &format!("{SERVICE}/ListSavedFilters"),
            None,
            r#"{"surface":"work"}"#,
        );
        assert_eq!(missing.status, 401);
        assert_eq!(missing.error_code(), "unauthenticated");

        let (status, mine) = post(
            &harness,
            "SaveSavedFilter",
            &format!(
                r#"{{"projectId":"{project_id}","surface":"work","name":"Mine","payloadJson":"{{\"version\":1,\"clauses\":[]}}"}}"#
            ),
        );
        assert_eq!(status, 200, "{mine}");
        let mine_id = mine["filter"]["id"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        assert!(!mine_id.is_empty(), "save = {mine}");
        assert_eq!(mine["filter"]["projectId"], project_id);
        assert_eq!(
            mine["filter"]["position"].as_i64(),
            None,
            "zero is left out"
        );
        assert!(
            mine["filter"]["createdAt"]
                .as_str()
                .is_some_and(|stamp| !stamp.is_empty()),
            "save = {mine}"
        );

        let (status, again) = post(
            &harness,
            "SaveSavedFilter",
            &format!(
                r#"{{"projectId":"{project_id}","surface":"work","name":"Mine","payloadJson":"{{\"version\":1,\"clauses\":[{{\"field\":\"stage\"}}]}}"}}"#
            ),
        );
        assert_eq!(status, 200, "{again}");
        assert_eq!(again["filter"]["id"], mine_id.as_str());
        assert_eq!(
            again["filter"]["payloadJson"],
            r#"{"version":1,"clauses":[{"field":"stage"}]}"#
        );

        let (status, app) = post(
            &harness,
            "SaveSavedFilter",
            r#"{"surface":"projects","name":"Active","payloadJson":"{\"version\":1}"}"#,
        );
        assert_eq!(status, 200, "{app}");
        assert_eq!(app["filter"]["projectId"].as_str(), None);
        let app_id = app["filter"]["id"].as_str().unwrap_or_default().to_string();

        let (status, listed) = post(
            &harness,
            "ListSavedFilters",
            &format!(r#"{{"projectId":"{project_id}","surface":"work"}}"#),
        );
        assert_eq!(status, 200, "{listed}");
        let filters = listed["filters"].as_array().cloned().unwrap_or_default();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0]["id"], mine_id.as_str());

        let (status, listed) = post(&harness, "ListSavedFilters", r#"{"surface":"projects"}"#);
        assert_eq!(status, 200, "{listed}");
        let filters = listed["filters"].as_array().cloned().unwrap_or_default();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0]["id"], app_id.as_str());

        for (name, body) in [
            (
                "blank surface",
                r#"{"surface":" ","name":"a","payloadJson":"{\"version\":1}"}"#.to_string(),
            ),
            (
                "blank name",
                r#"{"surface":"work","name":"","payloadJson":"{\"version\":1}"}"#.to_string(),
            ),
            (
                "no version",
                r#"{"surface":"work","name":"a","payloadJson":"{\"clauses\":[]}"}"#.to_string(),
            ),
            (
                "not an object",
                r#"{"surface":"work","name":"a","payloadJson":"[1]"}"#.to_string(),
            ),
        ] {
            let (status, body) = post(&harness, "SaveSavedFilter", &body);
            assert_eq!(status, 400, "{name}: {body}");
            assert_eq!(body["code"], "invalid_argument", "{name}");
        }

        let (status, body) = post(
            &harness,
            "SaveSavedFilter",
            r#"{"projectId":"missing","surface":"work","name":"a","payloadJson":"{\"version\":1}"}"#,
        );
        assert_eq!(status, 404, "{body}");
        assert_eq!(body["code"], "not_found");

        let (status, body) = post(
            &harness,
            "ListSavedFilters",
            &format!(r#"{{"projectId":"{project_id}"}}"#),
        );
        assert_eq!(status, 400, "{body}");
        assert_eq!(body["code"], "invalid_argument");

        let (status, body) = post(
            &harness,
            "DeleteSavedFilter",
            &format!(r#"{{"id":"{mine_id}"}}"#),
        );
        assert_eq!(status, 200, "{body}");
        let (status, body) = post(
            &harness,
            "DeleteSavedFilter",
            &format!(r#"{{"id":"{mine_id}"}}"#),
        );
        assert_eq!(status, 404, "{body}");
        assert_eq!(body["code"], "not_found");
        let (status, body) = post(&harness, "DeleteSavedFilter", "{}");
        assert_eq!(status, 400, "{body}");
        assert_eq!(body["code"], "invalid_argument");

        let (_, listed) = post(
            &harness,
            "ListSavedFilters",
            &format!(r#"{{"projectId":"{project_id}","surface":"work"}}"#),
        );
        assert!(
            listed["filters"]
                .as_array()
                .map(Vec::is_empty)
                .unwrap_or(true),
            "after delete = {listed}"
        );

        let change_ops: i64 = harness
            .core
            .store()
            .db
            .query_row(
                r#"SELECT COUNT(*) FROM change_ops WHERE entity_id IN (?, ?)"#,
                rusqlite::params![mine_id, app_id],
                |row| row.get(0),
            )
            .expect("change ops");
        assert_eq!(change_ops, 0, "saved filters are not audited");
    }
}
