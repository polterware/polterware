use polterware_core::domains::canon;
use polterware_core::providers::{demoseed, sqlite::Store};

#[test]
fn test_canon_facts_keep_their_provenance() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().join("canon-facts.db")).unwrap();
    demoseed::defaults(&store).unwrap();

    let fact = canon::new_fact(
        demoseed::PROJECT_ID,
        "polterware-core is the local authority binary",
        canon::TRUTH_DECLARED,
        "README",
        "system",
        "",
    )
    .unwrap();
    store.upsert_canon_fact(&fact).unwrap();
    let got = store.get_canon_fact(&fact.id).unwrap();
    assert_eq!(got.statement, fact.statement);
    assert_eq!(got.truth_class, canon::TRUTH_DECLARED);
    assert_eq!(got.source, "README");
    assert_eq!(
        canon::new_fact(
            demoseed::PROJECT_ID,
            "verified claim",
            canon::TRUTH_VERIFIED,
            "human",
            "system",
            "",
        ),
        Err(canon::Error::VerifiedRequiresActor)
    );
}
