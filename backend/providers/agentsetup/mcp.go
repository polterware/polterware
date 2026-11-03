// MCP clients: registering `polterware -db <db> mcp` with Claude Code, Codex, and Cursor.
package agentsetup

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

// McpServerName is the entry each client keeps for Polterware.
const McpServerName = "polterware"

// McpStatus is how a client stands with the polterware entry.
type McpStatus int

const (
	McpUnknown McpStatus = iota
	McpNotInstalled
	McpNotConfigured
	McpConfigured
	McpConfiguredOther
)

// McpClient configures one agent client to launch `polterware -db <db> mcp`.
type McpClient struct {
	ID     string
	Name   string
	status func(core, dbPath string) (McpStatus, error)
	add    func(core, dbPath string) error
	remove func() error
}

// McpArgs are the arguments a client passes to the binary.
func McpArgs(dbPath string) []string { return []string{"-db", dbPath, "mcp"} }

// McpClients lists the clients Polterware can set up.
func McpClients() []McpClient {
	return []McpClient{
		{
			ID:   "claude",
			Name: "Claude Code",
			status: func(core, dbPath string) (McpStatus, error) {
				return cliStatus("claude", []string{"mcp", "get", McpServerName}, core, dbPath)
			},
			add: func(core, dbPath string) error {
				args := append([]string{"mcp", "add", "--scope", "user", McpServerName, "--", core}, McpArgs(dbPath)...)
				return runClientCLI("claude", args...)
			},
			remove: func() error {
				return runClientCLI("claude", "mcp", "remove", "--scope", "user", McpServerName)
			},
		},
		{
			ID:   "codex",
			Name: "Codex",
			status: func(core, dbPath string) (McpStatus, error) {
				return cliStatus("codex", []string{"mcp", "get", McpServerName}, core, dbPath)
			},
			add: func(core, dbPath string) error {
				args := append([]string{"mcp", "add", McpServerName, "--", core}, McpArgs(dbPath)...)
				return runClientCLI("codex", args...)
			},
			remove: func() error {
				return runClientCLI("codex", "mcp", "remove", McpServerName)
			},
		},
		{
			ID:     "cursor",
			Name:   "Cursor",
			status: cursorStatus,
			add:    cursorAdd,
			remove: cursorRemove,
		},
	}
}

// Status reports whether the client launches this binary on this database.
func (c McpClient) Status(core, dbPath string) (McpStatus, error) { return c.status(core, dbPath) }

// Remove drops the polterware entry from the client.
func (c McpClient) Remove() error { return c.remove() }

// FindMcpClient returns the client with id (claude, codex, or cursor).
func FindMcpClient(id string) (McpClient, bool) {
	for _, client := range McpClients() {
		if client.ID == id {
			return client, true
		}
	}
	return McpClient{}, false
}

// String names the status in the CLI's JSON and the Desktop.
func (s McpStatus) String() string {
	switch s {
	case McpNotInstalled:
		return "not_installed"
	case McpNotConfigured:
		return "not_configured"
	case McpConfigured:
		return "configured"
	case McpConfiguredOther:
		return "configured_other"
	}
	return "unknown"
}

// ConfigureMcp installs or refreshes the polterware entry.
func ConfigureMcp(client McpClient, core, dbPath string) error {
	status, err := client.status(core, dbPath)
	if err != nil {
		return err
	}
	switch status {
	case McpNotInstalled:
		return fmt.Errorf("%s is not installed", client.Name)
	case McpConfigured:
		return nil
	case McpConfiguredOther:
		if err := client.remove(); err != nil {
			return err
		}
	}
	return client.add(core, dbPath)
}

func cliStatus(binary string, args []string, core, dbPath string) (McpStatus, error) {
	if _, err := exec.LookPath(binary); err != nil {
		return McpNotInstalled, nil
	}
	ctx, cancel := context.WithTimeout(context.Background(), 20*time.Second)
	defer cancel()
	out, err := exec.CommandContext(ctx, binary, args...).CombinedOutput()
	if ctx.Err() != nil {
		return McpUnknown, fmt.Errorf("%s timed out", binary)
	}
	if err != nil {
		var exitErr *exec.ExitError
		if errors.As(err, &exitErr) {
			return McpNotConfigured, nil
		}
		return McpUnknown, err
	}
	text := string(out)
	if strings.Contains(text, core) && strings.Contains(text, dbPath) {
		return McpConfigured, nil
	}
	return McpConfiguredOther, nil
}

func runClientCLI(binary string, args ...string) error {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()
	out, err := exec.CommandContext(ctx, binary, args...).CombinedOutput()
	if err != nil {
		msg := strings.TrimSpace(string(out))
		if msg == "" {
			msg = err.Error()
		}
		return fmt.Errorf("%s: %s", binary, firstLine(msg))
	}
	return nil
}

func firstLine(s string) string {
	if i := strings.IndexByte(s, '\n'); i >= 0 {
		return s[:i]
	}
	return s
}

type cursorServer struct {
	Command string   `json:"command"`
	Args    []string `json:"args"`
}

func cursorConfigPath() (string, bool) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", false
	}
	dir := filepath.Join(home, ".cursor")
	if _, err := os.Stat(dir); err != nil {
		if _, lookErr := exec.LookPath("cursor"); lookErr != nil {
			return "", false
		}
	}
	return filepath.Join(dir, "mcp.json"), true
}

// readCursorConfig keeps unknown keys intact; a missing file is an empty config.
func readCursorConfig(path string) (map[string]json.RawMessage, map[string]json.RawMessage, error) {
	root := map[string]json.RawMessage{}
	servers := map[string]json.RawMessage{}
	body, err := os.ReadFile(path)
	if errors.Is(err, os.ErrNotExist) {
		return root, servers, nil
	}
	if err != nil {
		return nil, nil, err
	}
	if strings.TrimSpace(string(body)) != "" {
		if err := json.Unmarshal(body, &root); err != nil {
			return nil, nil, fmt.Errorf("%s is not valid JSON; fix it by hand", path)
		}
	}
	if raw, ok := root["mcpServers"]; ok && len(raw) > 0 && string(raw) != "null" {
		if err := json.Unmarshal(raw, &servers); err != nil {
			return nil, nil, fmt.Errorf("%s: mcpServers is not an object", path)
		}
	}
	return root, servers, nil
}

func writeCursorConfig(path string, root, servers map[string]json.RawMessage) error {
	encoded, err := json.Marshal(servers)
	if err != nil {
		return err
	}
	root["mcpServers"] = encoded
	body, err := json.MarshalIndent(root, "", "  ")
	if err != nil {
		return err
	}
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	mode := os.FileMode(0o644)
	if info, err := os.Stat(path); err == nil {
		mode = info.Mode().Perm()
	}
	tmp := path + ".polterware-tmp"
	if err := os.WriteFile(tmp, append(body, '\n'), mode); err != nil {
		return err
	}
	return os.Rename(tmp, path)
}

func cursorStatus(core, dbPath string) (McpStatus, error) {
	path, ok := cursorConfigPath()
	if !ok {
		return McpNotInstalled, nil
	}
	_, servers, err := readCursorConfig(path)
	if err != nil {
		return McpUnknown, err
	}
	raw, ok := servers[McpServerName]
	if !ok {
		return McpNotConfigured, nil
	}
	var entry cursorServer
	if json.Unmarshal(raw, &entry) == nil && entry.Command == core &&
		strings.Join(entry.Args, "\x00") == strings.Join(McpArgs(dbPath), "\x00") {
		return McpConfigured, nil
	}
	return McpConfiguredOther, nil
}

func cursorAdd(core, dbPath string) error {
	path, ok := cursorConfigPath()
	if !ok {
		return errors.New("Cursor is not installed")
	}
	root, servers, err := readCursorConfig(path)
	if err != nil {
		return err
	}
	entry, err := json.Marshal(cursorServer{Command: core, Args: McpArgs(dbPath)})
	if err != nil {
		return err
	}
	servers[McpServerName] = entry
	return writeCursorConfig(path, root, servers)
}

func cursorRemove() error {
	path, ok := cursorConfigPath()
	if !ok {
		return nil
	}
	root, servers, err := readCursorConfig(path)
	if err != nil {
		return err
	}
	if _, ok := servers[McpServerName]; !ok {
		return nil
	}
	delete(servers, McpServerName)
	return writeCursorConfig(path, root, servers)
}
