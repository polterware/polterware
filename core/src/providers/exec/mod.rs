//! Running a short-lived command with a timeout and a bounded stdout.
//!
//! Runs a short-lived command with a timeout and keeps a bounded stdout, so
//! every caller (Git, package managers) reports failures
//! the same way.

mod run;

pub use run::{DEFAULT_TIMEOUT, Error, MAX_STDOUT_BYTES, Result, run, run_with_timeout};
