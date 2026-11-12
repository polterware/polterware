//! Validation of a canvas graph against the kinds it may use.

use super::{
    Canvas, Definition, Error, Graph, KIND_NOTE, KIND_TEXT, Ref, Result, SHAPE_KINDS,
    STRUCTURE_KINDS, TYPE_CANVAS, lookup, normalize,
};

/// One reason a graph does not match its canvas type.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Violation {
    pub r#where: String,
    pub message: String,
}

/// Joins violations as "where: message" clauses.
pub fn format_violations(list: &[Violation]) -> String {
    let parts: Vec<String> = list
        .iter()
        .map(|item| {
            if item.r#where.is_empty() {
                item.message.clone()
            } else {
                format!("{}: {}", item.r#where, item.message)
            }
        })
        .collect();
    parts.join("; ")
}

/// Checks the graph against def. Only structure kinds (domain, component) must
/// ref a structure node; components and freeform nodes may omit a ref.
/// Unrestricted skips kind checks. The result is empty when the graph is valid.
/// Layout is not an input.
pub fn validate(def: &Definition, graph: &Graph) -> Vec<Violation> {
    let mut found: Vec<Violation> = Vec::new();
    if def.r#type.is_empty() {
        found.push(Violation {
            r#where: "type".to_string(),
            message: "canvas type is required".to_string(),
        });
        return found;
    }
    let mut add = |r#where: String, message: String| {
        found.push(Violation { r#where, message });
    };

    let mut ids: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (i, node) in graph.nodes.iter().enumerate() {
        let at = format!("nodes[{i}]");
        if node.id.is_empty() {
            add(at.clone(), "id is required".to_string());
        } else if !ids.insert(node.id.as_str()) {
            add(at.clone(), format!("duplicate id {:?}", node.id));
        }
        if node.kind.is_empty() {
            add(at.clone(), "kind is required".to_string());
        } else if !node_kind_allowed(def, &node.kind) {
            add(
                at.clone(),
                format!(
                    "kind {:?} is not allowed on a {} canvas",
                    node.kind, def.r#type
                ),
            );
        }
        if let Some(reference) = &node.r#ref {
            let ref_where = at.clone() + ".ref";
            if !super::valid_ref_kind(&reference.kind) {
                add(
                    ref_where.clone(),
                    format!(
                        "kind {:?} is not a card, decision, fact, or node",
                        reference.kind
                    ),
                );
            }
            if reference.id.is_empty() {
                add(ref_where, "id is required".to_string());
            }
        }
        if contains(&STRUCTURE_KINDS, &node.kind) && !structure_ref(node.r#ref.as_ref()) {
            add(at, "requires a ref to a structure node".to_string());
        }
    }

    let mut edge_ids: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (i, edge) in graph.edges.iter().enumerate() {
        let at = format!("edges[{i}]");
        if edge.id.is_empty() {
            add(at.clone(), "id is required".to_string());
        } else if !edge_ids.insert(edge.id.as_str()) {
            add(at.clone(), format!("duplicate id {:?}", edge.id));
        }
        if edge.kind.is_empty() {
            add(at.clone(), "kind is required".to_string());
        } else if !edge_kind_allowed(def, &edge.kind) {
            add(
                at.clone(),
                format!(
                    "kind {:?} is not allowed on a {} canvas",
                    edge.kind, def.r#type
                ),
            );
        }
        if edge.from.is_empty() {
            add(at.clone(), "from is required".to_string());
        } else if !ids.contains(edge.from.as_str()) {
            add(
                at.clone(),
                format!("from {:?} does not match a node", edge.from),
            );
        }
        if edge.to.is_empty() {
            add(at.clone(), "to is required".to_string());
        } else if !ids.contains(edge.to.as_str()) {
            add(at, format!("to {:?} does not match a node", edge.to));
        }
    }
    found
}

/// Trims identity fields, defaults an empty layout to "{}", and validates the
/// graph against the canvas type. Layout is checked only as JSON.
///
/// The source is the canvas: a write that sends Mermaid text has its graph read
/// from it, and a write that sends only the graph gets the canonical source the
/// text would have, so the two never disagree.
pub fn prepare(mut item: Canvas) -> Result<Canvas> {
    item.name = item.name.trim().to_string();
    item.r#type = item.r#type.trim().to_string();
    if item.r#type.is_empty() {
        item.r#type = TYPE_CANVAS.to_string();
    }
    item.scope_node_id = item.scope_node_id.trim().to_string();
    item.id = item.id.trim().to_string();
    item.project_id = item.project_id.trim().to_string();
    if item.name.is_empty() {
        return Err(Error::NameRequired);
    }
    let def = lookup(&item.r#type).ok_or_else(|| Error::UnknownType(item.r#type.clone()))?;
    let mut layout = item.layout_json.trim().to_string();
    if layout.is_empty() {
        layout = "{}".to_string();
    }
    if serde_json::from_str::<serde_json::Value>(&layout).is_err() {
        return Err(Error::Layout);
    }
    item.layout_json = layout;
    let source = item.source.trim().to_string();
    if source.is_empty() {
        item.source = super::export_mermaid(&item.graph, &item.layout_json)
            .trim_end()
            .to_string();
    } else if source == super::export_mermaid(&item.graph, &item.layout_json).trim_end() {
        // The canonical source of the graph already at hand: reading it back
        // would drop only what the text cannot carry (the editor's `style` prop
        // and edge props), so a write that sends the graph keeps them and
        // `prepare` stays idempotent.
        item.source = source;
    } else {
        item.graph = super::graph_from_source(&source)?;
        item.source = source;
    }
    normalize(&mut item.graph);
    let violations = validate(&def, &item.graph);
    if !violations.is_empty() {
        return Err(Error::InvalidGraph(format_violations(&violations)));
    }
    Ok(item)
}

fn is_freeform(kind: &str) -> bool {
    kind == KIND_NOTE || kind == KIND_TEXT || contains(&SHAPE_KINDS, kind)
}

fn node_kind_allowed(def: &Definition, kind: &str) -> bool {
    if def.unrestricted || is_freeform(kind) {
        return true;
    }
    contains(&def.node_kinds, kind)
}

fn edge_kind_allowed(def: &Definition, kind: &str) -> bool {
    if def.unrestricted {
        return true;
    }
    contains(&def.edge_kinds, kind)
}

fn contains<S: AsRef<str>>(list: &[S], value: &str) -> bool {
    list.iter().any(|item| item.as_ref() == value)
}

fn structure_ref(reference: Option<&Ref>) -> bool {
    matches!(reference, Some(reference) if reference.kind == super::REF_NODE && !reference.id.is_empty())
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value};

    use super::*;
    use crate::domains::canvas::{
        EDGE_CALLS, EDGE_CONTAINS, EDGE_READS, EDGE_RELATES, Edge, KIND_COMPONENT, KIND_DOMAIN,
        Node, REF_CARD, REF_NODE, TYPE_CANVAS, lookup, types,
    };

    fn def(typ: &str) -> Definition {
        lookup(typ).unwrap_or_else(|| panic!("missing type {typ}"))
    }

    fn violations(typ: &str, graph: &Graph) -> String {
        format_violations(&validate(&def(typ), graph))
    }

    fn must_valid(typ: &str, graph: &Graph) {
        let got = violations(typ, graph);
        assert_eq!(got, "", "{typ} rejected a valid graph: {got}");
    }

    fn must_reject(typ: &str, graph: &Graph, fragment: &str) {
        let got = violations(typ, graph);
        assert!(
            !got.is_empty(),
            "{typ} accepted graph, want a violation containing {fragment:?}"
        );
        assert!(
            got.contains(fragment),
            "{typ} violation {got:?}, want fragment {fragment:?}"
        );
    }

    fn node(id: &str, kind: &str) -> Node {
        Node {
            id: id.to_string(),
            kind: kind.to_string(),
            ..Node::default()
        }
    }

    fn with_label(mut item: Node, label: &str) -> Node {
        item.label = label.to_string();
        item
    }

    fn with_ref(mut item: Node, kind: &str, id: &str) -> Node {
        item.r#ref = Some(Ref {
            kind: kind.to_string(),
            id: id.to_string(),
        });
        item
    }

    fn with_tech(mut item: Node, tech: &str) -> Node {
        let mut props: Map<String, Value> = Map::new();
        props.insert("technology".to_string(), Value::String(tech.to_string()));
        item.props = props;
        item
    }

    fn edge(id: &str, from: &str, to: &str, kind: &str) -> Edge {
        Edge {
            id: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            kind: kind.to_string(),
            ..Edge::default()
        }
    }

    #[test]
    fn test_registry_has_one_generalist_type() {
        let types = types();
        assert_eq!(types.len(), 1, "types = {types:?}");
        assert_eq!(types[0].r#type, TYPE_CANVAS);
        assert!(!types[0].unrestricted);
        for kind in [
            "database",
            "frontend",
            "llm",
            "agent",
            "secrets",
            "pipeline",
            "dataset",
            KIND_DOMAIN,
            "cylinder",
            "frame",
            KIND_NOTE,
            KIND_TEXT,
        ] {
            assert!(
                contains(&types[0].node_kinds, kind),
                "palette missing node {kind}: {:?}",
                types[0].node_kinds
            );
        }
        for kind in [EDGE_RELATES, EDGE_CALLS, EDGE_CONTAINS] {
            assert!(
                contains(&types[0].edge_kinds, kind),
                "palette missing edge {kind}: {:?}",
                types[0].edge_kinds
            );
        }
    }

    #[test]
    fn test_validate_canvas() {
        must_valid(
            TYPE_CANVAS,
            &Graph {
                nodes: vec![
                    with_ref(
                        with_label(node("s", KIND_DOMAIN), "App"),
                        REF_NODE,
                        "structure-1",
                    ),
                    with_tech(with_label(node("web", "frontend"), "Web"), "React"),
                    with_ref(
                        with_tech(with_label(node("db", "database"), "Main"), "Postgres"),
                        REF_NODE,
                        "structure-2",
                    ),
                    with_label(node("r", "rectangle"), "box"),
                    with_label(node("n", KIND_NOTE), "later"),
                    with_label(node("t", KIND_TEXT), "body"),
                ],
                edges: vec![
                    edge("e1", "web", "db", EDGE_READS),
                    edge("e2", "s", "web", EDGE_CONTAINS),
                ],
            },
        );
        must_reject(
            TYPE_CANVAS,
            &Graph {
                nodes: vec![node("s", KIND_DOMAIN)],
                ..Graph::default()
            },
            "structure node",
        );
        must_reject(
            TYPE_CANVAS,
            &Graph {
                nodes: vec![with_ref(node("s", KIND_DOMAIN), REF_CARD, "card-1")],
                ..Graph::default()
            },
            "structure node",
        );
        must_reject(
            TYPE_CANVAS,
            &Graph {
                nodes: vec![node("x", "spaceship")],
                ..Graph::default()
            },
            "not allowed",
        );
        must_reject(
            TYPE_CANVAS,
            &Graph {
                nodes: vec![node("a", "api")],
                edges: vec![edge("e", "a", "a", "scribble")],
            },
            "not allowed",
        );
        must_reject(
            TYPE_CANVAS,
            &Graph {
                nodes: vec![node("a", "api"), node("a", "api")],
                ..Graph::default()
            },
            "duplicate id",
        );
        must_reject(
            TYPE_CANVAS,
            &Graph {
                nodes: vec![node("a", "api")],
                edges: vec![edge("e", "missing", "a", EDGE_CALLS)],
            },
            "does not match a node",
        );
        must_reject(
            TYPE_CANVAS,
            &Graph {
                nodes: vec![with_ref(node("a", "api"), "widget", "w")],
                ..Graph::default()
            },
            "not a card",
        );
    }

    #[test]
    fn test_prepare_defaults_type_and_rejects_name_and_layout() {
        assert_eq!(prepare(Canvas::default()).unwrap_err(), Error::NameRequired);
        let err = prepare(Canvas {
            name: "Map".to_string(),
            r#type: "poster".to_string(),
            ..Canvas::default()
        })
        .unwrap_err();
        assert!(err.to_string().contains("poster"), "type: {err}");
        assert_eq!(
            prepare(Canvas {
                name: "Map".to_string(),
                layout_json: "{".to_string(),
                ..Canvas::default()
            })
            .unwrap_err(),
            Error::Layout
        );
        let item = prepare(Canvas {
            name: " Map ".to_string(),
            layout_json: "  ".to_string(),
            ..Canvas::default()
        })
        .expect("prepare");
        assert_eq!(item.name, "Map");
        assert_eq!(item.r#type, TYPE_CANVAS);
        assert_eq!(item.layout_json, "{}");
    }

    #[test]
    fn prepare_keeps_what_the_source_cannot_carry_when_the_graph_is_sent() {
        let mut props: Map<String, Value> = Map::new();
        props.insert("style".to_string(), serde_json::json!({"fill": "red"}));
        let mut edge_props: Map<String, Value> = Map::new();
        edge_props.insert("sync".to_string(), Value::Bool(true));
        let once = prepare(Canvas {
            name: "Map".to_string(),
            graph: Graph {
                nodes: vec![
                    Node {
                        props,
                        ..with_label(node("a", "service"), "API")
                    },
                    with_label(node("b", "database"), "Postgres"),
                ],
                edges: vec![Edge {
                    props: edge_props,
                    ..edge("e", "a", "b", EDGE_READS)
                }],
            },
            ..Canvas::default()
        })
        .expect("prepare");
        assert!(
            once.source.contains("flowchart LR"),
            "no source: {}",
            once.source
        );
        // The store prepares again on its way in; the graph the person sent is
        // still the one stored, style and edge props included.
        let twice = prepare(once.clone()).expect("prepare again");
        assert_eq!(twice.source, once.source);
        assert_eq!(twice.graph, once.graph);
        assert_eq!(
            twice.graph.nodes[0].props.get("style"),
            Some(&serde_json::json!({"fill": "red"}))
        );
        assert_eq!(
            twice.graph.edges[0].props.get("sync"),
            Some(&Value::Bool(true))
        );
        // A source that is not the graph's own is the canvas, and it wins.
        let edited = prepare(Canvas {
            source: "flowchart LR\n  a@{ label: \"One\", kind: service }\n".to_string(),
            ..once
        })
        .expect("source write");
        assert_eq!(edited.graph.nodes.len(), 1);
        assert_eq!(edited.graph.nodes[0].label, "One");
    }

    #[test]
    fn prepare_reports_every_violation() {
        let err = prepare(Canvas {
            name: "Map".to_string(),
            graph: Graph {
                nodes: vec![node("s", KIND_COMPONENT)],
                ..Graph::default()
            },
            ..Canvas::default()
        })
        .unwrap_err();
        assert_eq!(
            err,
            Error::InvalidGraph("nodes[0]: requires a ref to a structure node".to_string())
        );
    }
}
