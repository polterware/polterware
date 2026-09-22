package ppm

import "testing"

func TestEntityIDStable(t *testing.T) {
	a := EntityID("proj", RefArchitectureNode, "n-gw")
	b := EntityID("proj", RefArchitectureNode, "n-gw")
	if a != b || a == "" {
		t.Fatalf("expected stable id, got %q %q", a, b)
	}
}

func TestParseLayoutNodesLegacyAndV1(t *testing.T) {
	legacy := `{"nodes":[{"id":"n-gw","label":"API Gateway","kind":"resource"}]}`
	nodes := ParseLayoutNodes(legacy)
	if len(nodes) != 1 || nodes[0].ID != "n-gw" || nodes[0].Title != "API Gateway" {
		t.Fatalf("legacy parse: %+v", nodes)
	}
	v1 := `{"formatVersion":1,"nodes":[{"id":"n-pg","name":"PostgreSQL","type":"database"}]}`
	nodes = ParseLayoutNodes(v1)
	if len(nodes) != 1 || nodes[0].Title != "PostgreSQL" {
		t.Fatalf("v1 parse: %+v", nodes)
	}
}

func TestDefaultPredicateForKnowledge(t *testing.T) {
	if DefaultPredicateForKnowledge("runbook") != PredOperates {
		t.Fatal("runbook")
	}
	if DefaultPredicateForKnowledge("doc") != PredDocuments {
		t.Fatal("doc")
	}
	if DefaultPredicateForKnowledge("spec") != PredDefines {
		t.Fatal("spec")
	}
}
