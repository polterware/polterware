use polterware_core::domains::context;
use polterware_core::providers::{demoseed, sqlite::Store};

#[test]
fn test_canon_knowledge_search_and_context() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().join("canon.db")).unwrap();
    demoseed::defaults(&store).unwrap();
    let hits = store
        .search_knowledge(demoseed::PROJECT_ID, "database", "", 10)
        .unwrap();
    assert!(!hits.is_empty(), "expected fts hits");
    let package = store
        .compile_context(&context::Request {
            project_id: demoseed::PROJECT_ID.to_string(),
            query: "database".to_string(),
            token_budget: 800,
            ..context::Request::default()
        })
        .unwrap();
    assert!(!package.items.is_empty(), "expected compiled context items");
}
