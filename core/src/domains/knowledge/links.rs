//! The entity links a knowledge item carries.

use time::OffsetDateTime;

use super::Item;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KnowledgeLinkError {
    #[error("invalid knowledge entity link")]
    InvalidLink,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityLink {
    pub id: String,
    pub project_id: String,
    pub knowledge_id: String,
    pub entity_id: String,
    pub predicate: String,
    pub truth_class: String,
    pub source: String,
    pub actor: String,
    pub observed_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl Default for EntityLink {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            knowledge_id: String::new(),
            entity_id: String::new(),
            predicate: String::new(),
            truth_class: String::new(),
            source: String::new(),
            actor: String::new(),
            observed_at: OffsetDateTime::UNIX_EPOCH,
            created_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SearchHit {
    pub item: Item,
    pub score: f64,
    pub snippet: String,
    pub truth_class: String,
    pub source: String,
}

pub fn link_id(knowledge_id: &str, entity_id: &str, predicate: &str) -> String {
    let sum = sha1(format!("{knowledge_id}|{entity_id}|{predicate}").as_bytes());
    format!("klink-{}", hex::encode(&sum[..12]))
}

pub fn new_entity_link(
    project_id: &str,
    knowledge_id: &str,
    entity_id: &str,
    predicate: &str,
    truth_class: &str,
    source: &str,
    actor: &str,
) -> Result<EntityLink, KnowledgeLinkError> {
    if project_id.is_empty()
        || knowledge_id.is_empty()
        || entity_id.is_empty()
        || predicate.is_empty()
    {
        return Err(KnowledgeLinkError::InvalidLink);
    }
    let truth_class = if truth_class.is_empty() {
        "declared"
    } else {
        truth_class
    };
    let source = if source.is_empty() {
        "polterware"
    } else {
        source
    };
    let actor = if actor.is_empty() { "human" } else { actor };
    let now = OffsetDateTime::now_utc();
    Ok(EntityLink {
        id: link_id(knowledge_id, entity_id, predicate),
        project_id: project_id.to_string(),
        knowledge_id: knowledge_id.to_string(),
        entity_id: entity_id.to_string(),
        predicate: predicate.to_string(),
        truth_class: truth_class.to_string(),
        source: source.to_string(),
        actor: actor.to_string(),
        observed_at: now,
        created_at: now,
    })
}

/// SHA-1, the digest Go's `crypto/sha1` gives `LinkID`, so the id stays byte for byte the same.
fn sha1(data: &[u8]) -> [u8; 20] {
    crate::domains::canon::sha1::digest(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_matches_the_known_digest() {
        assert_eq!(
            hex::encode(sha1(b"abc")),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(
            hex::encode(sha1(b"")),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
        assert_eq!(
            hex::encode(sha1(&[b'a'; 1000])),
            hex::encode(sha1(&vec![b'a'; 1000]))
        );
        assert_ne!(sha1(&vec![b'a'; 1000]), sha1(&vec![b'a'; 1001]));
    }

    #[test]
    fn link_ids_are_stable_and_short() {
        assert_eq!(
            link_id("know-1", "entity-1", "cites"),
            "klink-e6132c24437f54c63ca9b760"
        );
        assert_ne!(
            link_id("know-1", "entity-1", "cites"),
            link_id("know-1", "entity-1", "uses")
        );
    }

    #[test]
    fn new_entity_link_defaults() {
        let link =
            new_entity_link("p1", "know-1", "entity-1", "cites", "", "", "").expect("valid link");
        assert_eq!(link.truth_class, "declared");
        assert_eq!(link.source, "polterware");
        assert_eq!(link.actor, "human");
        assert_eq!(link.id, link_id("know-1", "entity-1", "cites"));
        assert_eq!(link.observed_at, link.created_at);
        assert_eq!(
            new_entity_link("p1", "", "entity-1", "cites", "", "", "").unwrap_err(),
            KnowledgeLinkError::InvalidLink
        );
    }
}
