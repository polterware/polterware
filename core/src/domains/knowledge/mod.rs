//! Knowledge items (docs), their links, and the search hit over them.
//!
//! Knowledge items (docs), their entity links, and the search hit shape over
//! them. Rules are their own law and are not knowledge items.

use rand::RngCore;
use time::OffsetDateTime;

pub mod links;

pub use links::*;

/// The longest knowledge body, counted in bytes.
pub const MAX_BODY: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KnowledgeError {
    #[error("invalid knowledge item")]
    Invalid,
    #[error("archive the item before deleting it permanently")]
    NotArchived,
    #[error("restore the item before editing it")]
    Archived,
    #[error("source docs are retired: agents read repo files directly")]
    SourceRetired,
}

/// The one kind: a doc is a document that backs project context.
pub const KIND_DOC: &str = "doc";

/// The one doc type. Docs hold only what lives in Polterware (agents read
/// repository files from the checkout) and are told apart by tags.
pub const DOC_TYPE_DOC: &str = "doc";

/// Was a registered project file. Archived rows may still carry it.
const RETIRED_DOC_TYPE_SOURCE: &str = "source";

/// Was a procedure; runbooks became docs tagged runbook.
const RETIRED_DOC_TYPE_RUNBOOK: &str = "runbook";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub body: String,
    pub kind: String,
    /// `doc` for a doc. Docs have no types: tags tell them apart.
    pub doc_type: String,
    pub tags: Vec<String>,
    pub summary: String,
    pub owner: String,
    pub status: String,
    pub extra_json: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    /// The archive stamp; `None` while the item is active.
    pub archived_at: Option<OffsetDateTime>,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            title: String::new(),
            body: String::new(),
            kind: String::new(),
            doc_type: String::new(),
            tags: Vec::new(),
            summary: String::new(),
            owner: String::new(),
            status: String::new(),
            extra_json: String::new(),
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
            archived_at: None,
        }
    }
}

impl Item {
    /// Reports whether the item is in the archive.
    pub fn archived(&self) -> bool {
        self.archived_at.is_some()
    }
}

pub fn new_item_id() -> String {
    let mut buf = [0u8; 12];
    rand::rng().fill_bytes(&mut buf);
    format!("know-{}", hex::encode(buf))
}

/// Refuses the retired source type; any other value is read by
/// [`doc_type_for`].
pub fn check_doc_type(doc_type: &str) -> Result<(), KnowledgeError> {
    if doc_type.trim() == RETIRED_DOC_TYPE_SOURCE {
        return Err(KnowledgeError::SourceRetired);
    }
    Ok(())
}

pub fn valid_kind(kind: &str) -> bool {
    kind == KIND_DOC
}

/// Reads a kind: doc and the retired runbook name a doc, and any other value is
/// kept as given.
pub fn normalize_kind(kind: &str) -> &str {
    match kind {
        DOC_TYPE_DOC | RETIRED_DOC_TYPE_RUNBOOK => KIND_DOC,
        _ => kind,
    }
}

/// Returns the stored doc type of `kind`: `doc` for a doc, empty otherwise.
pub fn doc_type_for(kind: &str) -> &'static str {
    if kind != KIND_DOC {
        return "";
    }
    DOC_TYPE_DOC
}

pub fn new_item(
    id: &str,
    project_id: &str,
    title: &str,
    body: &str,
    kind: &str,
    tags: Vec<String>,
) -> Result<Item, KnowledgeError> {
    check_doc_type(kind)?;
    let kind = normalize_kind(kind);
    if id.is_empty()
        || project_id.is_empty()
        || title.is_empty()
        || !valid_kind(kind)
        || body.len() > MAX_BODY
    {
        return Err(KnowledgeError::Invalid);
    }
    let now = OffsetDateTime::now_utc();
    Ok(Item {
        id: id.to_string(),
        project_id: project_id.to_string(),
        title: title.to_string(),
        body: body.to_string(),
        kind: kind.to_string(),
        doc_type: doc_type_for(kind).to_string(),
        tags,
        extra_json: "{}".to_string(),
        created_at: now,
        updated_at: now,
        ..Item::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knowledge_kinds() {
        for kind in ["evidence", "verdict", "adr"] {
            // legacy kinds
            assert!(
                new_item(
                    &format!("{kind}-1"),
                    "project",
                    "Title",
                    "Body",
                    kind,
                    vec![]
                )
                .is_err(),
                "expected kind {kind} to be rejected"
            );
        }
        // A doc type names a doc of that type; tags stay as given.
        // A runbook is a doc now; tags tell docs apart.
        for kind in ["doc", "runbook"] {
            let item = new_item(
                &format!("{kind}-1"),
                "project",
                "Title",
                "Body",
                kind,
                vec!["ops".to_string()],
            )
            .unwrap_or_else(|err| panic!("expected doc type {kind} to be valid: {err}"));
            assert_eq!(item.kind, KIND_DOC, "doc type {kind} read as {item:?}");
            assert_eq!(item.doc_type, DOC_TYPE_DOC);
            assert_eq!(item.tags, vec!["ops".to_string()]);
        }
        // Source docs are retired: agents read repository files directly.
        assert_eq!(
            new_item("source-1", "project", "Title", "", "source", vec![]).unwrap_err(),
            KnowledgeError::SourceRetired
        );
        // The five-document Work model moved to the Work tables (Decision 0006).
        for kind in [
            "spec",
            "plan",
            "tasks",
            "clarify",
            "checklist",
            "constitution",
        ] {
            assert!(
                new_item(
                    &format!("{kind}-1"),
                    "project",
                    "Title",
                    "Body",
                    kind,
                    vec![]
                )
                .is_err(),
                "expected kind {kind} to be rejected"
            );
        }
    }

    #[test]
    fn item_ids_are_prefixed_random_hex() {
        let id = new_item_id();
        assert!(id.starts_with("know-"), "id {id}");
        assert_eq!(id.len(), "know-".len() + 24, "id {id}");
        assert_ne!(id, new_item_id());
    }
}
