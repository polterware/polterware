use polterware_core::providers::{demoseed, sqlite::Store};

#[test]
fn test_no_legacy_conf_required() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().join("local.db")).unwrap();
    demoseed::defaults(&store).unwrap();
}
