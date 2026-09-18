package integration

import (
	"path/filepath"
	"testing"

	sqlstore "github.com/polterware/polterware/backend/providers/sqlite"
	"github.com/polterware/polterware/backend/providers/seed"
)

func TestNoLegacyConfRequired(t *testing.T) {
	s, err := sqlstore.New(filepath.Join(t.TempDir(), "local.db"))
	if err != nil {
		t.Fatal(err)
	}
	defer s.Close()
	if err := seed.Defaults(s); err != nil {
		t.Fatal(err)
	}
}
