package drift

import (
	"fmt"

	"github.com/polterware/polterware/backend/domains/ppm"
)

func checkEdgeDecisions(rels []RelationshipRef, adrs []AcceptedADR) []Finding {
	governed := map[string][]AcceptedADR{}
	for _, adr := range adrs {
		for _, entityID := range adr.EntityIDs {
			governed[entityID] = append(governed[entityID], adr)
		}
	}

	var findings []Finding
	for _, rel := range rels {
		if !isDependencyEdge(rel.Predicate) {
			continue
		}
		if rel.TruthClass != ppm.TruthInferred {
			continue
		}
		var hit []AcceptedADR
		hit = append(hit, governed[rel.SubjectEntityID]...)
		hit = append(hit, governed[rel.ObjectEntityID]...)
		if len(hit) == 0 {
			continue
		}
		adr := hit[0]
		findings = append(findings, Finding{
			Type:       TypeEdgeDecisionConflict,
			Summary:    fmt.Sprintf("Dependency edge without evidence under accepted decision %q", adr.Title),
			Severity:   SeverityConflict,
			SubjectIDs: []string{rel.ID, rel.SubjectEntityID, rel.ObjectEntityID, adr.ID},
			Details: map[string]string{
				"predicate":         rel.Predicate,
				"truth_class":       rel.TruthClass,
				"accepted_adr_id":   adr.ID,
				"accepted_adr_title": adr.Title,
			},
		})
	}
	return findings
}
