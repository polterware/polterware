package mcp

import (
	"bytes"
	"encoding/json"
	"path/filepath"
	"strings"
	"testing"

	"github.com/polterware/polterware/backend/domains/drift"
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
	want := []string{
		ToolContextGet, ToolEntityGet, ToolEvidenceGet, ToolDriftCheck,
		ToolAgentRunStart, ToolAgentRunAppend, ToolAgentRunComplete, ToolAgentRunGet, ToolAgentRunList,
	}
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
	if info["version"] != "0.3.0" {
		t.Fatalf("version=%v", info["version"])
	}
	instructions, _ := payload["instructions"].(string)
	if strings.Contains(strings.ToLower(instructions), "experimental") {
		t.Fatalf("instructions still experimental: %s", instructions)
	}
	if !strings.Contains(instructions, "agent_run") {
		t.Fatalf("instructions missing agent_run: %s", instructions)
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

func TestDriftCheckNarrow(t *testing.T) {
	server := testServer(t)
	text, err := server.callTool(ToolDriftCheck, map[string]any{
		"project_id": seed.ProjectID,
		"entity_id":  "ent-example",
	})
	if err != nil {
		t.Fatal(err)
	}
	var result drift.Result
	if err := json.Unmarshal([]byte(text), &result); err != nil {
		t.Fatal(err)
	}
	if result.Status != "ok" || result.Checker != drift.CheckerNarrow {
		t.Fatalf("unexpected result: %+v", result)
	}
	if result.ProjectID != seed.ProjectID || result.EntityID != "ent-example" {
		t.Fatalf("unexpected ids: %+v", result)
	}
	if result.Findings == nil {
		t.Fatal("findings must be a slice, got nil")
	}

	// Put an inferred DEPENDS_ON involving n-pg (ADR-linked) into the layout so Sync keeps it.
	layout, err := server.Store.GetLayout(seed.ProjectID)
	if err != nil {
		t.Fatal(err)
	}
	body := strings.Replace(layout.BodyJSON,
		`{"id":"e6","from":"n-gw","to":"n-pg","kind":"related","evidence":true}`,
		`{"id":"e6","from":"n-gw","to":"n-pg","kind":"depends_on","evidence":false}`,
		1,
	)
	if body == layout.BodyJSON {
		t.Fatal("failed to patch layout edge e6")
	}
	if _, err := server.Store.UpdateLayout(seed.ProjectID, layout.Revision, body); err != nil {
		t.Fatal(err)
	}

	pg, err := server.Store.GetPpmEntityByRef(seed.ProjectID, ppm.RefArchitectureNode, "n-pg")
	if err != nil {
		t.Fatal(err)
	}
	text, err = server.callTool(ToolDriftCheck, map[string]any{
		"project_id": seed.ProjectID,
		"entity_id":  pg.ID,
	})
	if err != nil {
		t.Fatal(err)
	}
	if err := json.Unmarshal([]byte(text), &result); err != nil {
		t.Fatal(err)
	}
	found := false
	for _, f := range result.Findings {
		if f.Type == drift.TypeEdgeDecisionConflict {
			found = true
			break
		}
	}
	if !found {
		t.Fatalf("expected edge_decision_conflict after layout patch, got %+v", result.Findings)
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

func TestAgentRunAuditChain(t *testing.T) {
	server := testServer(t)

	startText, err := server.callTool(ToolAgentRunStart, map[string]any{
		"project_id":  seed.ProjectID,
		"task":        "Validate import drift",
		"agent_label": "test",
	})
	if err != nil {
		t.Fatal(err)
	}
	var started map[string]any
	if err := json.Unmarshal([]byte(startText), &started); err != nil {
		t.Fatal(err)
	}
	runID, _ := started["id"].(string)
	if runID == "" || started["status"] != "open" {
		t.Fatalf("unexpected start: %s", startText)
	}

	if _, err := server.callTool(ToolContextGet, map[string]any{
		"project_id":   seed.ProjectID,
		"agent_run_id": runID,
	}); err != nil {
		t.Fatal(err)
	}

	if _, err := server.callTool(ToolAgentRunAppend, map[string]any{
		"agent_run_id": runID,
		"kind":         "change",
		"summary":      "Document DEPENDS_ON edge",
		"rationale":    "Matches accepted ADR scope",
		"paths":        []any{"docs/adr/0004-product-positioning-and-wedge.md"},
	}); err != nil {
		t.Fatal(err)
	}

	evidence, err := ppm.NewEvidence(
		seed.ProjectID,
		ppm.EvidenceKindFile,
		"ADR text",
		"fs",
		"docs/adr/0004-product-positioning-and-wedge.md",
		"system",
	)
	if err != nil {
		t.Fatal(err)
	}
	if err := server.Store.UpsertPpmEvidence(evidence); err != nil {
		t.Fatal(err)
	}
	if _, err := server.callTool(ToolAgentRunAppend, map[string]any{
		"agent_run_id":  runID,
		"kind":         "evidence",
		"evidence_ids": []any{evidence.ID},
	}); err != nil {
		t.Fatal(err)
	}

	if _, err := server.callTool(ToolDriftCheck, map[string]any{
		"project_id":   seed.ProjectID,
		"agent_run_id": runID,
	}); err != nil {
		t.Fatal(err)
	}

	if _, err := server.callTool(ToolAgentRunComplete, map[string]any{
		"agent_run_id": runID,
	}); err != nil {
		t.Fatal(err)
	}

	getText, err := server.callTool(ToolAgentRunGet, map[string]any{"id": runID})
	if err != nil {
		t.Fatal(err)
	}
	var record map[string]any
	if err := json.Unmarshal([]byte(getText), &record); err != nil {
		t.Fatal(err)
	}
	if record["status"] != "completed" {
		t.Fatalf("expected completed: %s", getText)
	}
	steps, ok := record["steps"].([]any)
	if !ok || len(steps) != 4 {
		t.Fatalf("expected 4 steps, got %#v", record["steps"])
	}
	kinds := make([]string, 0, 4)
	for _, raw := range steps {
		step := raw.(map[string]any)
		kinds = append(kinds, step["kind"].(string))
	}
	wantKinds := []string{"context", "change", "evidence", "drift"}
	for i, want := range wantKinds {
		if kinds[i] != want {
			t.Fatalf("step[%d]=%q, want %q (all=%v)", i, kinds[i], want, kinds)
		}
	}

	listText, err := server.callTool(ToolAgentRunList, map[string]any{
		"project_id": seed.ProjectID,
	})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(listText, runID) {
		t.Fatalf("list missing run: %s", listText)
	}
}
