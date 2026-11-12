//! The demo database that fills every surface.
//!
//! The demo database: one project with knowledge, structure, facts, Work
//! cards, and connections that fills every surface, and one empty project that
//! shows every empty state.

mod integrations;
mod knowledge;
mod seed;
mod work;

#[cfg(test)]
mod tests;

pub use seed::*;
