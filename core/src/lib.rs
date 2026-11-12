//! The Polterware local core: Connect-JSON routes, SQLite, CLI, and MCP.

#[path = "../../app/src/contracts/common_v1.rs"]
pub mod common_v1;
#[path = "../../app/src/contracts/events_v1.rs"]
pub mod events_v1;
#[path = "../../app/src/contracts/local_v1.rs"]
pub mod local_v1;
#[path = "../../app/src/proto_json.rs"]
pub mod proto_json;

pub mod compositions;
pub mod domains;
pub mod platform;
pub mod providers;
pub mod runtimes;
