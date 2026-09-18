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
	if len(vers) != 1 || vers[0] != 1 {
		t.Fatalf("want [1] got %v", vers)
	}
	// idempotent
	if err := Migrate(db); err != nil {
		t.Fatal(err)
	}
}
