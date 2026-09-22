package importer

import (
	"fmt"
	"time"

	"github.com/polterware/polterware/backend/domains/ppm"
)

// Materialize converts a FileResult into domain facts and evidence with stable IDs.
func Materialize(projectID string, file FileResult, now time.Time) (ppm.Evidence, []ppm.Fact, error) {
	if file.Skipped {
		return ppm.Evidence{}, nil, fmt.Errorf("skipped: %s", file.Reason)
	}
	if now.IsZero() {
		now = time.Now().UTC()
	}
	evidenceID := StableEvidenceID(projectID, file.Path)
	evidence := ppm.Evidence{
		ID:         evidenceID,
		ProjectID:  projectID,
		Kind:       ppm.EvidenceKindFile,
		Summary:    file.Evidence.Summary,
		Source:     file.Evidence.Source,
		Path:       file.Path,
		Actor:      "system",
		ObservedAt: now,
		CreatedAt:  now,
		UpdatedAt:  now,
	}
	if err := ppm.ValidateEvidence(evidence); err != nil {
		return ppm.Evidence{}, nil, err
	}
	facts := make([]ppm.Fact, 0, len(file.Facts))
	for _, draft := range file.Facts {
		fact := ppm.Fact{
			ID:         StableFactID(projectID, file.Path, draft.Key),
			ProjectID:  projectID,
			Statement:  draft.Statement,
			TruthClass: ppm.TruthObserved,
			Source:     draft.Source,
			Actor:      "system",
			ObservedAt: now,
			CreatedAt:  now,
			UpdatedAt:  now,
		}
		if err := ppm.ValidateFact(fact); err != nil {
			return ppm.Evidence{}, nil, err
		}
		facts = append(facts, fact)
	}
	return evidence, facts, nil
}
