use polterware_core::domains::{canon, context};
use polterware_core::providers::{demoseed, sqlite::Store};

#[test]
fn test_compile_context_includes_facts_with_reasons() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().join("context-compile.db")).unwrap();
    demoseed::defaults(&store).unwrap();
    store.sync_project_model(demoseed::PROJECT_ID).unwrap();

    let fact = canon::new_fact(
        demoseed::PROJECT_ID,
        "module uses Go modules for dependency management",
        canon::TRUTH_DECLARED,
        "polterware",
        "system",
        "",
    )
    .unwrap();
    store.upsert_canon_fact(&fact).unwrap();
    let sample = store
        .compile_context(&context::Request {
            project_id: demoseed::PROJECT_ID.to_string(),
            token_budget: 2000,
            ..context::Request::default()
        })
        .unwrap();
    assert_fact_item(&sample, &fact.id, context::REASON_PROJECT_SAMPLE);

    let matched = store
        .compile_context(&context::Request {
            project_id: demoseed::PROJECT_ID.to_string(),
            query: "Go modules".to_string(),
            token_budget: 2000,
            ..context::Request::default()
        })
        .unwrap();
    assert_fact_item(&matched, &fact.id, context::REASON_FACT_QUERY_MATCH);
}

fn assert_fact_item(package: &context::Package, fact_id: &str, want_reason: &str) {
    let found = package
        .items
        .iter()
        .find(|item| item.kind == "fact" && item.id == fact_id)
        .unwrap_or_else(|| {
            panic!(
                "expected fact {fact_id} in package ({} items)",
                package.items.len()
            )
        });
    assert!(!found.reasons.is_empty(), "expected non-empty reasons");
    assert!(
        found.reasons.iter().any(|reason| reason == want_reason),
        "expected reason {want_reason:?}, got {:?}",
        found.reasons
    );
}
