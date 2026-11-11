//! Port of `backend/providers/pkg`.
//!
//! Runs a package manager's script through the exec port.

mod script;

pub use script::run_script;
