//! Port of `backend/domains/filters`.
//!
//! Saved filters: named filter combinations per project and surface (page).
//! They are UI preferences, not audited project changes.

pub mod saved;

pub use saved::*;
