//! Port of `backend/providers/cli`.
//!
//! Reads platform resources through each provider's CLI and the sign-in it
//! already has: the connection probe, the bounded inventory pull, and the
//! deploy targets and their latest deploy status. Polterware never stores a
//! provider token.

mod command;
mod inventory;
mod json;
mod platforms;
mod probe;
mod probe_credentials;
mod text;

pub use inventory::{InventoryItem, InventoryResult, list_inventory};
pub use platforms::{
    DeployStatus, STATE_BUILDING, STATE_FAILED, STATE_OK, STATE_STOPPED, STATE_UNKNOWN, Target,
    TargetsResult, check_deploy, dashboard_url, has_status, list_targets,
};
pub use probe::{AuthPlan, ProbeResult, begin_auth, find_provider, probe};

/// Why a provider call failed. The text is the one the Go package answered.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The id is not in the integrations catalog.
    #[error("unknown provider")]
    UnknownProvider,
    /// The platform's CLI reports no deploy status for a target.
    #[error("this platform reports no deploy status")]
    NoStatus,
    /// Any other message, as the CLI or the catalog wrote it.
    #[error("{0}")]
    Message(String),
}

/// The outcome of one CLI call: the bytes it wrote and why it failed, the
/// shape of the Go `commandRunner`'s `([]byte, error)`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Output {
    pub(crate) out: Vec<u8>,
    pub(crate) err: Option<Error>,
}

/// Runs one CLI command and answers its output. The deadline is decided by
/// whoever builds the runner, as the Go package derived it from a context.
pub(crate) type CommandRunner = dyn Fn(&str, &[&str]) -> Output;
