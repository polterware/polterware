//! A core started for a fresh database, run only when `POLTERWARE_TEST_BIN` names a
//! `polterware` binary: `POLTERWARE_TEST_BIN=../dist/polterware cargo test --test live_core`.
//! It starts `polterware serve` the way the app's Start the core does, reads the projects
//! and each one's Inbox, and finds the same core again. The core exits on its own when idle.

use polterware_gpui::client;

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
    let core = client::start(&db).expect("the core starts");
    let projects = client::list_projects(&core).expect("projects");
    for project in &projects {
        client::list_inbox(&core, &project.id).expect("inbox");
    }
    assert_eq!(client::discover(&db).expect("the core is found"), core);
}
