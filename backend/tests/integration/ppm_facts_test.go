package integration

import (
	"path/filepath"
	"testing"

	"github.com/polterware/polterware/backend/domains/ppm"
	"github.com/polterware/polterware/backend/providers/demoseed"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func TestPPMFactsEvidenceProvenance(t *testing.T) {
	store, err := storepkg.New(filepath.Join(t.TempDir(), "ppm-facts.db"))
	if err != nil {
		t.Fatal(err)
	}
	defer store.Close()
	if err := demoseed.Defaults(store); err != nil {
		t.Fatal(err)
	}
	if _, err := store.SyncProjectModel(demoseed.ProjectID); err != nil {
		t.Fatal(err)
	}

	fact, err := ppm.NewFact(
		demoseed.ProjectID,
		"polterware-core is the local authority binary",
		ppm.TruthObserved,
		"import:README",
		"system",
		"",
	)
	if err != nil {
		t.Fatal(err)
	}
	if err := store.UpsertPpmFact(fact); err != nil {
		t.Fatal(err)
	}

	evidence, err := ppm.NewEvidence(
		demoseed.ProjectID,
		ppm.EvidenceKindFile,
		"README describes polterware-core",
		"fs",
		"README.md",
		"system",
	)
	if err != nil {
		t.Fatal(err)
	}
	if err := store.UpsertPpmEvidence(evidence); err != nil {
		t.Fatal(err)
	}
	if err := store.LinkPpmEvidenceToFact(demoseed.ProjectID, fact.ID, evidence.ID); err != nil {
		t.Fatal(err)
	}

	got, err := store.GetPpmFact(fact.ID)
	if err != nil {
		t.Fatal(err)
	}
	if got.Statement != fact.Statement || got.TruthClass != ppm.TruthObserved || got.Source != "import:README" {
		t.Fatalf("unexpected fact: %+v", got)
	}
	linked, err := store.ListPpmEvidenceForFact(fact.ID)
	if err != nil {
		t.Fatal(err)
	}
	if len(linked) != 1 || linked[0].ID != evidence.ID || linked[0].Path != "README.md" {
		t.Fatalf("unexpected evidence: %+v", linked)
	}

	_, err = ppm.NewFact(demoseed.ProjectID, "verified claim", ppm.TruthVerified, "human", "system", "")
	if err != ppm.ErrVerifiedRequiresActor {
		t.Fatalf("expected verified actor error, got %v", err)
	}
}
