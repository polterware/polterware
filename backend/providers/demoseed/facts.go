package demoseed

import (
	"time"

	"github.com/polterware/polterware/backend/domains/ppm"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

// seedFacts records facts with evidence, plus two with none, which the Inbox
// lists as unbacked.
func seedFacts(store *storepkg.Store) error {
	observed := time.Date(2026, 9, 18, 10, 30, 0, 0, time.UTC)
	evidence := []ppm.Evidence{
		{ID: "ev-package-json", Kind: ppm.EvidenceKindFile, Summary: "Workspace manifest declares pnpm and Electron.", Source: "import", Path: "package.json"},
		{ID: "ev-go-mod", Kind: ppm.EvidenceKindFile, Summary: "Go module requires modernc.org/sqlite.", Source: "import", Path: "go.mod"},
		{ID: "ev-commit-core", Kind: ppm.EvidenceKindCommit, Summary: "a1f3c9e Add a single local core with live sync.", Source: "git"},
		{ID: "ev-test-drift", Kind: ppm.EvidenceKindTest, Summary: "Drift check suite passed on the last run.", Source: "go test", Path: "backend/domains/drift/check_test.go"},
	}
	for _, item := range evidence {
		item.ProjectID, item.Actor, item.ObservedAt, item.CreatedAt, item.UpdatedAt = ProjectID, "system", observed, observed, observed
		if err := store.UpsertPpmEvidence(item); err != nil {
			return err
		}
	}
	facts := []struct {
		fact     ppm.Fact
		evidence []string
	}{
		{ppm.Fact{ID: "fact-pnpm", Statement: "The workspace uses pnpm.", TruthClass: ppm.TruthObserved, Source: "package.json"}, []string{"ev-package-json"}},
		{ppm.Fact{ID: "fact-electron", Statement: "The Desktop is an Electron app.", TruthClass: ppm.TruthObserved, Source: "package.json"}, []string{"ev-package-json"}},
		{ppm.Fact{ID: "fact-sqlite", Statement: "Project state lives in a local SQLite database.", TruthClass: ppm.TruthObserved, Source: "go.mod"}, []string{"ev-go-mod", "ev-commit-core"}},
		{ppm.Fact{ID: "fact-drift-check", Statement: "Drift findings are computed from declared facts.", TruthClass: ppm.TruthVerified, Source: "go test", Actor: "claude-code"}, []string{"ev-test-drift"}},
		{ppm.Fact{ID: "fact-no-network", Statement: "Imports never call the network.", TruthClass: ppm.TruthDeclared, Source: "ADR 0002"}, nil},
		{ppm.Fact{ID: "fact-redis-sessions", Statement: "Sessions are cached in Redis.", TruthClass: ppm.TruthDeclared, Source: "doc-architecture"}, nil},
		{ppm.Fact{ID: "fact-webhook-queue", Statement: "Webhook retries will probably run through the event queue.", TruthClass: ppm.TruthInferred, Source: "billing-webhooks"}, nil},
	}
	for i, item := range facts {
		fact := item.fact
		at := observed.Add(time.Duration(i) * time.Minute)
		fact.ProjectID, fact.ObservedAt, fact.CreatedAt, fact.UpdatedAt = ProjectID, at, at, at
		if fact.Actor == "" {
			fact.Actor = "system"
		}
		if err := store.UpsertPpmFact(fact); err != nil {
			return err
		}
		for _, evidenceID := range item.evidence {
			if err := store.LinkPpmEvidenceToFact(ProjectID, fact.ID, evidenceID); err != nil {
				return err
			}
		}
	}
	return nil
}
