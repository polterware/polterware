package projection

import (
	"strings"
	"testing"

	"github.com/polterware/polterware/backend/domains/atlas"
	"github.com/polterware/polterware/backend/domains/work"
)

func testGraph() ([]atlas.Node, []ADR) {
	nodes := []atlas.Node{
		{ID: "s", Path: "shop", Level: atlas.LevelSystem, Title: "Shop", Status: atlas.NodeActive,
			Decisions:  []string{"ADR 0001"},
			Invariants: []atlas.Invariant{{Text: "Money is integer cents", State: work.InvariantActive}}},
		{ID: "d", ParentID: "s", Path: "shop/auth", Level: atlas.LevelDomain, Title: "Auth", Status: atlas.NodeActive,
			Repo: "api", Folder: "src/auth", Decisions: []string{"adr-tokens"},
			Invariants: []atlas.Invariant{
				{Text: "Tokens expire in 15 minutes", State: work.InvariantActive},
				{Text: "Old rule", State: work.InvariantRemoved},
			}},
	}
	adrs := []ADR{
		{ID: "adr-cents", Title: "ADR 0001: Integer cents", Body: "## Decision\n\nStore cents.", Status: "accepted"},
		{ID: "adr-tokens", Title: "ADR 0002: Short tokens", Body: "## Decision\n\nShort tokens."},
		{ID: "adr-free", Title: "ADR 0003: Free standing", Body: "Body."},
	}
	return nodes, adrs
}

func TestGovernByCitingNodeRepo(t *testing.T) {
	nodes, adrs := testGraph()
	governed := Govern(nodes, adrs)
	ids := func(list []ADR) string {
		var out []string
		for _, adr := range list {
			out = append(out, adr.ID)
		}
		return strings.Join(out, ",")
	}
	if got := ids(governed[""]); got != "adr-cents,adr-free" {
		t.Fatalf("root = %s", got)
	}
	if got := ids(governed["api"]); got != "adr-tokens" {
		t.Fatalf("api = %s", got)
	}
}

func TestADRFile(t *testing.T) {
	_, adrs := testGraph()
	name, content, ok := ADRFile(adrs[0])
	if !ok || name != "0001-integer-cents.md" {
		t.Fatalf("name = %q ok=%v", name, ok)
	}
	if id, ok := ADRMarker(content); !ok || id != "adr-cents" {
		t.Fatalf("marker = %q %v\n%s", id, ok, content)
	}
	if !strings.Contains(content, "# ADR 0001: Integer cents\n\nLifecycle status: accepted\n\n## Decision") {
		t.Fatalf("content = %s", content)
	}
	if _, _, ok := ADRFile(ADR{ID: "x", Title: "Untitled decision"}); ok {
		t.Fatalf("an unnumbered title has no file")
	}
	if _, ok := ADRMarker("# ADR 0001: Hand-written\n"); ok {
		t.Fatalf("hand-written files carry no marker")
	}
}

func TestBlockInheritsAncestors(t *testing.T) {
	nodes, adrs := testGraph()
	governed := Govern(nodes, adrs)
	api := Block("api", nodes, governed["api"], adrs)
	for _, want := range []string{
		"- shop/auth (domain) Auth · path: src/auth",
		"- ADR 0001: Integer cents (accepted)",
		"- ADR 0002: Short tokens",
		"- Money is integer cents (from shop)",
		"- Tokens expire in 15 minutes (from shop/auth)",
	} {
		if !strings.Contains(api, want) {
			t.Fatalf("api block missing %q:\n%s", want, api)
		}
	}
	if strings.Contains(api, "Old rule") || strings.Contains(api, "ADR 0003") || strings.Contains(api, "- shop (system)") {
		t.Fatalf("api block has foreign lines:\n%s", api)
	}
	root := Block("", nodes, governed[""], adrs)
	if strings.Contains(root, "Tokens expire") || !strings.Contains(root, "ADR 0003: Free standing") {
		t.Fatalf("root block:\n%s", root)
	}
	if !strings.HasPrefix(api, BlockBegin+"\n") || !strings.HasSuffix(api, BlockEnd+"\n") {
		t.Fatalf("sentinels:\n%s", api)
	}
}

func TestReplaceBlock(t *testing.T) {
	block := BlockBegin + "\nnew\n" + BlockEnd + "\n"

	got, err := ReplaceBlock("", block)
	if err != nil || got != block {
		t.Fatalf("empty file: %q %v", got, err)
	}
	got, err = ReplaceBlock("# Agents\n\nRules.\n", block)
	if err != nil || got != "# Agents\n\nRules.\n\n"+block {
		t.Fatalf("append: %q %v", got, err)
	}
	existing := "# Agents\n\n" + BlockBegin + "\nold\nhand edit\n" + BlockEnd + "\n\n## Tail\nKeep me.\n"
	got, err = ReplaceBlock(existing, block)
	if err != nil || got != "# Agents\n\n"+block+"\n## Tail\nKeep me.\n" {
		t.Fatalf("replace: %q %v", got, err)
	}
	again, err := ReplaceBlock(got, block)
	if err != nil || again != got {
		t.Fatalf("re-export must be stable: %q", again)
	}
	if _, err := ReplaceBlock("x\n"+BlockBegin+"\nno end\n", block); err == nil {
		t.Fatalf("missing end sentinel must fail")
	}
}
