// Package runstate owns the small JSON files that let local Polterware processes
// find each other: the standalone core (core.json) and live MCP sessions
// (mcp-<pid>.json). Files live in <db dir>/run and are private to the user.
package runstate

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"syscall"
	"time"
)

const (
	coreFile  = "core.json"
	mcpPrefix = "mcp-"
)

// Core describes a running `polterware-core serve` process.
type Core struct {
	PID       int       `json:"pid"`
	Endpoint  string    `json:"endpoint"`
	Token     string    `json:"token"`
	DBPath    string    `json:"dbPath"`
	Owner     string    `json:"owner"`
	StartedAt time.Time `json:"startedAt"`
}

// MCPSession describes a running `polterware-core mcp` stdio process.
type MCPSession struct {
	PID           int       `json:"pid"`
	DBPath        string    `json:"dbPath"`
	ClientName    string    `json:"clientName"`
	ClientVersion string    `json:"clientVersion"`
	StartedAt     time.Time `json:"startedAt"`
}

// Dir returns the run-state directory for a database path.
func Dir(dbPath string) string {
	return filepath.Join(filepath.Dir(dbPath), "run")
}

func CorePath(dbPath string) string { return filepath.Join(Dir(dbPath), coreFile) }

func LogPath(dbPath string) string { return filepath.Join(Dir(dbPath), "core.log") }

func MCPPath(dbPath string, pid int) string {
	return filepath.Join(Dir(dbPath), fmt.Sprintf("%s%d.json", mcpPrefix, pid))
}

// EnsureDir creates the run-state directory with user-only permissions.
func EnsureDir(dbPath string) error {
	return os.MkdirAll(Dir(dbPath), 0o700)
}

// WriteJSON writes v atomically with 0600 permissions.
func WriteJSON(path string, v any) error {
	body, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		return err
	}
	if err := os.MkdirAll(filepath.Dir(path), 0o700); err != nil {
		return err
	}
	tmp, err := os.CreateTemp(filepath.Dir(path), ".tmp-*")
	if err != nil {
		return err
	}
	defer os.Remove(tmp.Name())
	if err := tmp.Chmod(0o600); err != nil {
		_ = tmp.Close()
		return err
	}
	if _, err := tmp.Write(append(body, '\n')); err != nil {
		_ = tmp.Close()
		return err
	}
	if err := tmp.Close(); err != nil {
		return err
	}
	return os.Rename(tmp.Name(), path)
}

// PIDAlive reports whether a process with pid exists.
func PIDAlive(pid int) bool {
	if pid <= 0 {
		return false
	}
	err := syscall.Kill(pid, 0)
	return err == nil || errors.Is(err, syscall.EPERM)
}

// ReadCore returns the live core for dbPath. A stale file is removed and
// reported as not found.
func ReadCore(dbPath string) (Core, bool) {
	path := CorePath(dbPath)
	body, err := os.ReadFile(path)
	if err != nil {
		return Core{}, false
	}
	var core Core
	if json.Unmarshal(body, &core) != nil || !PIDAlive(core.PID) {
		_ = os.Remove(path)
		return Core{}, false
	}
	return core, true
}

// RemoveCore deletes core.json only when it still belongs to pid.
func RemoveCore(dbPath string, pid int) {
	path := CorePath(dbPath)
	body, err := os.ReadFile(path)
	if err != nil {
		return
	}
	var core Core
	if json.Unmarshal(body, &core) == nil && core.PID != pid {
		return
	}
	_ = os.Remove(path)
}

// ListMCP returns live MCP sessions for dbPath, oldest first, and prunes stale files.
func ListMCP(dbPath string) []MCPSession {
	entries, err := os.ReadDir(Dir(dbPath))
	if err != nil {
		return nil
	}
	var out []MCPSession
	for _, entry := range entries {
		name := entry.Name()
		if entry.IsDir() || !strings.HasPrefix(name, mcpPrefix) || !strings.HasSuffix(name, ".json") {
			continue
		}
		path := filepath.Join(Dir(dbPath), name)
		body, err := os.ReadFile(path)
		if err != nil {
			continue
		}
		var session MCPSession
		if json.Unmarshal(body, &session) != nil || !PIDAlive(session.PID) {
			_ = os.Remove(path)
			continue
		}
		out = append(out, session)
	}
	sort.Slice(out, func(i, j int) bool { return out[i].StartedAt.Before(out[j].StartedAt) })
	return out
}
