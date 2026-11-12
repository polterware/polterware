use std::collections::BTreeMap;
use std::io::Cursor;

use polterware_core::domains::{canvas, context};
use polterware_core::providers::{mcp, seed, sqlite::Store};
use serde_json::{Value, json};

// These ceilings ratchet the text an agent receives from the seeded project.
const TOKEN_CEILINGS: &[(&str, i32)] = &[
    ("mcp.initialize", 237),
    ("mcp.tools_list", 738),
    ("polter.format", 397),
    ("card.context", 341),
    // The seeded Structure text grew by one token when its demo nodes were
    // retitled for the Rules model.
    ("node.context", 267),
    ("search", 36),
    // The canvas source is the whole canvas: ids, kinds, refs, and props ride
    // in each node's metadata so the text reads back unchanged. It grew when
    // nodes gained `shape:` beside `kind:` and the `classDef unverified` line.
    ("canvas.show", 257),
];

#[test]
fn test_agent_read_token_budget() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().join("budget.db")).unwrap();
    seed::polterware(&store, dir.path().to_str().unwrap()).unwrap();
    seed_architecture_canvas(&store);

    let reads = [
        ("mcp.initialize", json!({"method":"initialize","params":{}})),
        ("mcp.tools_list", json!({"method":"tools/list"})),
        (
            "polter.format",
            tool_call(mcp::TOOL_POLTER_FORMAT, json!({})),
        ),
        (
            "card.context",
            tool_call(
                mcp::TOOL_POLTER_QUERY,
                json!({"project_id":seed::PROJECT_ID,"query":"brief local-build-dogfood"}),
            ),
        ),
        (
            "node.context",
            tool_call(
                mcp::TOOL_POLTER_QUERY,
                json!({"project_id":seed::PROJECT_ID,"query":"brief cli/commands"}),
            ),
        ),
        (
            "search",
            tool_call(
                mcp::TOOL_POLTER_QUERY,
                json!({"project_id":seed::PROJECT_ID,"query":"search commits"}),
            ),
        ),
        (
            "canvas.show",
            tool_call(
                mcp::TOOL_POLTER_QUERY,
                json!({"project_id":seed::PROJECT_ID,"query":"canvas Architecture"}),
            ),
        ),
    ];

    let mut measured = BTreeMap::new();
    for (name, request) in reads {
        let text = mcp_result(&store, request);
        if std::env::var_os("RUST_TEST_NOCAPTURE").is_some()
            && !["mcp.tools_list", "polter.format", "mcp.initialize"].contains(&name)
        {
            eprintln!("---- {name}\n{text}");
        }
        measured.insert(name, context::estimate_tokens(&text));
    }

    for (name, got) in measured {
        let ceiling = TOKEN_CEILINGS
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| *value)
            .unwrap_or(0);
        eprintln!("{name:<16} {got:>6} tokens (ceiling {ceiling})");
        if ceiling == 0 {
            panic!("{name} has no ceiling: add {name:?}: {got}");
        }
        assert!(
            got <= ceiling,
            "{name} grew to {got} tokens, over its ceiling of {ceiling}"
        );
        assert!(
            got * 10 >= ceiling * 9,
            "{name} fell to {got} tokens: lower its ceiling from {ceiling} to {got}"
        );
    }
}

fn seed_architecture_canvas(store: &Store) {
    let nodes = store.structure_nodes(seed::PROJECT_ID).unwrap();
    let ref_node = |path: &str| {
        let node = nodes
            .iter()
            .find(|node| node.path == path)
            .unwrap_or_else(|| panic!("no Structure node {path}"));
        Some(canvas::Ref {
            kind: canvas::REF_NODE.to_string(),
            id: node.id.clone(),
        })
    };
    let graph = canvas::Graph {
        nodes: vec![
            canvas::Node {
                id: "node-frame-core".into(),
                kind: "frame".into(),
                label: "Local core".into(),
                ..canvas::Node::default()
            },
            canvas::Node {
                id: "node-core-server".into(),
                kind: "service".into(),
                label: "Server".into(),
                r#ref: ref_node("core/server"),
                ..canvas::Node::default()
            },
            canvas::Node {
                id: "node-core-store".into(),
                kind: "database".into(),
                label: "Store".into(),
                r#ref: ref_node("core/store"),
                ..canvas::Node::default()
            },
            canvas::Node {
                id: "node-renderer".into(),
                kind: "frontend".into(),
                label: "Renderer".into(),
                props: serde_json::from_value(
                    json!({"technology":"Solid","style":{"fill":"blue"}}),
                )
                .unwrap(),
                ..canvas::Node::default()
            },
            canvas::Node {
                id: "node-sqlite".into(),
                kind: "database".into(),
                label: "SQLite file".into(),
                props: serde_json::from_value(json!({"technology":"SQLite"})).unwrap(),
                ..canvas::Node::default()
            },
            canvas::Node {
                id: "node-note-lock".into(),
                kind: canvas::KIND_NOTE.into(),
                label: "One serve process per database, guarded by a lock.".into(),
                ..canvas::Node::default()
            },
            canvas::Node {
                id: "node-blank".into(),
                kind: "rectangle".into(),
                ..canvas::Node::default()
            },
        ],
        edges: vec![
            canvas::Edge {
                id: "edge-renderer-server".into(),
                from: "node-renderer".into(),
                to: "node-core-server".into(),
                kind: canvas::EDGE_CALLS.into(),
                ..canvas::Edge::default()
            },
            canvas::Edge {
                id: "edge-server-store".into(),
                from: "node-core-server".into(),
                to: "node-core-store".into(),
                kind: canvas::EDGE_USES.into(),
                ..canvas::Edge::default()
            },
            canvas::Edge {
                id: "edge-store-sqlite".into(),
                from: "node-core-store".into(),
                to: "node-sqlite".into(),
                kind: canvas::EDGE_WRITES.into(),
                ..canvas::Edge::default()
            },
        ],
    };
    let layout = r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":{"node-frame-core":{"x":300,"y":0,"width":520,"height":400},"node-core-server":{"x":340,"y":60,"width":180,"height":100},"node-core-store":{"x":340,"y":240,"width":180,"height":100},"node-note-lock":{"x":580,"y":60,"width":200,"height":120},"node-renderer":{"x":0,"y":60,"width":180,"height":100},"node-sqlite":{"x":900,"y":240,"width":180,"height":100},"node-blank":{"x":900,"y":0,"width":120,"height":80}}}"#;
    store
        .create_canvas(
            canvas::Canvas {
                id: "canvas-architecture".into(),
                project_id: seed::PROJECT_ID.into(),
                name: "Architecture".into(),
                r#type: canvas::TYPE_CANVAS.into(),
                agent_visible: true,
                graph,
                layout_json: layout.into(),
                ..canvas::Canvas::default()
            },
            "",
            "",
        )
        .unwrap();
}

fn tool_call(name: &str, arguments: Value) -> Value {
    json!({"method":"tools/call","params":{"name":name,"arguments":arguments}})
}

fn mcp_result(store: &Store, mut request: Value) -> String {
    request["jsonrpc"] = json!("2.0");
    request["id"] = json!(1);
    let mut input = serde_json::to_vec(&request).unwrap();
    input.push(b'\n');
    let mut output = Vec::new();
    let mut server = mcp::Server::new();
    server
        .serve(store, Cursor::new(input), &mut output)
        .unwrap();
    server.close(store);
    let response: Value = serde_json::from_slice(&output).unwrap();
    assert!(response.get("error").is_none(), "{}", response["error"]);
    let result = &response["result"];
    if request["method"] != "tools/call" || result.get("content").is_none() {
        return result.to_string();
    }
    assert_ne!(result["isError"], true, "{}", result["content"]);
    result["content"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|part| part["text"].as_str())
        .map(|text| format!("{text}\n"))
        .collect()
}
