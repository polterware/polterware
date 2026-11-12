//! The project tree: domains, components, and where code lives.
//!
//! The project graph above Work cards: the project is the root, with domains
//! under it and components under a domain or loose. Features stay cards. Status
//! is derived when the graph is read.

mod deploy;
mod history;
mod kinds;
mod node;
mod scope;

pub use deploy::*;
pub use history::*;
pub use kinds::*;
pub use node::*;
pub use scope::*;

/// The errors Structure returns, one per Go `fmt.Errorf` site.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error(
        "unknown point {0:?}: use a change batch id, an RFC3339 time, a YYYY-MM-DD day, or a commit sha"
    )]
    UnknownPoint(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
