package seed

import (
	"encoding/json"
	"time"

	"github.com/polterware/polterware/backend/domains/knowledge"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

const constitutionBody = "# Project constitution\n\n## Core principles\n\n### Local-first truth\nProject state must remain useful on one machine without requiring a hosted account.\n\n### Honest product boundaries\nDo not describe planned capabilities as shipped. Post-wedge depth (Change Gates, Web App, Cloud) is direction, not behavior.\n\n### Provenance over inference\nDeclared and observed facts take precedence over inferred knowledge. Inference never silently becomes project truth.\n\n### One authority per database\nOnly the local core opens SQLite; every other surface is a client.\n\n### Opaque, shared interface\nProduct UI uses the shared design system: opaque surfaces, token colors and radii, one icon set.\n\n## Governance\n\nChanges to durable project truth require an attributable actor and a clear reason."

// docs are the repository documents that define Polterware. The store holds a
// pointer and a summary; the file stays the source.
var docs = []struct{ id, title, path, summary string }{
	{"doc-future", "Future work", "docs/future.md", "Planned direction that is not implemented: Cloud, Jev, enrichment, research."},
	{"doc-design", "Desktop design", "DESIGN.md", "The visual and interaction contract for the Desktop."},
}

func seedKnowledge(store *storepkg.Store) error {
	at := time.Now().UTC()
	items := []knowledge.Item{{
		ID: "constitution-project", ProjectID: ProjectID, Title: "Project constitution", Kind: "constitution",
		Body: constitutionBody, Tags: []string{"constitution"}, Summary: "Principles that govern Polterware.",
		Owner: "Platform", Status: "active", ExtraJSON: "{}", CreatedAt: at, UpdatedAt: at,
	}}
	for _, doc := range docs {
		items = append(items, knowledge.Item{
			ID: doc.id, ProjectID: ProjectID, Title: doc.title, Kind: "doc",
			Body:    doc.summary + "\n\nSource: " + doc.path,
			Summary: doc.summary, Tags: []string{"polterware"}, Owner: "Platform", Status: "published",
			ExtraJSON: mustJSON(map[string]string{"path": doc.path}), CreatedAt: at, UpdatedAt: at,
		})
	}
	for _, item := range items {
		if err := store.SaveKnowledge(item); err != nil {
			return err
		}
	}
	return nil
}

func mustJSON(value any) string {
	body, err := json.Marshal(value)
	if err != nil {
		return "{}"
	}
	return string(body)
}
