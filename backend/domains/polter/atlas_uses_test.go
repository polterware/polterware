package polter

import (
	"testing"

	"github.com/polterware/polterware/backend/domains/atlas"
)

func TestLintComponentUnderSystem(t *testing.T) {
	nodes := []atlas.Node{{ID: "s1", Path: "app", Level: atlas.LevelSystem, Status: atlas.NodeActive}}
	text := "---\npolter: 1\n---\n# app/cli: CLI\nlevel: component\nunder: app\n"
	_, diags := Lint(text, &References{Nodes: nodes})
	if HasErrors(diags) {
		t.Fatalf("diags = %+v", diags)
	}
}

func TestLintNodeUses(t *testing.T) {
	nodes := []atlas.Node{
		{ID: "s1", Path: "app", Level: atlas.LevelSystem, Status: atlas.NodeActive},
		{ID: "d1", Path: "app/platform", Level: atlas.LevelDomain, Status: atlas.NodeActive, ParentID: "s1"},
		{ID: "c1", Path: "app/platform/core", Level: atlas.LevelComponent, Status: atlas.NodeActive, ParentID: "d1"},
	}
	ok := "---\npolter: 1\n---\n# app/cli: CLI\nlevel: component\nunder: app\nuses: app/platform/core\n"
	if _, diags := Lint(ok, &References{Nodes: nodes}); HasErrors(diags) {
		t.Fatalf("diags = %+v", diags)
	}
	bad := "---\npolter: 1\n---\n# app/cli: CLI\nlevel: component\nunder: app\nuses: app/missing\n"
	if _, diags := Lint(bad, &References{Nodes: nodes}); !hasDiag(diags, 4, "unknown uses") {
		t.Fatalf("diags = %+v", diags)
	}
	self := "---\npolter: 1\n---\n# app/cli: CLI\nlevel: component\nunder: app\nuses: app/cli\n"
	if _, diags := Lint(self, &References{Nodes: nodes}); !hasDiag(diags, 4, "cannot use itself") {
		t.Fatalf("diags = %+v", diags)
	}
}
