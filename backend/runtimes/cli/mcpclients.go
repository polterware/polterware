package main

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

const mcpServerName = "polterware"

type clientStatus int

const (
	clientUnknown clientStatus = iota
	clientNotInstalled
	clientNotConfigured
	clientConfigured
	clientConfiguredOther
)

// mcpClient configures one agent client to launch `polterware-core -db <db> mcp`.
type mcpClient struct {
	ID     string
	Name   string
	status func(core, dbPath string) (clientStatus, error)
	add    func(core, dbPath string) error
	remove func() error
}

func mcpArgs(dbPath string) []string { return []string{"-db", dbPath, "mcp"} }

func mcpClients() []mcpClient {
	return []mcpClient{
		{
			ID:   "claude",
			Name: "Claude Code",
			status: func(core, dbPath string) (clientStatus, error) {
				return cliStatus("claude", []string{"mcp", "get", mcpServerName}, core, dbPath)
			},
			add: func(core, dbPath string) error {
				args := append([]string{"mcp", "add", "--scope", "user", mcpServerName, "--", core}, mcpArgs(dbPath)...)
				return runClientCLI("claude", args...)
			},
			remove: func() error {
				return runClientCLI("claude", "mcp", "remove", "--scope", "user", mcpServerName)
			},
		},
		{
			ID:   "codex",
			Name: "Codex",
			status: func(core, dbPath string) (clientStatus, error) {
				return cliStatus("codex", []string{"mcp", "get", mcpServerName}, core, dbPath)
			},
			add: func(core, dbPath string) error {
				args := append([]string{"mcp", "add", mcpServerName, "--", core}, mcpArgs(dbPath)...)
				return runClientCLI("codex", args...)
			},
			remove: func() error {
				return runClientCLI("codex", "mcp", "remove", mcpServerName)
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

// configureClient installs or refreshes the polterware entry.
func configureClient(client mcpClient, core, dbPath string) error {
	status, err := client.status(core, dbPath)
	if err != nil {
		return err
	}
	switch status {
	case clientNotInstalled:
		return fmt.Errorf("%s is not installed", client.Name)
	case clientConfigured:
		return nil
	case clientConfiguredOther:
		if err := client.remove(); err != nil {
			return err
		}
	}
	return client.add(core, dbPath)
}

func cliStatus(binary string, args []string, core, dbPath string) (clientStatus, error) {
	if _, err := exec.LookPath(binary); err != nil {
		return clientNotInstalled, nil
	}
	ctx, cancel := context.WithTimeout(context.Background(), 20*time.Second)
	defer cancel()
	out, err := exec.CommandContext(ctx, binary, args...).CombinedOutput()
	if ctx.Err() != nil {
		return clientUnknown, fmt.Errorf("%s timed out", binary)
	}
	if err != nil {
		var exitErr *exec.ExitError
		if errors.As(err, &exitErr) {
			return clientNotConfigured, nil
		}
		return clientUnknown, err
	}
	text := string(out)
	if strings.Contains(text, core) && strings.Contains(text, dbPath) {
		return clientConfigured, nil
	}
	return clientConfiguredOther, nil
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

func cursorStatus(core, dbPath string) (clientStatus, error) {
	path, ok := cursorConfigPath()
	if !ok {
		return clientNotInstalled, nil
	}
	_, servers, err := readCursorConfig(path)
	if err != nil {
		return clientUnknown, err
	}
	raw, ok := servers[mcpServerName]
	if !ok {
		return clientNotConfigured, nil
	}
	var entry cursorServer
	if json.Unmarshal(raw, &entry) == nil && entry.Command == core &&
		strings.Join(entry.Args, "\x00") == strings.Join(mcpArgs(dbPath), "\x00") {
		return clientConfigured, nil
	}
	return clientConfiguredOther, nil
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
	entry, err := json.Marshal(cursorServer{Command: core, Args: mcpArgs(dbPath)})
	if err != nil {
		return err
	}
	servers[mcpServerName] = entry
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
	if _, ok := servers[mcpServerName]; !ok {
		return nil
	}
	delete(servers, mcpServerName)
	return writeCursorConfig(path, root, servers)
}
