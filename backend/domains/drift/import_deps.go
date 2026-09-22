package drift

import "fmt"

func checkImportDeps(stored []FactRef, parsed []ParsedDep) []Finding {
	storedByID := make(map[string]FactRef)
	for _, fact := range stored {
		if !isImportSource(fact.Source) {
			continue
		}
		storedByID[fact.ID] = fact
	}

	parsedByID := make(map[string]ParsedDep, len(parsed))
	for _, dep := range parsed {
		parsedByID[dep.FactID] = dep
	}

	var findings []Finding
	for id, dep := range parsedByID {
		if _, ok := storedByID[id]; ok {
			continue
		}
		findings = append(findings, Finding{
			Type:     TypeDependencyAdded,
			Summary:  fmt.Sprintf("Observed dependency not yet in the project model: %s", dep.Statement),
			Severity: SeverityInfo,
			Details: map[string]string{
				"key":       dep.Key,
				"source":    dep.Source,
				"path":      dep.Path,
				"statement": dep.Statement,
			},
		})
	}
	for id, fact := range storedByID {
		if _, ok := parsedByID[id]; ok {
			continue
		}
		findings = append(findings, Finding{
			Type:        TypeDependencyRemoved,
			Summary:     fmt.Sprintf("Stored import fact missing from current sources: %s", fact.Statement),
			Severity:    SeverityWarning,
			SubjectIDs:  nonEmpty(fact.ID, fact.EntityID),
			EvidenceIDs: fact.EvidenceIDs,
			Details: map[string]string{
				"source":    fact.Source,
				"statement": fact.Statement,
			},
		})
	}
	return findings
}

func nonEmpty(ids ...string) []string {
	out := make([]string, 0, len(ids))
	for _, id := range ids {
		if id != "" {
			out = append(out, id)
		}
	}
	return out
}
