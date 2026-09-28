package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"syscall"
	"time"

	"github.com/polterware/polterware/backend/platform/runstate"
)

// selfBinary resolves this executable; the core server and MCP server are its
// own `serve` and `mcp` subcommands.
func selfBinary() (string, error) {
	exe, err := os.Executable()
	if err != nil {
		return "", fmt.Errorf("resolve polterware executable: %w", err)
	}
	if resolved, err := filepath.EvalSymlinks(exe); err == nil {
		exe = resolved
	}
	return exe, nil
}

// startCore launches a detached standalone core and waits for core.json.
func startCore(binary, dbPath string) (runstate.Core, error) {
	if core, ok := runstate.ReadCore(dbPath); ok {
		return core, nil
	}
	if err := runstate.EnsureDir(dbPath); err != nil {
		return runstate.Core{}, err
	}
	logFile, err := os.OpenFile(runstate.LogPath(dbPath), os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o600)
	if err != nil {
		return runstate.Core{}, err
	}
	defer logFile.Close()

	cmd := exec.Command(binary, "-db", dbPath, "serve", "--listen", "127.0.0.1:0")
	cmd.Stdout = logFile
	cmd.Stderr = logFile
	cmd.Env = append(os.Environ(), "POLTERWARE_CORE_OWNER=standalone")
	cmd.SysProcAttr = &syscall.SysProcAttr{Setsid: true}
	if err := cmd.Start(); err != nil {
		return runstate.Core{}, err
	}
	exited := make(chan error, 1)
	go func() { exited <- cmd.Wait() }()

	deadline := time.After(15 * time.Second)
	for {
		if core, ok := runstate.ReadCore(dbPath); ok {
			return core, nil
		}
		select {
		case err := <-exited:
			return runstate.Core{}, fmt.Errorf("core exited before listening (%v); see %s", err, runstate.LogPath(dbPath))
		case <-deadline:
			return runstate.Core{}, fmt.Errorf("timed out waiting for core; see %s", runstate.LogPath(dbPath))
		case <-time.After(150 * time.Millisecond):
		}
	}
}

// stopCore stops a standalone core. Desktop-owned cores are left alone.
func stopCore(dbPath string) error {
	core, ok := runstate.ReadCore(dbPath)
	if !ok {
		return nil
	}
	if core.Owner != "standalone" {
		return fmt.Errorf("core is managed by %s; quit it there", core.Owner)
	}
	if err := syscall.Kill(core.PID, syscall.SIGTERM); err != nil {
		return err
	}
	for i := 0; i < 40; i++ {
		if !runstate.PIDAlive(core.PID) {
			return nil
		}
		time.Sleep(100 * time.Millisecond)
	}
	return fmt.Errorf("core pid %d did not exit", core.PID)
}
