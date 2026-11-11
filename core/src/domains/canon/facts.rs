//! Port of `backend/domains/canon/facts.go`.

use time::OffsetDateTime;

use super::{Error, Result, TRUTH_DECLARED, TRUTH_INFERRED};

pub const TRUTH_VERIFIED: &str = "verified";

/// A project claim with provenance. Truth classes exclude operational (that
/// class remains on entities only).
#[derive(Debug, Clone, PartialEq)]
pub struct Fact {
    pub id: String,
    pub project_id: String,
    pub statement: String,
    pub truth_class: String,
    pub source: String,
    pub actor: String,
    pub observed_at: OffsetDateTime,
    pub entity_id: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Default for Fact {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            statement: String::new(),
            truth_class: String::new(),
            source: String::new(),
            actor: String::new(),
            observed_at: OffsetDateTime::UNIX_EPOCH,
            entity_id: String::new(),
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

pub fn new_fact_id() -> String {
    let buf: [u8; 12] = rand::random();
    format!("fact-{}", hex::encode(buf))
}

pub fn valid_fact_truth_class(value: &str) -> bool {
    matches!(value, TRUTH_DECLARED | TRUTH_INFERRED | TRUTH_VERIFIED)
}

pub fn new_fact(
    project_id: &str,
    statement: &str,
    truth_class: &str,
    source: &str,
    actor: &str,
    entity_id: &str,
) -> Result<Fact> {
    if project_id.is_empty() || statement.is_empty() {
        return Err(Error::Invalid);
    }
    let truth_class = if truth_class.is_empty() {
        TRUTH_DECLARED
    } else {
        truth_class
    };
    if !valid_fact_truth_class(truth_class) {
        return Err(Error::Invalid);
    }
    let source = if source.is_empty() {
        "polterware"
    } else {
        source
    };
    let actor = if actor.is_empty() { "system" } else { actor };
    if truth_class == TRUTH_VERIFIED && actor == "system" {
        return Err(Error::VerifiedRequiresActor);
    }
    let now = OffsetDateTime::now_utc();
    Ok(Fact {
        id: new_fact_id(),
        project_id: project_id.to_string(),
        statement: statement.to_string(),
        truth_class: truth_class.to_string(),
        source: source.to_string(),
        actor: actor.to_string(),
        observed_at: now,
        entity_id: entity_id.to_string(),
        created_at: now,
        updated_at: now,
    })
}

/// Checks an existing or client-supplied fact before upsert.
pub fn validate_fact(fact: &Fact) -> Result<()> {
    if fact.id.is_empty() || fact.project_id.is_empty() || fact.statement.is_empty() {
        return Err(Error::Invalid);
    }
    if !valid_fact_truth_class(&fact.truth_class) {
        return Err(Error::Invalid);
    }
    if fact.truth_class == TRUTH_VERIFIED && fact.actor == "system" {
        return Err(Error::VerifiedRequiresActor);
    }
    if fact.source.is_empty() || fact.actor.is_empty() {
        return Err(Error::Invalid);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::canon::TRUTH_OBSERVED;

    #[test]
    fn new_fact_declared_and_no_observed() {
        let fact = new_fact(
            "proj",
            "go.mod declares module polterware",
            "",
            "polterware",
            "system",
            "",
        )
        .unwrap();
        assert!(
            !fact.id.is_empty() && fact.truth_class == TRUTH_DECLARED,
            "{fact:?}"
        );
        validate_fact(&fact).unwrap();
        assert_eq!(
            new_fact(
                "proj",
                "seen in a file",
                TRUTH_OBSERVED,
                "import:go.mod",
                "system",
                ""
            ),
            Err(Error::Invalid),
            "observed facts are retired"
        );
    }

    #[test]
    fn new_fact_verified_requires_actor() {
        let err = new_fact(
            "proj",
            "API gateway owns auth",
            TRUTH_VERIFIED,
            "human",
            "system",
            "",
        )
        .unwrap_err();
        assert_eq!(err, Error::VerifiedRequiresActor);
        let fact = new_fact(
            "proj",
            "API gateway owns auth",
            TRUTH_VERIFIED,
            "human",
            "erick",
            "ent-1",
        )
        .unwrap();
        assert!(
            fact.actor == "erick" && fact.entity_id == "ent-1",
            "{fact:?}"
        );
    }

    #[test]
    fn fact_rejects_operational() {
        let err = new_fact(
            "proj",
            "run succeeded",
            crate::domains::canon::TRUTH_OPERATIONAL,
            "ops",
            "system",
            "",
        )
        .unwrap_err();
        assert_eq!(err, Error::Invalid, "operational must be invalid on facts");
    }
}
