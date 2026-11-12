use polterware_core::domains::context;
use polterware_core::providers::{demoseed, sqlite::Store};

#[test]
fn test_compile_context_includes_rules_with_reasons() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().join("context-compile.db")).unwrap();
    demoseed::defaults(&store).unwrap();
    store.sync_project_model(demoseed::PROJECT_ID).unwrap();

    let rule_id = "decision-principle-local-first";
    let sample = store
        .compile_context(&context::Request {
            project_id: demoseed::PROJECT_ID.to_string(),
            token_budget: 2000,
            ..context::Request::default()
        })
        .unwrap();
    assert_rule_item(&sample, rule_id, context::REASON_PROJECT_SAMPLE);

    let matched = store
        .compile_context(&context::Request {
            project_id: demoseed::PROJECT_ID.to_string(),
            query: "hosted account".to_string(),
            token_budget: 2000,
            ..context::Request::default()
        })
        .unwrap();
    assert_rule_item(&matched, rule_id, context::REASON_FACT_QUERY_MATCH);
}

fn assert_rule_item(package: &context::Package, rule_id: &str, want_reason: &str) {
    let found = package
        .items
        .iter()
        .find(|item| item.kind == "rule" && item.id == rule_id)
        .unwrap_or_else(|| {
            panic!(
                "expected rule {rule_id} in package ({} items)",
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
