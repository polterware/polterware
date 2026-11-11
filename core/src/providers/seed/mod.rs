//! Port of `backend/providers/seed`.
//!
//! Seeds an empty database with the Polterware project itself and one empty
//! project, so Polterware is managed with its own build.

mod decisions;
mod defaults;
mod facts;
mod knowledge;
mod structure;

pub use defaults::{
    EMPTY_PROJECT_ID, Error, PROJECT_ID, PROJECT_NAME, Result, defaults, detect_root, polterware,
};
