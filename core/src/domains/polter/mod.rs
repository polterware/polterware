//! Port of `backend/domains/polter`: the Polter format codec.
//!
//! The markdown dialect agents use to read and write Polterware Work state. It
//! is pure (no store) so the CLI, MCP, the Desktop (through the core), and
//! Cloud share the same code.
//!
//! This module ports the format foundation: decode, encode, structure encode,
//! the format spec, the graph checks, scopes, coordination, lint, hints, and
//! commit references. The planners (`Plan`, `Brief`, the feature flow,
//! milestones, decisions, rules, conflicts, facts, deploys, handoff notes) are
//! added by later submodules and read the shared types from [`model`].

mod commitref;
mod coord;
mod decode;
mod deploy_spec;
mod encode;
mod format;
mod glob;
mod graph;
mod hints;
mod lint;
pub mod model;
mod scopes;
mod structure_encode;

pub use commitref::*;
pub use coord::*;
pub use decode::*;
pub use deploy_spec::*;
pub use encode::*;
pub use format::*;
pub use glob::*;
pub use graph::*;
pub use hints::*;
pub use lint::*;
pub use model::*;
pub use scopes::*;
pub use structure_encode::*;
