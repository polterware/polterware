//! Seeding an empty database with the Polterware project and an empty one.
//!
//! Seeds an empty database with the Polterware project itself and one empty
//! project, so Polterware is managed with its own build.

mod defaults;
mod knowledge;
mod structure;

pub use defaults::{
    EMPTY_PROJECT_ID, Error, PROJECT_ID, PROJECT_NAME, Result, defaults, detect_root, polterware,
};
