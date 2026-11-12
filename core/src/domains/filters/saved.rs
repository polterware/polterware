//! A saved filter's record and its stored shape.

use serde_json::Value;
use time::OffsetDateTime;

/// The longest saved filter name, counted in runes.
pub const MAX_NAME_LEN: usize = 80;
/// The longest surface key, counted in runes.
pub const MAX_SURFACE_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SavedFilterError {
    #[error("saved filter surface is required")]
    SurfaceRequired,
    #[error("saved filter surface is too long")]
    SurfaceLength,
    #[error("saved filter name is required")]
    NameRequired,
    #[error("saved filter name is too long")]
    NameLength,
    #[error("saved filter payload must be a JSON object with an integer version")]
    Payload,
    #[error("saved filter not found")]
    NotFound,
    #[error("saved filter project not found")]
    ProjectNotFound,
}

/// One named filter combination for a surface. An empty `project_id` is an
/// app-level surface such as "projects". `payload_json` is opaque JSON the UI
/// owns and must contain an integer version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedFilter {
    pub id: String,
    pub project_id: String,
    pub surface: String,
    pub name: String,
    pub payload_json: String,
    pub position: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Default for SavedFilter {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            surface: String::new(),
            name: String::new(),
            payload_json: String::new(),
            position: 0,
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

/// Trims the filter's keys and payload and validates them.
pub fn prepare_save(mut filter: SavedFilter) -> Result<SavedFilter, SavedFilterError> {
    filter.project_id = filter.project_id.trim().to_string();
    filter.surface = filter.surface.trim().to_string();
    filter.name = filter.name.trim().to_string();
    filter.payload_json = filter.payload_json.trim().to_string();
    validate_surface(&filter.surface)?;
    if filter.name.is_empty() {
        return Err(SavedFilterError::NameRequired);
    }
    if filter.name.chars().count() > MAX_NAME_LEN {
        return Err(SavedFilterError::NameLength);
    }
    validate_payload(&filter.payload_json)?;
    Ok(filter)
}

/// Reports whether a trimmed surface is non-empty and within
/// [`MAX_SURFACE_LEN`].
pub fn validate_surface(surface: &str) -> Result<(), SavedFilterError> {
    if surface.is_empty() {
        return Err(SavedFilterError::SurfaceRequired);
    }
    if surface.chars().count() > MAX_SURFACE_LEN {
        return Err(SavedFilterError::SurfaceLength);
    }
    Ok(())
}

/// Reports whether `payload` is a JSON object with an integer version of at
/// least 1. Every other field stays opaque.
pub fn validate_payload(payload: &str) -> Result<(), SavedFilterError> {
    let value: Value = serde_json::from_str(payload).map_err(|_| SavedFilterError::Payload)?;
    let Some(object) = value.as_object() else {
        return Err(SavedFilterError::Payload);
    };
    let Some(version) = object.get("version") else {
        return Err(SavedFilterError::Payload);
    };
    match version.as_i64() {
        Some(version) if version >= 1 => Ok(()),
        _ => Err(SavedFilterError::Payload),
    }
}

/// One past the highest position, or 0 when there are none.
pub fn next_position(existing: &[SavedFilter]) -> i32 {
    let mut next = 0;
    for filter in existing {
        if filter.position >= next {
            next = filter.position + 1;
        }
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter(surface: &str, name: &str, payload_json: &str) -> SavedFilter {
        SavedFilter {
            surface: surface.to_string(),
            name: name.to_string(),
            payload_json: payload_json.to_string(),
            ..SavedFilter::default()
        }
    }

    #[test]
    fn prepare_save_trims_and_validates() {
        let got = prepare_save(filter(
            " work.board ",
            " Mine ",
            r#" {"version":1,"clauses":[]} "#,
        ))
        .expect("valid filter");
        assert_eq!(got.surface, "work.board");
        assert_eq!(got.name, "Mine");
        assert_eq!(got.payload_json, r#"{"version":1,"clauses":[]}"#);

        let cases = [
            (
                filter(" ", "a", r#"{"version":1}"#),
                SavedFilterError::SurfaceRequired,
            ),
            (
                filter(&"s".repeat(MAX_SURFACE_LEN + 1), "a", r#"{"version":1}"#),
                SavedFilterError::SurfaceLength,
            ),
            (
                filter("work", "  ", r#"{"version":1}"#),
                SavedFilterError::NameRequired,
            ),
            (
                filter("work", &"n".repeat(MAX_NAME_LEN + 1), r#"{"version":1}"#),
                SavedFilterError::NameLength,
            ),
            (filter("work", "a", ""), SavedFilterError::Payload),
        ];
        for (case, want) in cases {
            assert_eq!(prepare_save(case).unwrap_err(), want);
        }
    }

    #[test]
    fn validate_payload_accepts_objects_with_an_integer_version() {
        for ok in [
            r#"{"version":1}"#,
            r#"{"version":2,"clauses":[{"field":"stage"}],"extra":{}}"#,
        ] {
            assert_eq!(validate_payload(ok), Ok(()), "payload {ok}");
        }
        for bad in [
            "{}",
            "[]",
            "null",
            r#"{"version":"1"}"#,
            r#"{"version":1.5}"#,
            r#"{"version":0}"#,
            r#"{"version":1} {}"#,
            "{",
        ] {
            assert_eq!(
                validate_payload(bad).unwrap_err(),
                SavedFilterError::Payload,
                "payload {bad}"
            );
        }
    }

    #[test]
    fn next_position_follows_the_highest_one() {
        assert_eq!(next_position(&[]), 0, "empty surface starts at 0");
        let existing = [
            SavedFilter {
                position: 2,
                ..SavedFilter::default()
            },
            SavedFilter {
                position: 0,
                ..SavedFilter::default()
            },
        ];
        assert_eq!(next_position(&existing), 3);
    }
}
