package importer

import (
	"os"
	"path/filepath"
	"testing"
)

func TestDiscoverParsesMVPSources(t *testing.T) {
	root := t.TempDir()
	write := func(rel, body string) {
		abs := filepath.Join(root, filepath.FromSlash(rel))
		if err := os.MkdirAll(filepath.Dir(abs), 0o755); err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(abs, []byte(body), 0o644); err != nil {
			t.Fatal(err)
		}
	}
	write("go.mod", "module github.com/acme/app\n\nrequire (\n\tgithub.com/foo/bar v1.0.0\n)\n")
	write("package.json", `{"name":"acme","dependencies":{"lodash":"4.0.0"},"devDependencies":{"vitest":"1.0.0"}}`)
	write("README.md", "# Acme App\n\nHello\n")
	write("AGENTS.md", "# Agent rules\n")
	write("Dockerfile", "FROM node:20-alpine\nRUN echo hi\n")
	write("docker-compose.yml", "services:\n  api:\n    image: api\n  web:\n    image: web\n")
	write("openapi/openapi.yaml", "openapi: 3.0.0\ninfo:\n  title: Acme API\n  version: 1.2.3\n")
	write("terraform/main.tf", "provider \"aws\" {}\nresource \"aws_s3_bucket\" \"data\" {}\n")

	result, err := Discover(root)
	if err != nil {
		t.Fatal(err)
	}
	if result.Skipped != 0 {
		t.Fatalf("unexpected skips: %+v", result)
	}
	byPath := map[string]FileResult{}
	for _, file := range result.Files {
		byPath[file.Path] = file
	}
	if len(byPath["go.mod"].Facts) < 2 {
		t.Fatalf("go.mod facts: %+v", byPath["go.mod"].Facts)
	}
	if byPath["package.json"].Title != "acme" {
		t.Fatalf("package title: %q", byPath["package.json"].Title)
	}
	if len(byPath["docker-compose.yml"].Facts) < 2 {
		t.Fatalf("compose facts: %+v", byPath["docker-compose.yml"].Facts)
	}
	open := byPath["openapi/openapi.yaml"]
	if len(open.Facts) < 2 {
		t.Fatalf("openapi facts: %+v", open.Facts)
	}
	tf := byPath["terraform/main.tf"]
	if len(tf.Facts) < 2 {
		t.Fatalf("terraform facts: %+v", tf.Facts)
	}
}

func TestStableIDsAreDeterministic(t *testing.T) {
	a := StableFactID("p1", "go.mod", "module")
	b := StableFactID("p1", "go.mod", "module")
	if a != b || a == "" {
		t.Fatalf("expected stable ids, got %q %q", a, b)
	}
	if StableEvidenceID("p1", "go.mod") == StableEvidenceID("p1", "README.md") {
		t.Fatal("evidence ids should differ by path")
	}
}
