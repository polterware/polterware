package mcp_test

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"unicode/utf8"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	mindpkg "github.com/polterware/polterware/sparky/backend/domains/mind"
)

func TestSDKToolsListBudget(t *testing.T) {
	h := setupMCP(t)
	tools, err := h.sess.ListTools(t.Context(), nil)
	if err != nil {
		t.Fatal(err)
	}
	if len(tools.Tools) != 18 {
		t.Fatalf("tools: %d", len(tools.Tools))
	}
	compact := make([]map[string]any, 0, len(tools.Tools))
	for _, tool := range tools.Tools {
		compact = append(compact, map[string]any{"name": tool.Name, "description": tool.Description, "inputSchema": tool.InputSchema})
	}
	wire, err := json.Marshal(map[string]any{"jsonrpc": "2.0", "id": 1, "result": map[string]any{"tools": compact}})
	if err != nil {
		t.Fatal(err)
	}
	if len(wire) > 6000 {
		t.Fatalf("tools/list token payload: %d bytes > 6000", len(wire))
	}
	full, err := json.Marshal(map[string]any{"jsonrpc": "2.0", "id": 1, "result": tools})
	if err != nil {
		t.Fatal(err)
	}
	if len(full) > 6144 {
		t.Fatalf("tools/list SDK remarchal: %d bytes > 6144", len(full))
	}
	t.Logf("SDK tools/list compact %d bytes, remarchal %d bytes", len(wire), len(full))
	for _, tool := range tools.Tools {
		if utf8.RuneCountInString(tool.Description) > 300 {
			t.Fatalf("description: %s", tool.Name)
		}
		if tool.OutputSchema != nil {
			t.Fatalf("output schema leaked: %s", tool.Name)
		}
	}
}

func TestCreateSearchCompleteTranscript(t *testing.T) {
	h := setupMCP(t)
	created := h.call(t, "create_memories", map[string]any{"items": []map[string]any{{"title": "Buy milk", "note": "2%"}}})
	if created.IsError {
		t.Fatal(textOf(created))
	}
	body := structured(t, created)
	id := body["results"].([]any)[0].(map[string]any)["id"].(string)
	if body["actor"] != "mcp:test-client" {
		t.Fatalf("actor: %v", body["actor"])
	}
	search := h.call(t, "search_memories", map[string]any{"query": "milk"})
	if search.IsError {
		t.Fatal(textOf(search))
	}
	got := h.call(t, "get_memory", map[string]any{"id": id})
	if got.IsError {
		t.Fatal(textOf(got))
	}
	done := h.call(t, "set_memory_status", map[string]any{"id": id, "status": "completed"})
	if done.IsError {
		t.Fatal(textOf(done))
	}
	transcript, err := json.Marshal(map[string]any{
		"create": compactTool(created), "search": compactTool(search),
		"get": compactTool(got), "complete": compactTool(done),
	})
	if err != nil {
		t.Fatal(err)
	}
	path := filepath.Join("golden", "create-search-complete.json")
	want, err := os.ReadFile(path)
	if os.IsNotExist(err) {
		if err := os.WriteFile(path, append(transcript, '\n'), 0o644); err != nil {
			t.Fatal(err)
		}
		t.Logf("wrote %s", path)
		return
	}
	if err != nil {
		t.Fatal(err)
	}
	var compact bytes.Buffer
	if err := json.Compact(&compact, want); err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(transcript, compact.Bytes()) {
		t.Fatalf("transcript drifted\ngot %s\nwant %s", transcript, compact.Bytes())
	}
}

func TestListBudgetsAndNameLookup(t *testing.T) {
	h := setupMCP(t)
	items := make([]map[string]any, 0, 20)
	for i := 0; i < 20; i++ {
		items = append(items, map[string]any{"title": "Item"})
	}
	if res := h.call(t, "create_memories", map[string]any{"items": items}); res.IsError {
		t.Fatal(textOf(res))
	}
	listed := h.call(t, "list_memories", map[string]any{"limit": 20})
	if listed.IsError {
		t.Fatal(textOf(listed))
	}
	page, err := json.Marshal(structured(t, listed))
	if err != nil {
		t.Fatal(err)
	}
	if len(page) > 3500 {
		t.Fatalf("list_memories page: %d", len(page))
	}
	id := structured(t, listed)["memories"].([]any)[0].(map[string]any)["id"].(string)
	got := h.call(t, "get_memory", map[string]any{"id": id})
	body, _ := json.Marshal(structured(t, got))
	if len(body) > 1500 {
		t.Fatalf("get_memory: %d %s", len(body), body)
	}
	for i := 0; i < 30; i++ {
		if res := h.call(t, "create_mind", map[string]any{"name": "Mind"}); i == 0 && res.IsError {
			t.Fatal(textOf(res))
		}
	}
	minds := h.call(t, "list_minds", map[string]any{})
	tree, _ := json.Marshal(structured(t, minds))
	if len(tree) > 1200 {
		t.Fatalf("list_minds: %d", len(tree))
	}
	miss := h.call(t, "create_memories", map[string]any{"items": []map[string]any{{"title": "X", "mind": "Wrok"}}})
	row := structured(t, miss)["results"].([]any)[0].(map[string]any)
	if row["ok"] != false || !strings.Contains(row["error"].(string), "not found") {
		t.Fatalf("name lookup: %#v", row)
	}
}

func TestConfirmAndTraversal(t *testing.T) {
	h := setupMCP(t)
	created := h.call(t, "create_memories", map[string]any{"items": []map[string]any{{"title": "Keep me"}}})
	id := structured(t, created)["results"].([]any)[0].(map[string]any)["id"].(string)
	refuse := h.call(t, "delete_memories", map[string]any{"ids": []string{id}})
	if !refuse.IsError || !strings.Contains(strings.ToLower(textOf(refuse)), "confirm") {
		t.Fatalf("confirm: %s", textOf(refuse))
	}
	mind := h.call(t, "create_mind", map[string]any{"name": "Work"})
	mindID := structured(t, mind)["id"].(string)
	refuseMind := h.call(t, "delete_mind", map[string]any{"id": mindID})
	if !refuseMind.IsError || !strings.Contains(strings.ToLower(textOf(refuseMind)), "confirm") {
		t.Fatalf("delete_mind confirm: %s", textOf(refuseMind))
	}
	filesOff := h.call(t, "add_attachments", map[string]any{"memoryId": id, "files": []map[string]any{{"path": "/tmp/x"}}})
	if !filesOff.IsError || !strings.Contains(textOf(filesOff), "Allow file attach") {
		t.Fatalf("file attach setting: %s", textOf(filesOff))
	}
	h.allowFiles(t)
	trav := h.call(t, "add_attachments", map[string]any{"memoryId": id, "files": []map[string]any{{"path": "../etc/passwd"}}})
	if !trav.IsError || !strings.Contains(textOf(trav), "..") {
		t.Fatalf("traversal: %s", textOf(trav))
	}
}

func TestSessionConflictAndBatch(t *testing.T) {
	h := setupMCP(t)
	created := h.call(t, "create_memories", map[string]any{"items": []map[string]any{{"title": "One"}, {"title": "Two"}}})
	if created.IsError {
		t.Fatal(textOf(created))
	}
	id := structured(t, created)["results"].([]any)[0].(map[string]any)["id"].(string)
	if h.call(t, "get_memory", map[string]any{"id": id}).IsError {
		t.Fatal("get")
	}
	title := "UI"
	if _, err := h.fix.MS.Update(h.fix.Ctx, actor.UI(), id, 1, memory.Patch{Title: &title}); err != nil {
		t.Fatal(err)
	}
	conflict := h.call(t, "update_memories", map[string]any{"items": []map[string]any{{"id": id, "title": "Agent"}}})
	row := structured(t, conflict)["results"].([]any)[0].(map[string]any)
	if row["ok"] != false {
		t.Fatalf("expected conflict: %#v", row)
	}
}

func TestSessionRevisionOnDeleteToggleStatusAndAttachments(t *testing.T) {
	h := setupMCP(t)
	created := h.call(t, "create_memories", map[string]any{"items": []map[string]any{{"title": "One", "checklist": []string{"A"}}}})
	if created.IsError {
		t.Fatal(textOf(created))
	}
	id := structured(t, created)["results"].([]any)[0].(map[string]any)["id"].(string)
	if h.call(t, "get_memory", map[string]any{"id": id, "include": []string{"checklist"}}).IsError {
		t.Fatal("get")
	}
	title := "UI"
	if _, err := h.fix.MS.Update(h.fix.Ctx, actor.UI(), id, 1, memory.Patch{Title: &title}); err != nil {
		t.Fatal(err)
	}
	status := h.call(t, "set_memory_status", map[string]any{"id": id, "status": "completed"})
	if !status.IsError || structured(t, status)["conflict"] != true {
		t.Fatalf("status: %s %#v", textOf(status), structured(t, status))
	}
	items, err := h.fix.Mem.ListCheckItems(h.fix.Ctx, id)
	if err != nil || len(items) != 1 {
		t.Fatal(items, err)
	}
	toggle := h.call(t, "toggle_check_item", map[string]any{"memoryId": id, "itemId": items[0].ID})
	if !toggle.IsError || structured(t, toggle)["conflict"] != true {
		t.Fatalf("toggle: %s %#v", textOf(toggle), structured(t, toggle))
	}
	del := h.call(t, "delete_memories", map[string]any{"ids": []string{id}, "confirm": true})
	if del.IsError {
		t.Fatal(textOf(del))
	}
	row := structured(t, del)["results"].([]any)[0].(map[string]any)
	if row["ok"] != false || row["conflict"] != true {
		t.Fatalf("delete: %#v", row)
	}
	attach := h.call(t, "add_attachments", map[string]any{"memoryId": id, "links": []map[string]any{{"url": "https://example.com"}}})
	if !attach.IsError || structured(t, attach)["conflict"] != true {
		t.Fatalf("attachments: %s %#v", textOf(attach), structured(t, attach))
	}
	explicit := h.call(t, "set_memory_status", map[string]any{"id": id, "status": "completed", "expectedRev": 2})
	if explicit.IsError {
		t.Fatal(textOf(explicit))
	}
	mind := h.call(t, "create_mind", map[string]any{"name": "Work"})
	if mind.IsError {
		t.Fatal(textOf(mind))
	}
	mindID := structured(t, mind)["id"].(string)
	if h.call(t, "list_minds", map[string]any{}).IsError {
		t.Fatal("list minds")
	}
	name := "Home"
	if _, err := h.fix.NS.Update(h.fix.Ctx, actor.UI(), mindID, 1, mindpkg.Patch{Name: &name}); err != nil {
		t.Fatal(err)
	}
	drop := h.call(t, "delete_mind", map[string]any{"id": mindID, "confirm": true})
	if !drop.IsError || structured(t, drop)["conflict"] != true {
		t.Fatalf("delete_mind: %s %#v", textOf(drop), structured(t, drop))
	}
}

func compactTool(res *mcp.CallToolResult) map[string]any {
	return map[string]any{"isError": res.IsError, "text": textOf(res), "structured": res.StructuredContent}
}
