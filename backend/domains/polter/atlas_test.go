package polter

import (
	"strings"
	"testing"

	"github.com/polterware/polterware/backend/domains/atlas"
	"github.com/polterware/polterware/backend/domains/work"
)

func TestDecodeAtlasNode(t *testing.T) {
	text := `---
polter: 1
---
# core: Core
level: domain

The core.

! No network calls during scan

# core/import: Import
level: component
under: core

## git-import: Git import
under: core/import
`
	doc, diags := Decode(text)
	if HasErrors(diags) {
		t.Fatalf("decode: %+v", diags)
	}
	if len(doc.Nodes) != 2 || doc.Nodes[0].Slug != "core" || doc.Nodes[0].Path != "core" {
		t.Fatalf("nodes = %+v", doc.Nodes)
	}
	if level, _ := doc.Nodes[1].MetaValue("level"); level != "component" || len(doc.Nodes[0].Invariants) != 1 {
		t.Fatalf("meta/invariants = %+v", doc.Nodes)
	}
	if len(doc.Cards) != 1 || doc.Cards[0].Slug != "git-import" {
		t.Fatalf("cards = %+v", doc.Cards)
	}
	again := Format(doc)
	round, diags := Decode(again)
	if HasErrors(diags) || len(round.Nodes) != 2 || len(round.Cards) != 1 {
		t.Fatalf("round trip: %+v nodes=%d", diags, len(round.Nodes))
	}
}

func TestLintUnderRequiresComponent(t *testing.T) {
	nodes := []atlas.Node{{
		ID: "n1", Path: "core/import", Level: atlas.LevelComponent, Status: atlas.NodeActive,
	}}
	bad := "---\npolter: 1\n---\n## git-import: Git import\nunder: missing\n"
	_, diags := Lint(bad, &References{Nodes: nodes})
	if !hasDiag(diags, 4, "unknown under") {
		t.Fatalf("diags = %+v", diags)
	}
	ok := "---\npolter: 1\n---\n## git-import: Git import\nunder: core/import\n"
	_, diags = Lint(ok, &References{Nodes: nodes})
	if HasErrors(diags) {
		t.Fatalf("component under should pass: %+v", diags)
	}
	legacy := "---\npolter: 1\n---\n## git-import: Git import\nunder: core/import\n"
	_, diags = Lint(legacy, &References{})
	if HasErrors(diags) {
		t.Fatalf("legacy under should pass with no components: %+v", diags)
	}
}

func TestPlanMoveNode(t *testing.T) {
	snapshot := work.Snapshot{
		ProjectID: "p1",
		Nodes: []atlas.Node{
			{ID: "desktop", Path: "desktop", Level: atlas.LevelSystem, Title: "Desktop", Status: atlas.NodeActive},
			{ID: "mobile", Path: "mobile", Level: atlas.LevelSystem, Title: "Mobile", Status: atlas.NodeActive},
			{ID: "core", ParentID: "desktop", Path: "desktop/core", Level: atlas.LevelDomain, Title: "Core", Status: atlas.NodeActive},
			{ID: "api", ParentID: "core", Path: "desktop/core/api", Level: atlas.LevelComponent, Title: "API", Status: atlas.NodeActive},
		},
		Cards: []work.Card{{
			ID: "card-1", Slug: "import", Title: "Import", Under: "desktop/core/api", Status: work.CardActive,
		}},
	}
	text := "---\npolter: 1\n---\n# desktop/core: Core\nlevel: domain\nunder: mobile\n"
	doc, diags := Decode(text)
	if HasErrors(diags) {
		t.Fatalf("decode: %+v", diags)
	}
	ops, diags := Plan(doc, snapshot)
	if HasErrors(diags) {
		t.Fatalf("plan: %+v", diags)
	}
	moved := map[string]string{}
	for _, op := range ops {
		if op.Kind != work.OpUpdate {
			continue
		}
		if path, ok := op.Fields["path"].(string); ok {
			moved[op.EntityID] = path
		}
		if under, ok := op.Fields["under"].(string); ok {
			moved[op.EntityID] = under
		}
		if parent, ok := op.Fields["parent_id"].(string); ok && op.EntityID == "core" && parent != "mobile" {
			t.Fatalf("parent = %s", parent)
		}
	}
	if moved["core"] != "mobile/core" || moved["api"] != "mobile/core/api" || moved["card-1"] != "mobile/core/api" {
		t.Fatalf("moved = %+v ops = %+v", moved, ops)
	}
}

func TestLintNodeParent(t *testing.T) {
	text := `---
polter: 1
---
# core/import: Import
level: component
under: core
`
	_, diags := Lint(text, &References{})
	if !hasDiag(diags, 4, "unknown under") {
		t.Fatalf("diags = %+v", diags)
	}
}

func TestOracleBudgetDropsAncestors(t *testing.T) {
	system := atlas.Node{ID: "s", Path: "polterware", Level: atlas.LevelSystem, Title: "Polterware", Status: atlas.NodeActive}
	domain := atlas.Node{
		ID: "d", ParentID: "s", Path: "core", Level: atlas.LevelDomain, Title: "Core", Status: atlas.NodeActive,
		Invariants: []atlas.Invariant{{Text: "No network calls during scan", State: work.InvariantActive}},
	}
	component := atlas.Node{ID: "c", ParentID: "d", Path: "core/import", Level: atlas.LevelComponent, Title: "Import", Status: atlas.NodeActive}
	card := work.Card{
		Slug: "git-import", Title: "Git import", Stage: work.StageBuild, Status: work.CardActive, Under: "core/import",
		Intent: "Import HEAD as evidence.",
		Tasks:  []work.Task{{Number: 1, Title: "Upsert commit evidence", State: work.TaskOpen}},
		Questions: []work.Question{{
			Text: "Include merge commits?", Blocking: true, State: work.QuestionOpen, Options: []string{"yes", "no"},
		}},
	}
	nodes := []atlas.Node{system, domain, component}
	full, err := Oracle(nodes, []work.Card{card}, "git-import", "task", 800)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(full, "(from core)") || !strings.Contains(full, "Include merge commits?") || !strings.Contains(full, "domain: core") {
		t.Fatalf("full oracle = %s", full)
	}
	tight, err := Oracle(nodes, []work.Card{card}, "git-import", "task", 40)
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(tight, "domain: core") {
		t.Fatalf("tight budget should drop ancestor lines: %s", tight)
	}
	if !strings.Contains(tight, "## git-import:") || !strings.Contains(tight, "(from core)") {
		t.Fatalf("tight budget should keep the heading and inherited invariant: %s", tight)
	}
}
