//! The project graph: entities, relationships, and the facts about them.
//!
//! Canon is the project's graph: entities with stable ids derived from their ref,
//! the relationships between them, and the facts that carry provenance. Every
//! value has a truth class, and the layout parser projects an architecture layout
//! into the same nodes and edges.

mod layout;
pub mod sha1;

pub use layout::*;

use time::OffsetDateTime;

pub const KIND_COMPONENT: &str = "component";

pub const REF_ARCHITECTURE_NODE: &str = "architecture_node";

pub const TRUTH_DECLARED: &str = "declared";
pub const TRUTH_OBSERVED: &str = "observed";
pub const TRUTH_OPERATIONAL: &str = "operational";
pub const TRUTH_INFERRED: &str = "inferred";

pub const PRED_DOCUMENTS: &str = "DOCUMENTS";
pub const PRED_OPERATES: &str = "OPERATES";
pub const PRED_RELATED_TO: &str = "RELATED_TO";
pub const PRED_DEFINES: &str = "DEFINES";
pub const PRED_DEPENDS_ON: &str = "DEPENDS_ON";
pub const PRED_CALLS: &str = "CALLS";

pub const SOURCE_ARCHITECTURE_LAYOUT: &str = "architecture_layout";

/// The errors canon returns, one per Go sentinel error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("invalid canon value")]
    Invalid,
    #[error("unknown canon entity")]
    UnknownEntity,
    #[error("unknown canon fact")]
    UnknownFact,
    #[error("verified facts require a non-system actor")]
    VerifiedRequiresActor,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// One canon entity: a component, projected from its ref.
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub id: String,
    pub project_id: String,
    pub kind: String,
    pub ref_kind: String,
    pub ref_id: String,
    pub title: String,
    pub truth_class: String,
    pub source: String,
    pub actor: String,
    pub observed_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            kind: String::new(),
            ref_kind: String::new(),
            ref_id: String::new(),
            title: String::new(),
            truth_class: String::new(),
            source: String::new(),
            actor: String::new(),
            observed_at: OffsetDateTime::UNIX_EPOCH,
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

/// One canon relationship: a predicate from a subject entity to an object entity.
#[derive(Debug, Clone, PartialEq)]
pub struct Relationship {
    pub id: String,
    pub project_id: String,
    pub subject_entity_id: String,
    pub predicate: String,
    pub object_entity_id: String,
    pub truth_class: String,
    pub source: String,
    pub actor: String,
    pub observed_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl Default for Relationship {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            subject_entity_id: String::new(),
            predicate: String::new(),
            object_entity_id: String::new(),
            truth_class: String::new(),
            source: String::new(),
            actor: String::new(),
            observed_at: OffsetDateTime::UNIX_EPOCH,
            created_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

/// A node of an architecture layout, as the layout declares it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LayoutNode {
    pub id: String,
    pub title: String,
}

pub fn valid_kind(kind: &str) -> bool {
    kind == KIND_COMPONENT
}

pub fn valid_ref_kind(ref_kind: &str) -> bool {
    ref_kind == REF_ARCHITECTURE_NODE
}

pub fn valid_truth_class(value: &str) -> bool {
    matches!(
        value,
        TRUTH_DECLARED | TRUTH_OBSERVED | TRUTH_OPERATIONAL | TRUTH_INFERRED
    )
}

pub fn valid_predicate(value: &str) -> bool {
    matches!(
        value,
        PRED_DOCUMENTS
            | PRED_OPERATES
            | PRED_RELATED_TO
            | PRED_DEFINES
            | PRED_DEPENDS_ON
            | PRED_CALLS
    )
}

/// The edge types checked against active decisions.
pub fn dependency_predicates() -> [&'static str; 2] {
    [PRED_DEPENDS_ON, PRED_CALLS]
}

pub fn is_dependency_predicate(value: &str) -> bool {
    value == PRED_DEPENDS_ON || value == PRED_CALLS
}

pub fn entity_id(project_id: &str, ref_kind: &str, ref_id: &str) -> String {
    let sum = sha1::digest(format!("{project_id}|{ref_kind}|{ref_id}").as_bytes());
    format!("ent-{}", hex::encode(&sum[..12]))
}

pub fn relationship_id(
    project_id: &str,
    subject_id: &str,
    predicate: &str,
    object_id: &str,
) -> String {
    let sum = sha1::digest(format!("{project_id}|{subject_id}|{predicate}|{object_id}").as_bytes());
    format!("rel-{}", hex::encode(&sum[..12]))
}

#[allow(clippy::too_many_arguments)]
pub fn new_entity(
    project_id: &str,
    kind: &str,
    ref_kind: &str,
    ref_id: &str,
    title: &str,
    truth_class: &str,
    source: &str,
    actor: &str,
) -> Result<Entity> {
    if project_id.is_empty()
        || ref_id.is_empty()
        || title.is_empty()
        || !valid_kind(kind)
        || !valid_ref_kind(ref_kind)
    {
        return Err(Error::Invalid);
    }
    let truth_class = if truth_class.is_empty() {
        TRUTH_DECLARED
    } else {
        truth_class
    };
    if !valid_truth_class(truth_class) {
        return Err(Error::Invalid);
    }
    let source = if source.is_empty() {
        "polterware"
    } else {
        source
    };
    let actor = if actor.is_empty() { "system" } else { actor };
    let now = OffsetDateTime::now_utc();
    Ok(Entity {
        id: entity_id(project_id, ref_kind, ref_id),
        project_id: project_id.to_string(),
        kind: kind.to_string(),
        ref_kind: ref_kind.to_string(),
        ref_id: ref_id.to_string(),
        title: title.to_string(),
        truth_class: truth_class.to_string(),
        source: source.to_string(),
        actor: actor.to_string(),
        observed_at: now,
        created_at: now,
        updated_at: now,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn new_relationship(
    project_id: &str,
    subject_id: &str,
    predicate: &str,
    object_id: &str,
    truth_class: &str,
    source: &str,
    actor: &str,
) -> Result<Relationship> {
    if project_id.is_empty()
        || subject_id.is_empty()
        || object_id.is_empty()
        || !valid_predicate(predicate)
    {
        return Err(Error::Invalid);
    }
    let truth_class = if truth_class.is_empty() {
        TRUTH_DECLARED
    } else {
        truth_class
    };
    if !valid_truth_class(truth_class) {
        return Err(Error::Invalid);
    }
    let source = if source.is_empty() {
        "polterware"
    } else {
        source
    };
    let actor = if actor.is_empty() { "system" } else { actor };
    let now = OffsetDateTime::now_utc();
    Ok(Relationship {
        id: relationship_id(project_id, subject_id, predicate, object_id),
        project_id: project_id.to_string(),
        subject_entity_id: subject_id.to_string(),
        predicate: predicate.to_string(),
        object_entity_id: object_id.to_string(),
        truth_class: truth_class.to_string(),
        source: source.to_string(),
        actor: actor.to_string(),
        observed_at: now,
        created_at: now,
    })
}

pub fn default_predicate_for_knowledge(kind: &str) -> &'static str {
    match kind {
        "runbook" => PRED_OPERATES,
        _ => PRED_DOCUMENTS,
    }
}

pub fn format_path(entity: &Entity) -> String {
    format!("canon/{}/{}", entity.kind, entity.ref_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_id_stable() {
        let a = entity_id("proj", REF_ARCHITECTURE_NODE, "n-gw");
        let b = entity_id("proj", REF_ARCHITECTURE_NODE, "n-gw");
        assert!(
            a == b && !a.is_empty(),
            "expected stable id, got {a:?} {b:?}"
        );
        // The id the Go core writes, so rows stay compatible.
        assert_eq!(a, "ent-9a4e55dd45f8abdd7a9fc9ca");
        assert_eq!(
            relationship_id("proj", "ent-1", PRED_DEPENDS_ON, "ent-2"),
            "rel-ebc63326192ae52f730049b9"
        );
    }

    #[test]
    fn parse_layout_nodes_legacy_and_v1() {
        let legacy = r#"{"nodes":[{"id":"n-gw","label":"API Gateway","kind":"resource"}]}"#;
        let nodes = parse_layout_nodes(legacy);
        assert!(
            nodes.len() == 1 && nodes[0].id == "n-gw" && nodes[0].title == "API Gateway",
            "legacy parse: {nodes:?}"
        );
        let v1 =
            r#"{"formatVersion":1,"nodes":[{"id":"n-pg","name":"PostgreSQL","type":"database"}]}"#;
        let nodes = parse_layout_nodes(v1);
        assert!(
            nodes.len() == 1 && nodes[0].title == "PostgreSQL",
            "v1 parse: {nodes:?}"
        );
    }

    #[test]
    fn parse_layout_edges_legacy_and_v1() {
        let legacy = r#"{"edges":[{"id":"e1","from":"n-gw","to":"n-pg","kind":"depends_on","evidence":false}]}"#;
        let edges = parse_layout_edges(legacy);
        assert!(
            edges.len() == 1 && edges[0].predicate == PRED_DEPENDS_ON && !edges[0].evidence,
            "legacy edges: {edges:?}"
        );
        let v1 = r#"{"relationships":[{"sourceNodeId":"n-gw","targetNodeId":"n-auth","type":"calls","evidence":true}]}"#;
        let edges = parse_layout_edges(v1);
        assert!(
            edges.len() == 1 && edges[0].predicate == PRED_CALLS && edges[0].evidence,
            "v1 relationships: {edges:?}"
        );
        let related = parse_layout_edges(r#"{"edges":[{"from":"a","to":"b","kind":"related"}]}"#);
        assert!(
            related.len() == 1 && related[0].predicate == PRED_RELATED_TO && related[0].evidence,
            "related default evidence: {related:?}"
        );
    }

    #[test]
    fn predicate_for_canvas_type_of_canvas_types() {
        assert_eq!(predicate_for_canvas_type("stores_in"), PRED_DEPENDS_ON);
        assert_eq!(predicate_for_canvas_type("routes_to"), PRED_CALLS);
        assert_eq!(predicate_for_canvas_type("custom"), PRED_RELATED_TO);
    }

    #[test]
    fn default_predicate_for_knowledge_kind() {
        assert_eq!(default_predicate_for_knowledge("runbook"), PRED_OPERATES);
        assert_eq!(default_predicate_for_knowledge("doc"), PRED_DOCUMENTS);
    }
}
