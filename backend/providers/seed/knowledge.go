package seed

import (
	"encoding/json"
	"time"

	"github.com/polterware/polterware/backend/domains/knowledge"
	"github.com/polterware/polterware/backend/domains/ppm"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func seedKnowledge(store *storepkg.Store) error {
	docsUpdated := time.Date(2026, 9, 17, 22, 0, 0, 0, time.UTC)
	runbookUpdated := time.Date(2026, 9, 14, 12, 0, 0, 0, time.UTC)
	items := []knowledge.Item{
		{
			ID: "doc-architecture", ProjectID: ProjectID, Title: "System architecture", Kind: "doc",
			Body: "This document describes the local project topology represented on the Architecture canvas.\n\n## Boundaries\n\nComponents, resources, and operational workflows stay scoped to this project.",
			Tags: []string{"architecture", "local"}, Summary: "How the local project components fit together.",
			Owner: "Platform", Status: "published", ExtraJSON: mustJSON(map[string]any{
				"author": "Project maintainer", "environment": "All",
				"componentIds": []string{"n-gw", "n-pg"}, "componentNames": []string{"API Gateway", "PostgreSQL"},
			}),
			CreatedAt: docsUpdated, UpdatedAt: docsUpdated,
		},
		{
			ID: "doc-database", ProjectID: ProjectID, Title: "Database conventions", Kind: "doc",
			Body: "Use expand-and-contract migrations and validate backups before maintenance.\n\n## Naming\n\n- Tables: `snake_case` plural\n- Indexes: `idx_<table>_<columns>`",
			Tags: []string{"database", "conventions"}, Summary: "Naming, migrations, and backup windows.",
			Owner: "Platform", Status: "draft", ExtraJSON: mustJSON(map[string]any{
				"author": "Project maintainer", "environment": "All",
				"componentIds": []string{"n-pg"}, "componentNames": []string{"PostgreSQL"},
			}),
			CreatedAt: docsUpdated.Add(-72 * time.Hour), UpdatedAt: docsUpdated.Add(-72 * time.Hour),
		},
		{
			ID: "rb-service", ProjectID: ProjectID, Title: "Service recovery", Kind: "runbook",
			Body: "## Objective\n\nRestore a local service after a failed health check.\n\n## Prerequisites\n\n- Confirm the affected environment\n- Open a visual terminal session\n\n## Steps\n\n1. Inspect the process record\n2. Review the restart command\n3. Record the expected result\n\n## Commands\n\n```bash\nsystemctl status example-api\nsystemctl restart example-api\n```\n\n## Rollback\n\nRestore the previous service version if the health check remains degraded.",
			Tags: []string{"api", "recovery"}, Summary: "Restore a local service after a failed health check.",
			Owner: "Platform", Status: "published", ExtraJSON: mustJSON(map[string]any{
				"service": "API", "severity": "high",
				"prerequisites": []string{"Confirm the affected environment", "Open a visual terminal session"},
				"steps": []string{"Inspect the process record", "Review the restart command", "Record the expected result"},
				"commands": []string{"systemctl status example-api", "systemctl restart example-api"},
				"rollback": "Restore the previous service version if the health check remains degraded.",
				"lastValidated": "2026-09-14T12:00:00Z", "componentId": "n-gw", "componentName": "API Gateway",
			}),
			CreatedAt: runbookUpdated, UpdatedAt: runbookUpdated,
		},
		{
			ID: "rb-database", ProjectID: ProjectID, Title: "Database failover", Kind: "runbook",
			Body: "## Objective\n\nReview a failover sequence for the primary database.\n\n## Prerequisites\n\n- Verify the standby is healthy\n- Pause local writes\n\n## Steps\n\n1. Review replication state\n2. Promote the standby\n3. Validate application health\n\n## Commands\n\n```bash\npg_isready\npsql -c 'select now();'\n```\n\n## Rollback\n\nRestore the previous primary reference if validation fails.",
			Tags: []string{"database", "failover"}, Summary: "Review a failover sequence for the primary database.",
			Owner: "Platform", Status: "published", ExtraJSON: mustJSON(map[string]any{
				"service": "Database", "severity": "critical",
				"prerequisites": []string{"Verify the standby is healthy", "Pause local writes"},
				"steps": []string{"Review replication state", "Promote the standby", "Validate application health"},
				"commands": []string{"pg_isready", "psql -c 'select now();'"},
				"rollback": "Restore the previous primary reference if validation fails.",
				"lastValidated": "2026-09-06T12:00:00Z", "componentId": "n-pg", "componentName": "PostgreSQL",
				"resourceId": "res-rds-prod", "resourceName": "prod-postgres",
			}),
			CreatedAt: runbookUpdated.Add(-8 * 24 * time.Hour), UpdatedAt: runbookUpdated.Add(-8 * 24 * time.Hour),
		},
		{
			ID: "adr-sqlite-local", ProjectID: ProjectID, Title: "ADR 0001: Local SQLite authority", Kind: "adr",
			Body: "## Status\n\nAccepted\n\n## Context\n\nThe desktop needs durable local project state without a hosted account directory.\n\n## Decision\n\nPersist project truth in a local SQLite database owned by polterware-core.\n\n## Consequences\n\n- Desktop remains useful offline.\n- Cloud sync is optional later and must not replace local ownership.",
			Tags: []string{"adr", "storage"}, Summary: "Keep local SQLite as the desktop authority for project data.",
			Owner: "Platform", Status: "accepted", ExtraJSON: mustJSON(map[string]any{
				"decisionStatus": "accepted",
			}),
			CreatedAt: docsUpdated.Add(-48 * time.Hour), UpdatedAt: docsUpdated.Add(-48 * time.Hour),
		},
		{
			ID: "constitution-project", ProjectID: ProjectID, Title: "Project constitution", Kind: "constitution",
			Body: "# Project constitution\n\n## Core principles\n\n### Local-first truth\nProject state must remain useful on one machine without requiring a hosted account.\n\n### Honest product boundaries\nDo not describe planned capabilities as shipped.\n\n### Provenance over inference\nDeclared and observed facts take precedence over inferred knowledge.\n\n## Governance\n\nChanges to durable project truth require an attributable actor and a clear reason.",
			Tags: []string{"constitution"}, Summary: "Project principles for the local desktop.",
			Owner: "Platform", Status: "active", ExtraJSON: "{}",
			CreatedAt: docsUpdated.Add(-96 * time.Hour), UpdatedAt: docsUpdated.Add(-48 * time.Hour),
		},
	}
	for _, item := range items {
		if err := store.SaveKnowledge(item); err != nil {
			return err
		}
	}
	return nil
}

func seedKnowledgeLinks(store *storepkg.Store) error {
	type linkSpec struct {
		knowledgeID string
		refKind     string
		refID       string
		predicate   string
	}
	specs := []linkSpec{
		{knowledgeID: "doc-architecture", refKind: ppm.RefArchitectureNode, refID: "n-gw", predicate: ppm.PredDocuments},
		{knowledgeID: "doc-architecture", refKind: ppm.RefArchitectureNode, refID: "n-pg", predicate: ppm.PredDocuments},
		{knowledgeID: "doc-database", refKind: ppm.RefArchitectureNode, refID: "n-pg", predicate: ppm.PredDocuments},
		{knowledgeID: "rb-service", refKind: ppm.RefArchitectureNode, refID: "n-gw", predicate: ppm.PredOperates},
		{knowledgeID: "rb-database", refKind: ppm.RefArchitectureNode, refID: "n-pg", predicate: ppm.PredOperates},
		{knowledgeID: "rb-database", refKind: ppm.RefResource, refID: "res-rds-prod", predicate: ppm.PredOperates},
		{knowledgeID: "adr-sqlite-local", refKind: ppm.RefArchitectureNode, refID: "n-pg", predicate: ppm.PredRelatedTo},
	}
	for _, spec := range specs {
		entity, err := store.GetPpmEntityByRef(ProjectID, spec.refKind, spec.refID)
		if err != nil {
			continue
		}
		link, err := knowledge.NewEntityLink(ProjectID, spec.knowledgeID, entity.ID, spec.predicate, ppm.TruthDeclared, "seed", "system")
		if err != nil {
			return err
		}
		if err := store.SaveKnowledgeLink(link); err != nil {
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
