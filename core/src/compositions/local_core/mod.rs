//! Port of `backend/compositions/local-core`: the core's Connect-JSON server.
//!
//! One module per Go file. [`server`] holds [`Core`], the listener, the router,
//! and the project, environment, knowledge, and connection handlers; [`connect`]
//! is the wire every handler answers on; the rest are the handler files of the
//! services, ported or still stubbed to Connect `unimplemented`.

pub mod agent_setup_handlers;
pub mod agentrun_handlers;
pub mod agents_sync;
pub mod archive_handlers;
pub mod audited_write;
pub mod canon_handlers;
pub mod canvas_handlers;
pub mod client_handlers;
pub mod commit_handlers;
pub mod connect;
pub mod deploy_checks;
pub mod dispatch_host_handlers;
pub mod extra;
pub mod feature_handlers;
pub mod inbox_handlers;
pub mod integration_sync;
pub mod lifecycle;
pub mod repos;
pub mod rule_handlers;
pub mod saved_filter_handlers;
pub mod scope_handlers;
pub mod server;
pub mod terminal;
pub mod work_handlers;

pub use server::{Core, mint_token};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_support;

/// Every route of every service in `polterware/local/v1`, in the order
/// `ServeConnect` registers them.
pub fn routes() -> Vec<connect::Route> {
    let mut routes = Vec::new();
    routes.extend(lifecycle::routes());
    routes.extend(server::routes());
    routes.extend(agent_setup_handlers::routes());
    routes.extend(agentrun_handlers::routes());
    routes.extend(archive_handlers::routes());
    routes.extend(canon_handlers::routes());
    routes.extend(canvas_handlers::routes());
    routes.extend(client_handlers::routes());
    routes.extend(commit_handlers::routes());
    routes.extend(deploy_checks::routes());
    routes.extend(dispatch_host_handlers::routes());
    routes.extend(extra::routes());
    routes.extend(feature_handlers::routes());
    routes.extend(inbox_handlers::routes());
    routes.extend(integration_sync::routes());
    routes.extend(repos::routes());
    routes.extend(rule_handlers::routes());
    routes.extend(saved_filter_handlers::routes());
    routes.extend(scope_handlers::routes());
    routes.extend(terminal::routes());
    routes.extend(work_handlers::routes());
    routes.extend(agents_sync::routes());
    routes
}
