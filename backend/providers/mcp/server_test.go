package mcp

import (
	"bytes"
	"encoding/json"
	"path/filepath"
	"strings"
	"testing"

	"github.com/polterware/polterware/backend/domains/ppm"
	"github.com/polterware/polterware/backend/providers/seed"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func testServer(t *testing.T) *Server {
	t.Helper()
	store, err := storepkg.New(filepath.Join(t.TempDir(), "mcp.db"))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = store.Close() })
	if err := seed.Defaults(store); err != nil {
		t.Fatal(err)
	}
	if _, err := store.SyncProjectModel(seed.ProjectID); err != nil {
		t.Fatal(err)
	}
	return &Server{Store: store, In: bytes.NewReader(nil), Out: &bytes.Buffer{}, Err: &bytes.Buffer{}}
}

func TestToolsListStableContract(t *testing.T) {
	server := testServer(t)
	result, err := server.dispatch(rpcRequest{Method: "tools/list", ID: json.RawMessage("1")})
	if err != nil {
		t.Fatal(err)
	}
	payload, ok := result.(map[string]any)
	if !ok {
		t.Fatalf("unexpected result type %T", result)
	}
	tools, ok := payload["tools"].([]map[string]any)
	if !ok {
		t.Fatalf("unexpected tools type %T", payload["tools"])
	}
	want := []string{ToolContextGet, ToolEntityGet, ToolEvidenceGet, ToolDriftCheck}
	if len(tools) != len(want) {
		t.Fatalf("expected %d tools, got %d", len(want), len(tools))
	}
	for i, name := range want {
		got, _ := tools[i]["name"].(string)
		if got != name {
			t.Fatalf("tool[%d]=%q, want %q", i, got, name)
		}
	}
}

func TestInitializeStableVersion(t *testing.T) {
	server := testServer(t)
	result, err := server.dispatch(rpcRequest{Method: "initialize", ID: json.RawMessage("1")})
	if err != nil {
		t.Fatal(err)
	}
	payload := result.(map[string]any)
	info := payload["serverInfo"].(map[string]any)
	if info["version"] != "0.2.0" {
		t.Fatalf("version=%v", info["version"])
	}
	instructions, _ := payload["instructions"].(string)
	if strings.Contains(strings.ToLower(instructions), "experimental") {
		t.Fatalf("instructions still experimental: %s", instructions)
	}
}

func TestContextGetHappyPath(t *testing.T) {
	server := testServer(t)
	text, err := server.callTool(ToolContextGet, map[string]any{
		"project_id": seed.ProjectID,
	})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(text, seed.ProjectID) || !strings.Contains(text, `"Items"`) {
		t.Fatalf("unexpected package: %s", text)
	}
}

func TestEntityGetHappyPath(t *testing.T) {
	server := testServer(t)
	entities, err := server.Store.ListPpmEntities(seed.ProjectID, "")
	if err != nil || len(entities) == 0 {
		t.Fatalf("need seeded entities: %v len=%d", err, len(entities))
	}
	text, err := server.callTool(ToolEntityGet, map[string]any{"id": entities[0].ID})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(text, entities[0].ID) {
		t.Fatalf("unexpected entity: %s", text)
	}
}

func TestEvidenceGetHappyPath(t *testing.T) {
	server := testServer(t)
	evidence, err := ppm.NewEvidence(
		seed.ProjectID,
		ppm.EvidenceKindFile,
		"README describes polterware-core",
		"fs",
		"README.md",
		"system",
	)
	if err != nil {
		t.Fatal(err)
	}
	if err := server.Store.UpsertPpmEvidence(evidence); err != nil {
		t.Fatal(err)
	}
	text, err := server.callTool(ToolEvidenceGet, map[string]any{"id": evidence.ID})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(text, evidence.ID) || !strings.Contains(text, "README.md") {
		t.Fatalf("unexpected evidence: %s", text)
	}
}

func TestDriftCheckStub(t *testing.T) {
	server := testServer(t)
	text, err := server.callTool(ToolDriftCheck, map[string]any{
		"project_id": seed.ProjectID,
		"entity_id":  "ent-example",
	})
	if err != nil {
		t.Fatal(err)
	}
	var result DriftCheckResult
	if err := json.Unmarshal([]byte(text), &result); err != nil {
		t.Fatal(err)
	}
	if result.Status != "ok" || result.Checker != "none" {
		t.Fatalf("unexpected stub: %+v", result)
	}
	if result.ProjectID != seed.ProjectID || result.EntityID != "ent-example" {
		t.Fatalf("unexpected ids: %+v", result)
	}
	if result.Findings == nil || len(result.Findings) != 0 {
		t.Fatalf("findings must be empty slice, got %#v", result.Findings)
	}
}

func TestUnknownAndLegacyToolsRejected(t *testing.T) {
	server := testServer(t)
	for _, name := range []string{"search", "compile_context", "get_entity", "unknown"} {
		_, err := server.invokeTool(name, map[string]any{"project_id": seed.ProjectID})
		if err == nil {
			t.Fatalf("expected error for %q", name)
		}
		if !strings.Contains(err.Error(), "unknown tool") {
			t.Fatalf("unexpected error for %q: %v", name, err)
		}
	}
}
