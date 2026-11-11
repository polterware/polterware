//! Port of `backend/providers/mcp`: the MCP server the core serves to coding
//! agents.
//!
//! The surface is four tools: [`TOOL_POLTER_QUERY`] reads, [`TOOL_POLTER_APPLY`]
//! and [`TOOL_DOC_UPSERT`] write in one audited change that joins the session's
//! AgentRun, and [`TOOL_POLTER_FORMAT`] prints the Polter format. A retired tool
//! name answers with its replacement.
//!
//! [`Server`] answers one JSON-RPC message at a time: [`Server::serve`] keeps the
//! line-oriented loop over stdio, and [`Sessions`] serves the streamable HTTP
//! transport inside the core. The wire is Go's `encoding/json`: struct fields in
//! declaration order, map keys sorted, and HTML escaped, so a client reads the
//! same bytes from either core.

mod docs;
mod helpers;
mod http;
mod polter;
mod server;
mod tools;
mod wire;

pub use http::{Info, Reply, SESSION_HEADER, Sessions, handle_request, method_router};
pub use server::{INSTRUCTIONS, SERVER_NAME, SERVER_VERSION, Server};
pub use tools::{TOOL_DOC_UPSERT, TOOL_POLTER_APPLY, TOOL_POLTER_FORMAT, TOOL_POLTER_QUERY};

/// A tool failure, printed as the one message the client reads.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct Error(pub String);

impl Error {
    /// Builds the error Go's `fmt.Errorf` would have written.
    pub(crate) fn message(text: impl Into<String>) -> Error {
        Error(text.into())
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
pub(crate) mod test_support;
