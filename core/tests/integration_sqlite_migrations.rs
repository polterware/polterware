use polterware_core::platform::db::sqlite::{applied_versions, migration_versions};
use polterware_core::providers::sqlite::Store;

#[test]
fn test_sqlite_uses_single_baseline() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().join("first-version.db")).unwrap();
    let versions = applied_versions(&store.db).unwrap();
    let want = migration_versions().unwrap();
    assert!(
        !versions.is_empty() && versions == want,
        "applied {versions:?}, embedded {want:?}"
    );
}
