package integration

import (
	"path/filepath"
	"testing"

	"github.com/polterware/polterware/backend/providers/demoseed"
	sqlstore "github.com/polterware/polterware/backend/providers/sqlite"
)

func TestNoLegacyConfRequired(t *testing.T) {
	s, err := sqlstore.New(filepath.Join(t.TempDir(), "local.db"))
	if err != nil {
		t.Fatal(err)
	}
	defer s.Close()
	if err := demoseed.Defaults(s); err != nil {
		t.Fatal(err)
	}
}
