package contextpkg

import "testing"

func TestCompileRespectsBudget(t *testing.T) {
	items := []Item{
		{Kind: "entity", ID: "e1", Title: "API Gateway", Excerpt: stringsRepeat("x", 400), TruthClass: "declared", Source: "layout"},
		{Kind: "doc", ID: "d1", Title: "Architecture", Excerpt: stringsRepeat("y", 400), TruthClass: "declared", Source: "knowledge"},
		{Kind: "doc", ID: "d2", Title: "Extra", Excerpt: stringsRepeat("z", 400), TruthClass: "inferred", Source: "knowledge"},
	}
	pkg := Compile(Request{ProjectID: "p1", TokenBudget: 200}, items)
	if len(pkg.Items) == 0 {
		t.Fatal("expected at least one item")
	}
	if pkg.TokensUsed > pkg.TokenBudget {
		t.Fatalf("over budget: %d > %d", pkg.TokensUsed, pkg.TokenBudget)
	}
	if pkg.Items[0].Kind != "entity" {
		t.Fatalf("expected entity first, got %s", pkg.Items[0].Kind)
	}
}

func stringsRepeat(s string, n int) string {
	out := make([]byte, 0, n*len(s))
	for i := 0; i < n; i++ {
		out = append(out, s...)
	}
	return string(out)
}
