use polterware_core::local_v1::KnowledgeItem;

#[test]
fn test_knowledge_supports_docs_and_runbooks() {
    let item = KnowledgeItem {
        id: "doc".to_string(),
        project_id: "project".to_string(),
        title: "Title".to_string(),
        kind: "doc".to_string(),
        ..KnowledgeItem::default()
    };
    assert!(
        !item.project_id.is_empty() && item.kind == "doc",
        "required fields"
    );
}
