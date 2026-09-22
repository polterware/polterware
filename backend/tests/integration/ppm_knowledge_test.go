package integration

import (
	"path/filepath"
	"testing"

	contextpkg "github.com/polterware/polterware/backend/domains/context"
	"github.com/polterware/polterware/backend/providers/seed"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func TestPPMKnowledgeSearchAndContext(t *testing.T) {
	store, err := storepkg.New(filepath.Join(t.TempDir(), "ppm.db"))
	if err != nil {
		t.Fatal(err)
	}
	defer store.Close()
	if err := seed.Defaults(store); err != nil {
		t.Fatal(err)
	}
	count, err := store.SyncProjectModel(seed.ProjectID)
	if err != nil {
		t.Fatal(err)
	}
	if count == 0 {
		t.Fatal("expected entities")
	}
	hits, err := store.SearchKnowledge(seed.ProjectID, "architecture", "", 10)
	if err != nil {
		t.Fatal(err)
	}
	if len(hits) == 0 {
		t.Fatal("expected fts hits")
	}
	links, err := store.ListKnowledgeLinks(seed.ProjectID, "doc-architecture", "")
	if err != nil {
		t.Fatal(err)
	}
	if len(links) == 0 {
		t.Fatal("expected knowledge links")
	}
	pkg, err := store.CompileContext(contextpkg.Request{
		ProjectID:   seed.ProjectID,
		Query:       "architecture",
		TokenBudget: 800,
	})
	if err != nil {
		t.Fatal(err)
	}
	if len(pkg.Items) == 0 {
		t.Fatal("expected compiled context items")
	}
}
