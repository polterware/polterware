//! A core started for a fresh database, run only when `POLTERWARE_TEST_BIN` names a
//! `polterware` binary: `POLTERWARE_TEST_BIN=../dist/polterware cargo test --test live_core`.
//! It starts `polterware serve` the way the app's Start the core does, reads the projects
//! and each one's Inbox, and finds the same core again. The core exits on its own when idle.

use polterware_gpui::client;
use polterware_gpui::contracts::{ListInboxRequest, ListProjectsRequest, WatchChangesRequest};
use polterware_gpui::prefs::Idle;

#[test]
fn starts_and_reads_a_core() {
    let Some(binary) = std::env::var_os("POLTERWARE_TEST_BIN") else {
        eprintln!("POLTERWARE_TEST_BIN is not set; skipped");
        return;
    };
    let binary = std::fs::canonicalize(binary).expect("POLTERWARE_TEST_BIN exists");
    // SAFETY: the test sets the variable before any thread reads it.
    unsafe { std::env::set_var("POLTERWARE_BIN", &binary) };
    let dir = std::env::temp_dir().join(format!("pw-gpui-live-{}", std::process::id()));
    let db = dir.join("local.db");
    // Ten minutes idle, so the core exits on its own.
    let core = client::start(&db, Idle::TenMinutes.flag()).expect("the core starts");
    let projects = core
        .project()
        .list_projects(&ListProjectsRequest::default())
        .expect("projects")
        .projects;
    for project in &projects {
        core.work()
            .list_inbox(&ListInboxRequest {
                project_id: project.id.clone(),
                ..ListInboxRequest::default()
            })
            .expect("inbox");
    }
    // The change stream opens with the current seq and no kinds.
    let mut changes = core
        .change()
        .watch_changes(&WatchChangesRequest::default())
        .expect("the change stream opens");
    let first = changes
        .next()
        .expect("a first event")
        .expect("a readable event");
    assert!(first.kinds.is_empty(), "{first:?}");
    assert_eq!(client::discover(&db).expect("the core is found"), core);
}
