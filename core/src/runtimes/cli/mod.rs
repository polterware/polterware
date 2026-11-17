//! The `polterware` binary.
//!
//! `main` parses `-db` and the command, then hands each command to its file:
//! `serve` (the only process that opens the store), `mcp` (the stdio bridge to
//! the core's `/mcp`), and the client commands. A client command with no
//! implementation yet answers `<command>: not ported yet` with exit code 2.

pub mod canvas;
pub mod conflict;
pub mod context;
pub mod core;
pub mod db;
pub mod doc;
pub mod export;
pub mod feature;
pub mod flags;
pub mod hook;
pub mod inbox;
pub mod init;
pub mod main;
pub use main::main;
pub mod mcpbridge;
pub mod mcpsetup;
pub mod milestone;
pub mod moved;
pub mod output;
pub mod polter;
pub mod repo;
pub mod scope;
pub mod serve;
pub mod setup;
pub mod structure;
