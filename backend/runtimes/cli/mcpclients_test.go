package main

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// cursorHome points HOME at a temp dir; withDir controls whether ~/.cursor exists.
func cursorHome(t *testing.T, withDir bool) string {
	t.Helper()
	home := t.TempDir()
	t.Setenv("HOME", home)
	// Hide any real `cursor` binary so installation is decided by ~/.cursor alone.
	t.Setenv("PATH", t.TempDir())
	if withDir {
		if err := os.MkdirAll(filepath.Join(home, ".cursor"), 0o755); err != nil {
			t.Fatal(err)
		}
	}
	return filepath.Join(home, ".cursor", "mcp.json")
}

func readJSON(t *testing.T, path string) map[string]json.RawMessage {
	t.Helper()
	body, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	var out map[string]json.RawMessage
	if err := json.Unmarshal(body, &out); err != nil {
		t.Fatalf("invalid JSON: %v\n%s", err, body)
	}
	return out
}

func TestCursorNotInstalled(t *testing.T) {
	cursorHome(t, false)
	status, err := cursorStatus("/bin/polterware", "/tmp/db")
	if err != nil || status != clientNotInstalled {
		t.Fatalf("status = %v, err = %v; want clientNotInstalled", status, err)
	}
	if err := cursorAdd("/bin/polterware", "/tmp/db"); err == nil {
		t.Fatal("cursorAdd must fail when Cursor is not installed")
	}
	if err := cursorRemove(); err != nil {
		t.Fatalf("cursorRemove without Cursor must be a no-op, got %v", err)
	}
}

func TestCursorMissingFileIsNotConfigured(t *testing.T) {
	cursorHome(t, true)
	status, err := cursorStatus("/bin/polterware", "/tmp/db")
	if err != nil || status != clientNotConfigured {
		t.Fatalf("status = %v, err = %v; want clientNotConfigured", status, err)
	}
}

func TestCursorAddWritesEntryAndIsConfigured(t *testing.T) {
	path := cursorHome(t, true)
	if err := cursorAdd("/bin/polterware", "/tmp/db"); err != nil {
		t.Fatal(err)
	}
	var servers map[string]cursorServer
	if err := json.Unmarshal(readJSON(t, path)["mcpServers"], &servers); err != nil {
		t.Fatal(err)
	}
	entry, ok := servers[mcpServerName]
	if !ok {
		t.Fatalf("missing %q entry: %v", mcpServerName, servers)
	}
	if entry.Command != "/bin/polterware" || strings.Join(entry.Args, " ") != "-db /tmp/db mcp" {
		t.Fatalf("unexpected entry: %+v", entry)
	}
	status, err := cursorStatus("/bin/polterware", "/tmp/db")
	if err != nil || status != clientConfigured {
		t.Fatalf("status = %v, err = %v; want clientConfigured", status, err)
	}
}

func TestCursorStatusDetectsDifferentConfig(t *testing.T) {
	cursorHome(t, true)
	if err := cursorAdd("/bin/polterware", "/tmp/db"); err != nil {
		t.Fatal(err)
	}
	for name, args := range map[string][2]string{
		"other binary": {"/other/polterware", "/tmp/db"},
		"other db":     {"/bin/polterware", "/tmp/other.db"},
	} {
		status, err := cursorStatus(args[0], args[1])
		if err != nil || status != clientConfiguredOther {
			t.Fatalf("%s: status = %v, err = %v; want clientConfiguredOther", name, status, err)
		}
	}
}

func TestCursorAddPreservesUnknownKeysAndOtherServers(t *testing.T) {
	path := cursorHome(t, true)
	seed := `{"theme":"dark","mcpServers":{"other":{"command":"x","args":["y"]}}}`
	if err := os.WriteFile(path, []byte(seed), 0o600); err != nil {
		t.Fatal(err)
	}
	if err := cursorAdd("/bin/polterware", "/tmp/db"); err != nil {
		t.Fatal(err)
	}
	root := readJSON(t, path)
	if string(root["theme"]) != `"dark"` {
		t.Fatalf("unknown root key lost: %s", root["theme"])
	}
	var servers map[string]json.RawMessage
	if err := json.Unmarshal(root["mcpServers"], &servers); err != nil {
		t.Fatal(err)
	}
	if _, ok := servers["other"]; !ok {
		t.Fatal("existing server entry was dropped")
	}
	if _, ok := servers[mcpServerName]; !ok {
		t.Fatal("polterware entry missing")
	}
	info, err := os.Stat(path)
	if err != nil {
		t.Fatal(err)
	}
	if info.Mode().Perm() != 0o600 {
		t.Fatalf("file mode changed to %v", info.Mode().Perm())
	}
}

func TestCursorAddIsIdempotent(t *testing.T) {
	path := cursorHome(t, true)
	for range 2 {
		if err := cursorAdd("/bin/polterware", "/tmp/db"); err != nil {
			t.Fatal(err)
		}
	}
	var servers map[string]json.RawMessage
	if err := json.Unmarshal(readJSON(t, path)["mcpServers"], &servers); err != nil {
		t.Fatal(err)
	}
	if len(servers) != 1 {
		t.Fatalf("want exactly one server, got %d", len(servers))
	}
}

func TestCursorRemoveOnlyDeletesPolterware(t *testing.T) {
	path := cursorHome(t, true)
	seed := `{"mcpServers":{"other":{"command":"x"}}}`
	if err := os.WriteFile(path, []byte(seed), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := cursorAdd("/bin/polterware", "/tmp/db"); err != nil {
		t.Fatal(err)
	}
	if err := cursorRemove(); err != nil {
		t.Fatal(err)
	}
	var servers map[string]json.RawMessage
	if err := json.Unmarshal(readJSON(t, path)["mcpServers"], &servers); err != nil {
		t.Fatal(err)
	}
	if _, ok := servers[mcpServerName]; ok {
		t.Fatal("polterware entry still present")
	}
	if _, ok := servers["other"]; !ok {
		t.Fatal("unrelated server was removed")
	}
	// Removing again is a no-op.
	if err := cursorRemove(); err != nil {
		t.Fatalf("second remove: %v", err)
	}
}

func TestCursorInvalidJSONIsNotOverwritten(t *testing.T) {
	path := cursorHome(t, true)
	const broken = `{"mcpServers": `
	if err := os.WriteFile(path, []byte(broken), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := cursorAdd("/bin/polterware", "/tmp/db"); err == nil {
		t.Fatal("cursorAdd must refuse invalid JSON")
	}
	if body, _ := os.ReadFile(path); string(body) != broken {
		t.Fatalf("invalid file was modified: %s", body)
	}
	if _, err := cursorStatus("/bin/polterware", "/tmp/db"); err == nil {
		t.Fatal("cursorStatus must report invalid JSON")
	}
}

func TestReadCursorConfigRejectsNonObjectServers(t *testing.T) {
	path := filepath.Join(t.TempDir(), "mcp.json")
	if err := os.WriteFile(path, []byte(`{"mcpServers":[1]}`), 0o644); err != nil {
		t.Fatal(err)
	}
	if _, _, err := readCursorConfig(path); err == nil {
		t.Fatal("want error for non-object mcpServers")
	}
}

func TestReadCursorConfigTreatsEmptyAndNullAsEmpty(t *testing.T) {
	dir := t.TempDir()
	for name, body := range map[string]string{"empty": "  \n", "null": `{"mcpServers":null}`} {
		path := filepath.Join(dir, name+".json")
		if err := os.WriteFile(path, []byte(body), 0o644); err != nil {
			t.Fatal(err)
		}
		_, servers, err := readCursorConfig(path)
		if err != nil || len(servers) != 0 {
			t.Fatalf("%s: servers = %v, err = %v", name, servers, err)
		}
	}
}

func TestConfigureClientRefreshesStaleEntry(t *testing.T) {
	var calls []string
	client := mcpClient{
		Name:   "Fake",
		status: func(string, string) (clientStatus, error) { return clientConfiguredOther, nil },
		add:    func(string, string) error { calls = append(calls, "add"); return nil },
		remove: func() error { calls = append(calls, "remove"); return nil },
	}
	if err := configureClient(client, "core", "db"); err != nil {
		t.Fatal(err)
	}
	if strings.Join(calls, ",") != "remove,add" {
		t.Fatalf("calls = %v; want remove,add", calls)
	}

	calls = nil
	client.status = func(string, string) (clientStatus, error) { return clientConfigured, nil }
	if err := configureClient(client, "core", "db"); err != nil || len(calls) != 0 {
		t.Fatalf("configured client must be left alone, calls = %v, err = %v", calls, err)
	}

	client.status = func(string, string) (clientStatus, error) { return clientNotInstalled, nil }
	if err := configureClient(client, "core", "db"); err == nil {
		t.Fatal("want error for a client that is not installed")
	}
}

func TestMCPArgs(t *testing.T) {
	if got := strings.Join(mcpArgs("/x.db"), " "); got != "-db /x.db mcp" {
		t.Fatalf("mcpArgs = %q", got)
	}
}

func TestMCPClientsCoverClaudeCodexCursor(t *testing.T) {
	var ids []string
	for _, c := range mcpClients() {
		if c.status == nil || c.add == nil || c.remove == nil {
			t.Fatalf("client %q has nil hooks", c.ID)
		}
		ids = append(ids, c.ID)
	}
	if strings.Join(ids, ",") != "claude,codex,cursor" {
		t.Fatalf("clients = %v", ids)
	}
}
