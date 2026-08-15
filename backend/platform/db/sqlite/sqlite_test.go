package sqlite

import "testing"

func TestOpenMigrateVersions(t *testing.T) {
	db, err := Open(":memory:")
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()
	if err := Migrate(db); err != nil {
		t.Fatal(err)
	}
	vers, err := AppliedVersions(db)
	if err != nil {
		t.Fatal(err)
	}
	if len(vers) < 8 {
		t.Fatalf("want >=8 migrations got %v", vers)
	}
	// idempotent
	if err := Migrate(db); err != nil {
		t.Fatal(err)
	}
}
