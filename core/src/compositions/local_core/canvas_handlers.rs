//! The CanvasService handlers.
//!
//! The CanvasService handlers: the project's canvases, the graph converts to and
//! from its wire shape, and the audited writes that create, update, archive, and
//! delete one.

use std::sync::Arc;

use serde_json::{Map, Value};

use crate::compositions::wedge;
use crate::domains::agentrun;
use crate::domains::canvas;
use crate::local_v1;
use crate::platform::ids;
use crate::providers::sqlite;

use super::audited_write::{change_envelope_from_proto, map_archive_err, map_audited_err};
use super::connect::{Code, ConnectError, ConnectRequest, Route};
use super::server::{Core, archived_stamp, internal, invalid};

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.CanvasService/ListCanvases",
            list_canvases,
        ),
        Route::unary("/polterware.local.v1.CanvasService/GetCanvas", get_canvas),
        Route::unary(
            "/polterware.local.v1.CanvasService/CreateCanvas",
            create_canvas,
        ),
        Route::unary(
            "/polterware.local.v1.CanvasService/UpdateCanvas",
            update_canvas,
        ),
        Route::unary(
            "/polterware.local.v1.CanvasService/DeleteCanvas",
            delete_canvas,
        ),
        Route::unary(
            "/polterware.local.v1.CanvasService/ListCanvasTypes",
            list_canvas_types,
        ),
    ]
}

pub fn list_canvases(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListCanvasesRequest>,
) -> Result<local_v1::ListCanvasesResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let store = core.store();
    let items = if msg.archived {
        store.list_archived_canvases(&msg.project_id)
    } else {
        store.list_canvases(&msg.project_id)
    }
    .map_err(internal)?;
    let resolved =
        wedge::load_node_scope(&store, &msg.project_id, &msg.scope_node, msg.scope_exact)
            .map_err(|err| invalid(err.to_string()))?;
    let mut out = Vec::with_capacity(items.len());
    for item in &items {
        if !resolved.matches(&item.scope_node_id) {
            continue;
        }
        out.push(canvas_proto(item));
    }
    Ok(local_v1::ListCanvasesResponse { canvases: out })
}

pub fn get_canvas(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::GetCanvasRequest>,
) -> Result<local_v1::GetCanvasResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() || msg.id.is_empty() {
        return Err(invalid("project id and canvas id are required"));
    }
    let item = match core.store().get_canvas(&msg.project_id, &msg.id) {
        Ok(item) => item,
        Err(err) if err.is_no_rows() => {
            return Err(ConnectError::new(Code::NotFound, "canvas not found"));
        }
        Err(err) => return Err(internal(err)),
    };
    Ok(local_v1::GetCanvasResponse {
        canvas: Some(canvas_proto(&item)),
    })
}

pub fn create_canvas(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::CreateCanvasRequest>,
) -> Result<local_v1::CreateCanvasResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let mut env = change_envelope_from_proto(msg.change.as_ref()).map_err(map_audited_err)?;
    let graph = graph_from_proto(msg.graph.as_ref()).map_err(invalid)?;
    let prepared = canvas::prepare(canvas::Canvas {
        id: msg.id.clone(),
        project_id: msg.project_id.clone(),
        name: msg.name.clone(),
        r#type: msg.r#type.clone(),
        agent_visible: msg.agent_visible,
        graph,
        layout_json: msg.layout_json.clone(),
        scope_node_id: msg.scope_node_id.clone(),
        ..canvas::Canvas::default()
    })
    .map_err(map_canvas_write_err)?;
    core.store()
        .require_structure_node(&prepared.project_id, &prepared.scope_node_id)
        .map_err(map_canvas_write_err)?;
    env.project_id = prepared.project_id.clone();
    env.batch_id = ids::new_ulid();
    let batch_id = env.batch_id.clone();
    let written = {
        let store = core.store();
        wedge::execute_audited_write(
            &store,
            env,
            agentrun::ACTOR_KIND_HUMAN,
            "desktop.canvas.create",
            |run| {
                let saved = store.create_canvas(prepared, &run.id, &batch_id)?;
                Ok((vec![saved.id.clone()], saved))
            },
        )
    }
    .map_err(map_canvas_write_err)?;
    let saved = written.result;
    Ok(local_v1::CreateCanvasResponse {
        canvas: Some(canvas_proto(&saved)),
    })
}

pub fn update_canvas(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::UpdateCanvasRequest>,
) -> Result<local_v1::UpdateCanvasResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() || msg.id.is_empty() {
        return Err(invalid("project id and canvas id are required"));
    }
    let mut env = change_envelope_from_proto(msg.change.as_ref()).map_err(map_audited_err)?;
    let mut current = match core.store().get_canvas(&msg.project_id, &msg.id) {
        Ok(item) => item,
        Err(err) if err.is_no_rows() => {
            return Err(ConnectError::new(Code::NotFound, "canvas not found"));
        }
        Err(err) => return Err(internal(err)),
    };
    if current.archived() {
        return Err(map_archive_err(canvas::Error::Archived));
    }
    let before = current.graph.clone();
    let mut changed = false;
    if let Some(name) = &msg.name {
        current.name = name.clone();
        changed = true;
    }
    if let Some(agent_visible) = msg.agent_visible {
        current.agent_visible = agent_visible;
        changed = true;
    }
    if let Some(graph) = msg.graph.as_ref() {
        let graph = graph_from_proto(Some(graph)).map_err(invalid)?;
        current.graph = graph;
        changed = true;
    }
    if let Some(layout_json) = &msg.layout_json {
        current.layout_json = layout_json.clone();
        changed = true;
    }
    if let Some(scope_node_id) = &msg.scope_node_id {
        current.scope_node_id = scope_node_id.clone();
        changed = true;
    }
    if !changed {
        return Err(invalid(
            "update requires name, agent_visible, graph, layout, or scope",
        ));
    }
    let prepared = canvas::prepare(current).map_err(map_canvas_write_err)?;
    core.store()
        .require_structure_node(&prepared.project_id, &prepared.scope_node_id)
        .map_err(map_canvas_write_err)?;
    env.project_id = prepared.project_id.clone();
    env.batch_id = ids::new_ulid();
    let batch_id = env.batch_id.clone();
    // Layout edits stay out of the history; a changed relation between Structure
    // nodes is project truth, so it is listed and joins both nodes' Activity.
    let mut tool = "desktop.canvas.update";
    let (added, removed) = canvas::relation_changes(&before, &prepared.graph);
    let subjects = canvas::relation_subjects(&added, &removed);
    if !subjects.is_empty() {
        tool = "desktop.relations";
        env.summary = canvas::relation_summary(&added, &removed);
        env.rationale = format!("Drawn on the canvas {}.", prepared.name);
    }
    let written = {
        let store = core.store();
        wedge::execute_audited_write(&store, env, agentrun::ACTOR_KIND_HUMAN, tool, |run| {
            let saved = store.update_canvas(prepared, &run.id, &batch_id)?;
            let mut ids = vec![saved.id.clone()];
            ids.extend(subjects.iter().cloned());
            Ok((ids, saved))
        })
    }
    .map_err(map_canvas_write_err)?;
    let saved = written.result;
    Ok(local_v1::UpdateCanvasResponse {
        canvas: Some(canvas_proto(&saved)),
    })
}

pub fn delete_canvas(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::DeleteCanvasRequest>,
) -> Result<local_v1::DeleteCanvasResponse, ConnectError> {
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
    if !existing.archived() {
        return Err(map_archive_err(canvas::Error::NotArchived));
    }
    env.project_id = existing.project_id.clone();
    env.batch_id = ids::new_ulid();
    let batch_id = env.batch_id.clone();
    {
        let store = core.store();
        wedge::execute_audited_write(
            &store,
            env,
            agentrun::ACTOR_KIND_HUMAN,
            "desktop.canvas.delete",
            |run| {
                store.delete_canvas(&existing.project_id, &existing.id, &run.id, &batch_id)?;
                Ok((vec![existing.id.clone()], ()))
            },
        )
    }
    .map_err(map_canvas_write_err)?;
    Ok(local_v1::DeleteCanvasResponse {})
}

pub fn list_canvas_types(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListCanvasTypesRequest>,
) -> Result<local_v1::ListCanvasTypesResponse, ConnectError> {
    let defs = canvas::types();
    let mut out = Vec::with_capacity(defs.len());
    for def in &defs {
        out.push(local_v1::CanvasTypeDefinition {
            r#type: def.r#type.clone(),
            label: def.label.clone(),
            node_kinds: def.node_kinds.clone(),
            edge_kinds: def.edge_kinds.clone(),
            rules: def.rules.clone(),
            unrestricted: def.unrestricted,
        });
    }
    Ok(local_v1::ListCanvasTypesResponse { types: out })
}

/// Maps a canvas write's failure to its Connect code, as Go's
/// `mapCanvasWriteErr` does.
pub fn map_canvas_write_err(err: impl Into<sqlite::Error>) -> ConnectError {
    let err = err.into();
    if err.is_no_rows() {
        return ConnectError::new(Code::NotFound, "canvas not found");
    }
    match &err {
        sqlite::Error::Canvas(canvas::Error::NotArchived | canvas::Error::Archived) => {
            map_archive_err(err)
        }
        sqlite::Error::Canvas(
            canvas::Error::InvalidGraph(_)
            | canvas::Error::UnknownType(_)
            | canvas::Error::NameRequired
            | canvas::Error::Layout
            | canvas::Error::UnknownScope,
        ) => invalid(err.to_string()),
        _ => map_audited_err(err),
    }
}

/// Reads a graph off the wire. A missing graph is an empty one.
fn graph_from_proto(msg: Option<&local_v1::CanvasGraph>) -> Result<canvas::Graph, String> {
    let Some(msg) = msg else {
        return Ok(canvas::Graph::default());
    };
    let mut graph = canvas::Graph {
        nodes: Vec::with_capacity(msg.nodes.len()),
        edges: Vec::with_capacity(msg.edges.len()),
    };
    for (index, node) in msg.nodes.iter().enumerate() {
        let props =
            object_props(&node.props_json).map_err(|err| format!("nodes[{index}].props: {err}"))?;
        let mut item = canvas::Node {
            id: node.id.clone(),
            kind: node.kind.clone(),
            label: node.label.clone(),
            props,
            r#ref: None,
        };
        if let Some(reference) = &node.r#ref
            && (!reference.kind.is_empty() || !reference.id.is_empty())
        {
            item.r#ref = Some(canvas::Ref {
                kind: reference.kind.clone(),
                id: reference.id.clone(),
            });
        }
        graph.nodes.push(item);
    }
    for (index, edge) in msg.edges.iter().enumerate() {
        let props =
            object_props(&edge.props_json).map_err(|err| format!("edges[{index}].props: {err}"))?;
        graph.edges.push(canvas::Edge {
            id: edge.id.clone(),
            from: edge.from.clone(),
            to: edge.to.clone(),
            kind: edge.kind.clone(),
            props,
        });
    }
    Ok(graph)
}

/// Reads a JSON object, as Go's `objectProps` does: empty and `null` are the
/// empty object, and anything else must parse as an object.
fn object_props(raw: &str) -> Result<Map<String, Value>, String> {
    let raw = raw.trim();
    if raw.is_empty() || raw == "null" {
        return Ok(Map::new());
    }
    serde_json::from_str(raw).map_err(|_| "must be a JSON object".to_string())
}

pub fn canvas_proto(item: &canvas::Canvas) -> local_v1::Canvas {
    local_v1::Canvas {
        id: item.id.clone(),
        project_id: item.project_id.clone(),
        name: item.name.clone(),
        r#type: item.r#type.clone(),
        agent_visible: item.agent_visible,
        graph: Some(graph_to_proto(&item.graph)),
        layout_json: item.layout_json.clone(),
        scope_node_id: item.scope_node_id.clone(),
        created_at: sqlite::format_rfc3339(item.created_at),
        updated_at: sqlite::format_rfc3339(item.updated_at),
        archived_at: archived_stamp(item.archived_at),
    }
}

fn graph_to_proto(graph: &canvas::Graph) -> local_v1::CanvasGraph {
    let mut msg = local_v1::CanvasGraph {
        nodes: Vec::with_capacity(graph.nodes.len()),
        edges: Vec::with_capacity(graph.edges.len()),
    };
    for node in &graph.nodes {
        msg.nodes.push(local_v1::CanvasNode {
            id: node.id.clone(),
            kind: node.kind.clone(),
            label: node.label.clone(),
            props_json: must_props(&node.props),
            r#ref: node.r#ref.as_ref().map(|reference| local_v1::CanvasRef {
                kind: reference.kind.clone(),
                id: reference.id.clone(),
            }),
        });
    }
    for edge in &graph.edges {
        msg.edges.push(local_v1::CanvasEdge {
            id: edge.id.clone(),
            from: edge.from.clone(),
            to: edge.to.clone(),
            kind: edge.kind.clone(),
            props_json: must_props(&edge.props),
        });
    }
    msg
}

/// Go's `json.Marshal` of a props object: sorted keys, `<`, `>`, `&`, and the
/// two line separators escaped, and numbers written the way Go writes them.
fn must_props(props: &Map<String, Value>) -> String {
    let mut out = String::new();
    write_object(&mut out, props);
    out
}

fn write_object(out: &mut String, fields: &Map<String, Value>) {
    out.push('{');
    let mut names: Vec<&String> = fields.keys().collect();
    names.sort();
    for (index, name) in names.into_iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        write_string(out, name);
        out.push(':');
        write_value(out, &fields[name.as_str()]);
    }
    out.push('}');
}

fn write_value(out: &mut String, value: &Value) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(number) => out.push_str(&go_number(number)),
        Value::String(text) => write_string(out, text),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_value(out, item);
            }
            out.push(']');
        }
        Value::Object(fields) => write_object(out, fields),
    }
}

/// `encoding/json` escapes the quote, the backslash, `\n`, `\r`, `\t`, every
/// other control character as `\u00XX`, and the HTML characters and the two
/// line separators.
fn write_string(out: &mut String, value: &str) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn go_number(number: &serde_json::Number) -> String {
    if let Some(value) = number.as_i64() {
        return value.to_string();
    }
    if let Some(value) = number.as_u64() {
        return value.to_string();
    }
    go_float(number.as_f64().unwrap_or_default())
}

/// Go's float encoding: plain decimal unless the magnitude is below `1e-6` or
/// at least `1e21`, where it switches to an exponent.
fn go_float(value: f64) -> String {
    let magnitude = value.abs();
    if magnitude != 0.0 && !(1e-6..1e21).contains(&magnitude) {
        return go_exponent(value);
    }
    format!("{value}")
}

/// `strconv.FormatFloat(value, 'e', -1, 64)` with Go's cleanup: a negative
/// exponent under ten loses its leading zero (`e-09` becomes `e-9`).
fn go_exponent(value: f64) -> String {
    let text = format!("{value:e}");
    let Some((mantissa, exponent)) = text.split_once('e') else {
        return text;
    };
    let Ok(exponent) = exponent.parse::<i32>() else {
        return text;
    };
    let digits = exponent.unsigned_abs();
    if exponent < 0 && digits < 10 {
        return format!("{mantissa}e-{digits}");
    }
    let sign = if exponent < 0 { '-' } else { '+' };
    format!("{mantissa}e{sign}{digits:02}")
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
            summary: "Save canvas".to_string(),
            rationale: "The roadmap needs a surface".to_string(),
            ..local_v1::ChangeEnvelope::default()
        }
    }

    #[test]
    fn must_props_marshals_like_go() {
        let props: Map<String, Value> = serde_json::from_str(
            r#"{"z":1,"a":"<tag> & \"quote\"","n":1.5,"whole":1.0,"list":[true,null]}"#,
        )
        .expect("props");
        assert_eq!(
            must_props(&props),
            r#"{"a":"\u003ctag\u003e \u0026 \"quote\"","list":[true,null],"n":1.5,"whole":1,"z":1}"#
        );
        assert_eq!(go_float(1e21), "1e+21");
        assert_eq!(go_float(1.5e-7), "1.5e-7");
        assert_eq!(go_float(-0.0), "-0");
        assert_eq!(go_float(2.5), "2.5");
        assert_eq!(must_props(&Map::new()), "{}");
    }

    #[test]
    fn props_must_be_a_json_object() {
        assert_eq!(object_props("  ").unwrap(), Map::new());
        assert_eq!(object_props("null").unwrap(), Map::new());
        assert_eq!(
            object_props("[1]").unwrap_err(),
            "must be a JSON object".to_string()
        );
    }

    #[test]
    fn canvas_service_rejects_and_updates_parts() {
        let harness = Harness::start();
        let core = Arc::clone(&harness.core);
        let now = OffsetDateTime::now_utc();
        core.store()
            .save_project(projects::Project {
                id: "proj-canvas".to_string(),
                name: "Canvas".to_string(),
                created_at: now,
                updated_at: now,
                ..projects::Project::default()
            })
            .expect("project");
        let change = change();

        let types = list_canvas_types(
            Arc::clone(&core),
            request(local_v1::ListCanvasTypesRequest::default()),
        )
        .expect("types");
        assert_eq!(types.types.len(), 1);
        let mut saw_note = false;
        for item in &types.types {
            if item.r#type != "canvas" {
                continue;
            }
            if item.node_kinds.iter().any(|kind| kind == "note") {
                saw_note = true;
            }
        }
        assert!(saw_note, "canvas palette must include note");

        let err = create_canvas(
            Arc::clone(&core),
            request(local_v1::CreateCanvasRequest {
                project_id: "proj-canvas".to_string(),
                name: "Broken".to_string(),
                r#type: "canvas".to_string(),
                change: Some(change.clone()),
                graph: Some(local_v1::CanvasGraph {
                    nodes: vec![local_v1::CanvasNode {
                        id: "a".to_string(),
                        kind: "spaceship".to_string(),
                        ..local_v1::CanvasNode::default()
                    }],
                    ..local_v1::CanvasGraph::default()
                }),
                ..local_v1::CreateCanvasRequest::default()
            }),
        )
        .expect_err("bad kind");
        assert_eq!(err.code, Code::InvalidArgument);
        assert!(err.message.contains("not allowed"), "{err}");

        let err = create_canvas(
            Arc::clone(&core),
            request(local_v1::CreateCanvasRequest {
                project_id: "proj-canvas".to_string(),
                name: "Sketch".to_string(),
                r#type: "canvas".to_string(),
                change: Some(change.clone()),
                graph: Some(local_v1::CanvasGraph {
                    nodes: vec![local_v1::CanvasNode {
                        id: "s".to_string(),
                        kind: "domain".to_string(),
                        label: "API".to_string(),
                        ..local_v1::CanvasNode::default()
                    }],
                    ..local_v1::CanvasGraph::default()
                }),
                ..local_v1::CreateCanvasRequest::default()
            }),
        )
        .expect_err("missing ref");
        assert_eq!(err.code, Code::InvalidArgument);
        assert!(err.message.contains("structure node"), "{err}");

        let note_only = create_canvas(
            Arc::clone(&core),
            request(local_v1::CreateCanvasRequest {
                project_id: "proj-canvas".to_string(),
                name: "Sketch".to_string(),
                r#type: "canvas".to_string(),
                change: Some(change.clone()),
                graph: Some(local_v1::CanvasGraph {
                    nodes: vec![local_v1::CanvasNode {
                        id: "n".to_string(),
                        kind: "note".to_string(),
                        label: "later".to_string(),
                        ..local_v1::CanvasNode::default()
                    }],
                    ..local_v1::CanvasGraph::default()
                }),
                ..local_v1::CreateCanvasRequest::default()
            }),
        )
        .expect("note on canvas");
        assert_eq!(
            note_only
                .canvas
                .as_ref()
                .expect("canvas")
                .graph
                .as_ref()
                .expect("graph")
                .nodes[0]
                .kind,
            "note"
        );

        let created = create_canvas(
            Arc::clone(&core),
            request(local_v1::CreateCanvasRequest {
                project_id: "proj-canvas".to_string(),
                name: "Platform".to_string(),
                r#type: "canvas".to_string(),
                agent_visible: true,
                change: Some(change.clone()),
                graph: Some(local_v1::CanvasGraph {
                    nodes: vec![
                        local_v1::CanvasNode {
                            id: "a".to_string(),
                            kind: "service".to_string(),
                            label: "Import".to_string(),
                            props_json: r#"{"technology":"Go"}"#.to_string(),
                            ..local_v1::CanvasNode::default()
                        },
                        local_v1::CanvasNode {
                            id: "n".to_string(),
                            kind: "note".to_string(),
                            label: "ask".to_string(),
                            ..local_v1::CanvasNode::default()
                        },
                    ],
                    edges: vec![local_v1::CanvasEdge {
                        id: "e".to_string(),
                        from: "a".to_string(),
                        to: "n".to_string(),
                        kind: "depends_on".to_string(),
                        ..local_v1::CanvasEdge::default()
                    }],
                }),
                ..local_v1::CreateCanvasRequest::default()
            }),
        )
        .expect("create canvas");
        let created = created.canvas.expect("canvas");
        let canvas_id = created.id.clone();
        assert!(!canvas_id.is_empty(), "created = {created:?}");
        assert_eq!(created.layout_json, "{}");
        assert!(created.agent_visible);

        let (run_id, entity): (String, String) = core
            .store()
            .db
            .query_row(
                r#"SELECT agent_run_id, entity FROM change_ops WHERE entity_id=? AND op='create'"#,
                rusqlite::params![canvas_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("audit row");
        assert_eq!(entity, "canvas");
        assert!(!run_id.is_empty());
        let record = core
            .store()
            .get_agent_run_record(&run_id)
            .expect("run record");
        assert_eq!(record.run.actor_kind, "human");

        let name = "Platform map".to_string();
        let renamed = update_canvas(
            Arc::clone(&core),
            request(local_v1::UpdateCanvasRequest {
                project_id: "proj-canvas".to_string(),
                id: canvas_id.clone(),
                change: Some(change.clone()),
                name: Some(name.clone()),
                ..local_v1::UpdateCanvasRequest::default()
            }),
        )
        .expect("rename");
        let renamed = renamed.canvas.expect("canvas");
        assert_eq!(renamed.name, name);
        assert_eq!(
            renamed.graph.as_ref().expect("graph").nodes.len(),
            2,
            "name update clobbered other parts: {renamed:?}"
        );
        assert_eq!(renamed.layout_json, "{}");

        let layout = r#"{"nodes":{"a":{"x":4,"y":8}}}"#.to_string();
        let moved = update_canvas(
            Arc::clone(&core),
            request(local_v1::UpdateCanvasRequest {
                project_id: "proj-canvas".to_string(),
                id: canvas_id.clone(),
                change: Some(change.clone()),
                layout_json: Some(layout.clone()),
                ..local_v1::UpdateCanvasRequest::default()
            }),
        )
        .expect("layout");
        let moved = moved.canvas.expect("canvas");
        assert_eq!(moved.layout_json, layout);
        assert_eq!(moved.name, name);
        let graph = moved.graph.as_ref().expect("graph");
        assert!(
            graph.nodes[0].props_json.contains("technology"),
            "layout update changed the graph: {graph:?}"
        );

        let err = update_canvas(
            Arc::clone(&core),
            request(local_v1::UpdateCanvasRequest {
                project_id: "proj-canvas".to_string(),
                id: canvas_id.clone(),
                change: Some(change.clone()),
                graph: Some(local_v1::CanvasGraph {
                    nodes: vec![local_v1::CanvasNode {
                        id: "a".to_string(),
                        kind: "spaceship".to_string(),
                        ..local_v1::CanvasNode::default()
                    }],
                    ..local_v1::CanvasGraph::default()
                }),
                ..local_v1::UpdateCanvasRequest::default()
            }),
        )
        .expect_err("bad graph update");
        assert_eq!(err.code, Code::InvalidArgument);

        let got = get_canvas(
            Arc::clone(&core),
            request(local_v1::GetCanvasRequest {
                project_id: "proj-canvas".to_string(),
                id: canvas_id.clone(),
            }),
        )
        .expect("get");
        let got = got.canvas.expect("canvas");
        assert_eq!(got.graph.as_ref().expect("graph").edges.len(), 1);
        assert_eq!(got.layout_json, layout);

        let err = delete_canvas(
            Arc::clone(&core),
            request(local_v1::DeleteCanvasRequest {
                project_id: "proj-canvas".to_string(),
                id: canvas_id.clone(),
                change: Some(change.clone()),
            }),
        )
        .expect_err("delete of an active canvas");
        assert_eq!(err.code, Code::FailedPrecondition);

        let archived = crate::compositions::local_core::archive_handlers::set_canvas_archived(
            Arc::clone(&core),
            request(local_v1::SetCanvasArchivedRequest {
                project_id: "proj-canvas".to_string(),
                id: canvas_id.clone(),
                archived: true,
                change: Some(change.clone()),
            }),
        )
        .expect("archive");
        assert!(!archived.canvas.expect("canvas").archived_at.is_empty());

        let err = update_canvas(
            Arc::clone(&core),
            request(local_v1::UpdateCanvasRequest {
                project_id: "proj-canvas".to_string(),
                id: canvas_id.clone(),
                change: Some(change.clone()),
                name: Some("Renamed".to_string()),
                ..local_v1::UpdateCanvasRequest::default()
            }),
        )
        .expect_err("update of an archived canvas");
        assert_eq!(err.code, Code::FailedPrecondition);

        let listed = list_canvases(
            Arc::clone(&core),
            request(local_v1::ListCanvasesRequest {
                project_id: "proj-canvas".to_string(),
                archived: true,
                ..local_v1::ListCanvasesRequest::default()
            }),
        )
        .expect("archived list");
        assert_eq!(listed.canvases.len(), 1);

        delete_canvas(
            Arc::clone(&core),
            request(local_v1::DeleteCanvasRequest {
                project_id: "proj-canvas".to_string(),
                id: canvas_id.clone(),
                change: Some(change.clone()),
            }),
        )
        .expect("delete");

        let err = get_canvas(
            Arc::clone(&core),
            request(local_v1::GetCanvasRequest {
                project_id: "proj-canvas".to_string(),
                id: canvas_id,
            }),
        )
        .expect_err("deleted get");
        assert_eq!(err.code, Code::NotFound);
    }
}
