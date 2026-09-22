package mcp

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/polterware/polterware/backend/domains/agentrun"
	"github.com/polterware/polterware/backend/domains/drift"
	"github.com/polterware/polterware/backend/providers/importer"
)

func (s *Server) driftCheck(args map[string]any) (string, error) {
	projectID := stringFrom(args["project_id"])
	if projectID == "" {
		return "", fmt.Errorf("project_id is required")
	}
	entityID := stringFrom(args["entity_id"])

	if _, err := s.Store.SyncProjectModel(projectID); err != nil {
		return "", err
	}

	project, err := s.Store.GetProject(projectID)
	if err != nil {
		return "", fmt.Errorf("unknown project")
	}

	facts, err := s.Store.ListPpmFacts(projectID, "")
	if err != nil {
		return "", err
	}
	factRefs := make([]drift.FactRef, 0, len(facts))
	for _, fact := range facts {
		evidence, err := s.Store.ListPpmEvidenceForFact(fact.ID)
		if err != nil {
			return "", err
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

	rels, err := s.Store.ListPpmRelationships(projectID, entityID)
	if err != nil {
		return "", err
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

	adrs, err := s.Store.ListKnowledge(projectID, "adr")
	if err != nil {
		return "", err
	}
	accepted := make([]drift.AcceptedADR, 0)
	for _, item := range adrs {
		if !strings.EqualFold(strings.TrimSpace(item.Status), "accepted") {
			continue
		}
		links, err := s.Store.ListKnowledgeLinks(projectID, item.ID, "")
		if err != nil {
			return "", err
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
		ProjectID:     projectID,
		EntityID:      entityID,
		StoredFacts:   factRefs,
		Relationships: relRefs,
		AcceptedADRs:  accepted,
	}

	rootPath := expandProjectRoot(project.RootPath)
	if rootPath != "" {
		discovered, err := importer.Discover(rootPath)
		if err == nil {
			input.ImportReady = true
			now := time.Now().UTC()
			for _, file := range discovered.Files {
				if file.Skipped {
					continue
				}
				_, facts, err := importer.Materialize(projectID, file, now)
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
	if runID := stringFrom(args["agent_run_id"]); runID != "" {
		now := time.Now().UTC()
		if err := s.Store.AppendAgentRunStepIfOpen(runID, func(run agentrun.Run) (agentrun.Step, error) {
			return agentrun.NewDriftStep(run.ID, result, now)
		}); err != nil {
			return "", err
		}
	}
	return marshalPretty(result)
}

func expandProjectRoot(path string) string {
	path = strings.TrimSpace(path)
	if path == "" {
		return ""
	}
	if path == "~" {
		home, err := os.UserHomeDir()
		if err != nil {
			return ""
		}
		return home
	}
	if strings.HasPrefix(path, "~/") {
		home, err := os.UserHomeDir()
		if err != nil {
			return path
		}
		return filepath.Join(home, path[2:])
	}
	return path
}
