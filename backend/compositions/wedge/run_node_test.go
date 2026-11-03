package wedge_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/polterware/polterware/backend/compositions/wedge"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func TestRunNodeFindsTheComponentByCardFolderOrRepository(t *testing.T) {
	store, err := storepkg.New(filepath.Join(t.TempDir(), "work.db"))
	if err != nil {
		t.Fatal(err)
	}
	defer store.Close()
	root := t.TempDir()
	if err := os.MkdirAll(filepath.Join(root, "web", "src"), 0o755); err != nil {
		t.Fatal(err)
	}
	project, err := initWithRepo(store, "Demo", root)
	if err != nil {
		t.Fatal(err)
	}
	result, err := wedge.ApplyPolter(store, wedge.ApplyInput{ProjectID: project.ID, AutoStartLabel: "cli", Text: strings.ReplaceAll(`---
polter: 1
summary: Map the parts
rationale: Runs price models by the component they work in
---

# api: API
level: component · dir: {root} · path: api · kind: service · tech: Go

# web: Web
level: component · dir: {root} · path: web · kind: frontend · tech: TypeScript, Solid

## login: Login
stage: build · under: api
`, "{root}", root)})
	if err != nil || len(result.Diagnostics) > 0 {
		t.Fatalf("apply: %v %+v", err, result.Diagnostics)
	}
	for _, c := range []struct{ card, dir, want string }{
		{"login", filepath.Join(root, "web"), "api"},
		{"", filepath.Join(root, "web", "src"), "web"},
		{"missing", filepath.Join(root, "web"), "web"},
	} {
		node, ok, err := wedge.RunNode(store, project.ID, c.card, c.dir)
		if err != nil || !ok || node.Path != c.want {
			t.Fatalf("card %q dir %q: %+v %v %v, want %s", c.card, c.dir, node.Path, ok, err, c.want)
		}
	}
	if _, ok, _ := wedge.RunNode(store, project.ID, "", root); ok {
		t.Fatal("a root shared by two components names neither")
	}
}
