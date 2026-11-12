//! The Polter format codec.
//!
//! The markdown dialect agents use to read and write Polterware Work state. It
//! is pure (no store) so the CLI, MCP, the Desktop (through the core), and
//! Cloud share the same code.
//!
//! This module holds the whole codec: decode, encode, structure encode, the
//! format spec, the graph checks, scopes, coordination, lint, hints, commit
//! references, and the planners (`Plan`, `Brief`, the feature flow, milestones,
//! rules and the retired blocks that point at them, conflicts, deploys, handoff
//! notes), which read the
//! shared types from [`model`].

mod brief;
mod commitref;
mod conflicts;
mod coord;
mod decode;
mod deploy_spec;
mod deploys;
mod encode;
mod feature;
mod format;
mod glob;
mod graph;
mod handoff;
mod hints;
mod lint;
mod milestones;
pub mod model;
mod plan;
mod rules;
mod scopes;
mod search;
mod structure_encode;
mod verify;

#[cfg(test)]
mod parity_tests;

pub use brief::*;
pub use commitref::*;
pub use conflicts::*;
pub use coord::*;
pub use decode::*;
pub use deploy_spec::*;
pub use deploys::*;
pub use encode::*;
pub use feature::*;
pub use format::*;
pub use glob::*;
pub use graph::*;
pub use handoff::*;
pub use hints::*;
pub use lint::*;
pub use milestones::*;
pub use model::*;
pub use plan::*;
pub use rules::*;
pub use scopes::*;
pub use search::*;
pub use structure_encode::*;
pub use verify::*;
