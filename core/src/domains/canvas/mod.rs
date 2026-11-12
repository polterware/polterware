//! The canvas model: a semantic property graph with layout stored beside it.
//!
//! Canvas is a semantic property graph. Layout (positions, sizes, groups) is
//! stored beside the graph and is never part of its semantics.

mod export;
mod model;
mod relations;
mod source;
mod types;
mod validate;

pub use export::*;
pub use model::*;
pub use relations::*;
pub use source::*;
pub use types::*;
pub use validate::*;

/// The errors the canvas returns, one per Go `errors.New`/`fmt.Errorf` site.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// Go wraps `ErrInvalidGraph` with the violations that hold it, so this
    /// variant is the only shape a caller sees.
    #[error("invalid canvas graph: {0}")]
    InvalidGraph(String),
    #[error("unknown canvas type {0:?}")]
    UnknownType(String),
    #[error("canvas name is required")]
    NameRequired,
    #[error("layout must be JSON")]
    Layout,
    #[error("unknown scope node")]
    UnknownScope,
    #[error("archive the canvas before deleting it permanently")]
    NotArchived,
    #[error("restore the canvas before editing it")]
    Archived,
    #[error("unknown canvas format {0:?}: use mermaid, json, or dot")]
    UnknownFormat(String),
    /// The source did not parse. The message names the place, so an agent can
    /// go straight to the line and column.
    #[error("line {line}, column {column}: {message}")]
    Source {
        line: u32,
        column: u32,
        message: String,
    },
    /// An `encoding/json` failure of a graph or export encode.
    #[error("{0}")]
    Json(String),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
