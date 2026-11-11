//! The Polterware core in Rust: a port of the Go `backend` (one module per Go package,
//! same path with `-` as `_`). The wire stays the same: Connect-JSON routes, the SQLite
//! schema, the CLI text, and the MCP tools, so the Desktop and agents do not change.

#[path = "../../app/src/proto_json.rs"]
pub mod proto_json;
#[path = "../../app/src/contracts/common_v1.rs"]
pub mod common_v1;
#[path = "../../app/src/contracts/events_v1.rs"]
pub mod events_v1;
#[path = "../../app/src/contracts/local_v1.rs"]
pub mod local_v1;

pub mod platform;
pub mod domains;
pub mod providers;
pub mod compositions;
pub mod runtimes;
