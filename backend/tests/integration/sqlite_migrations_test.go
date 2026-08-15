package integration

import (
	dbsqlite "github.com/polterware/polterware/backend/platform/db/sqlite"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
	"path/filepath"
	"testing"
)

func TestSQLiteUsesSingleBaseline(t *testing.T) {
	store, err := storepkg.New(filepath.Join(t.TempDir(), "first-version.db"))
	if err != nil {
		t.Fatal(err)
	}
	defer store.Close()
	versions, err := dbsqlite.AppliedVersions(store.DB)
	if err != nil {
		t.Fatal(err)
	}
	if len(versions) != 1 || versions[0] != 1 {
		t.Fatalf("versions %v", versions)
	}
}
