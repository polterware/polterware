package drift

import "testing"

func TestCheckImportDepsAddedAndRemoved(t *testing.T) {
	result := Check(Input{
		ProjectID:   "proj",
		ImportReady: true,
		StoredFacts: []FactRef{
			{ID: "fact-import-old", Statement: "Go module requires old.com/x", Source: "import:go.mod", TruthClass: "observed"},
			{ID: "fact-manual", Statement: "Human note", Source: "polterware", TruthClass: "declared"},
		},
		ParsedDeps: []ParsedDep{
			{FactID: "fact-import-new", Key: "require:new.com/y", Statement: "Go module requires new.com/y", Source: "import:go.mod", Path: "go.mod"},
		},
	})
	if result.Checker != CheckerNarrow {
		t.Fatalf("checker: %s", result.Checker)
	}
	var added, removed int
	for _, f := range result.Findings {
		switch f.Type {
		case TypeDependencyAdded:
			added++
		case TypeDependencyRemoved:
			removed++
		}
	}
	if added != 1 || removed != 1 {
		t.Fatalf("expected 1 added and 1 removed, got %+v", result.Findings)
	}
}

func TestCheckEdgeDecisionConflict(t *testing.T) {
	result := Check(Input{
		ProjectID: "proj",
		Relationships: []RelationshipRef{
			{
				ID: "rel-1", SubjectEntityID: "ent-a", ObjectEntityID: "ent-b",
				Predicate: "DEPENDS_ON", TruthClass: "inferred", Source: "architecture_layout",
			},
			{
				ID: "rel-2", SubjectEntityID: "ent-a", ObjectEntityID: "ent-c",
				Predicate: "DEPENDS_ON", TruthClass: "declared", Source: "architecture_layout",
			},
			{
				ID: "rel-3", SubjectEntityID: "ent-a", ObjectEntityID: "ent-d",
				Predicate: "RELATED_TO", TruthClass: "inferred", Source: "architecture_layout",
			},
		},
		AcceptedADRs: []AcceptedADR{
			{ID: "adr-1", Title: "Local SQLite", EntityIDs: []string{"ent-b"}},
		},
	})
	count := 0
	for _, f := range result.Findings {
		if f.Type == TypeEdgeDecisionConflict {
			count++
		}
	}
	if count != 1 {
		t.Fatalf("expected one edge decision conflict, got %+v", result.Findings)
	}
}

func TestCheckDeclaredObservedConflict(t *testing.T) {
	result := Check(Input{
		ProjectID: "proj",
		StoredFacts: []FactRef{
			{ID: "f1", EntityID: "ent-1", TruthClass: "declared", Statement: "Go module is example.com/a", Source: "polterware"},
			{ID: "f2", EntityID: "ent-1", TruthClass: "observed", Statement: "Go module is example.com/b", Source: "import:go.mod"},
			{ID: "f3", EntityID: "ent-2", TruthClass: "declared", Statement: "Go module is example.com/a", Source: "polterware"},
			{ID: "f4", EntityID: "ent-2", TruthClass: "observed", Statement: "Go module is example.com/a", Source: "import:go.mod"},
		},
	})
	count := 0
	for _, f := range result.Findings {
		if f.Type == TypeDeclaredObservedConflict {
			count++
		}
	}
	if count != 1 {
		t.Fatalf("expected one declared/observed conflict, got %+v", result.Findings)
	}
}

func TestCheckEntityFilter(t *testing.T) {
	result := Check(Input{
		ProjectID: "proj",
		EntityID:  "ent-keep",
		StoredFacts: []FactRef{
			{ID: "fact-import-old", Statement: "Go module requires old", Source: "import:go.mod", TruthClass: "observed", EntityID: "ent-other"},
		},
		ParsedDeps: []ParsedDep{},
		Relationships: []RelationshipRef{
			{
				ID: "rel-1", SubjectEntityID: "ent-keep", ObjectEntityID: "ent-b",
				Predicate: "CALLS", TruthClass: "inferred", Source: "architecture_layout",
			},
		},
		AcceptedADRs: []AcceptedADR{
			{ID: "adr-1", Title: "Decision", EntityIDs: []string{"ent-keep"}},
		},
	})
	if len(result.Findings) != 1 || result.Findings[0].Type != TypeEdgeDecisionConflict {
		t.Fatalf("filter should keep only edge conflict: %+v", result.Findings)
	}
}
