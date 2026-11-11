//! Port of `backend/providers/terminal`.

mod manager;

pub use manager::{CLOSED_PIPE_MESSAGE, Error, Manager, Session, SshProfile, build_ssh_command};

#[cfg(test)]
mod tests;
