package wedge

import (
	"fmt"
	"strings"
	"time"

	"github.com/polterware/polterware/backend/domains/agentrun"
	"github.com/polterware/polterware/backend/domains/drift"
	"github.com/polterware/polterware/backend/providers/importer"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

// DriftInput runs the narrow drift checker (B5).
type DriftInput struct {
	ProjectID  string
	EntityID   string
	AgentRunID string
}

// CheckDrift assembles store + fresh import input, runs drift.Check, and optionally appends an AgentRun drift step.
func CheckDrift(store *storepkg.Store, in DriftInput) (drift.Result, error) {
	if in.ProjectID == "" {
		return drift.Result{}, fmt.Errorf("project_id is required")
	}
	if _, err := store.SyncProjectModel(in.ProjectID); err != nil {
		return drift.Result{}, err
	}

	project, err := store.GetProject(in.ProjectID)
	if err != nil {
		return drift.Result{}, fmt.Errorf("unknown project")
	}

	facts, err := store.ListPpmFacts(in.ProjectID, "")
	if err != nil {
		return drift.Result{}, err
	}
	factRefs := make([]drift.FactRef, 0, len(facts))
	for _, fact := range facts {
		evidence, err := store.ListPpmEvidenceForFact(fact.ID)
		if err != nil {
			return drift.Result{}, err
		}
		evidenceIDs := make([]string, 0, len(evidence))
		for _, item := range evidence {
			evidenceIDs = append(evidenceIDs, item.ID)
		}
		factRefs = append(factRefs, drift.FactRef{
			ID:          fact.ID,
			Statement:   fact.Statement,
			TruthClass:  fact.TruthClass,
			Source:      fact.Source,
			EntityID:    fact.EntityID,
			EvidenceIDs: evidenceIDs,
		})
	}

	rels, err := store.ListPpmRelationships(in.ProjectID, in.EntityID)
	if err != nil {
		return drift.Result{}, err
	}
	relRefs := make([]drift.RelationshipRef, 0, len(rels))
	for _, rel := range rels {
		relRefs = append(relRefs, drift.RelationshipRef{
			ID:              rel.ID,
			SubjectEntityID: rel.SubjectEntityID,
			Predicate:       rel.Predicate,
			ObjectEntityID:  rel.ObjectEntityID,
			TruthClass:      rel.TruthClass,
			Source:          rel.Source,
		})
	}

	adrs, err := store.ListKnowledge(in.ProjectID, "adr")
	if err != nil {
		return drift.Result{}, err
	}
	accepted := make([]drift.AcceptedADR, 0)
	for _, item := range adrs {
		if !strings.EqualFold(strings.TrimSpace(item.Status), "accepted") {
			continue
		}
		links, err := store.ListKnowledgeLinks(in.ProjectID, item.ID, "")
		if err != nil {
			return drift.Result{}, err
		}
		entityIDs := make([]string, 0, len(links))
		for _, link := range links {
			entityIDs = append(entityIDs, link.EntityID)
		}
		accepted = append(accepted, drift.AcceptedADR{
			ID:        item.ID,
			Title:     item.Title,
			EntityIDs: entityIDs,
		})
	}

	input := drift.Input{
		ProjectID:     in.ProjectID,
		EntityID:      in.EntityID,
		StoredFacts:   factRefs,
		Relationships: relRefs,
		AcceptedADRs:  accepted,
	}

	rootPath := ExpandRootPath(project.RootPath)
	if rootPath != "" {
		discovered, err := importer.Discover(rootPath)
		if err == nil {
			input.ImportReady = true
			now := time.Now().UTC()
			for _, file := range discovered.Files {
				if file.Skipped {
					continue
				}
				_, facts, err := importer.Materialize(in.ProjectID, file, now)
				if err != nil {
					continue
				}
				for i, fact := range facts {
					key := ""
					if i < len(file.Facts) {
						key = file.Facts[i].Key
					}
					input.ParsedDeps = append(input.ParsedDeps, drift.ParsedDep{
						FactID:    fact.ID,
						Key:       key,
						Statement: fact.Statement,
						Source:    fact.Source,
						Path:      file.Path,
					})
				}
			}
		}
	}

	result := drift.Check(input)
	if in.AgentRunID != "" {
		now := time.Now().UTC()
		if err := store.AppendAgentRunStepIfOpen(in.AgentRunID, func(run agentrun.Run) (agentrun.Step, error) {
			return agentrun.NewDriftStep(run.ID, result, now)
		}); err != nil {
			return drift.Result{}, err
		}
	}
	return result, nil
}
