package process

import (
	"runtime"
	"testing"
	"time"
)

func TestManagerSpawnCancel(t *testing.T) {
	t.Parallel()
	manager := NewManager()
	name := "sleep"
	args := []string{"30"}
	if runtime.GOOS == "windows" {
		name = "timeout"
		args = []string{"/t", "30", "/nobreak"}
	}
	pid, wait, err := manager.Spawn("spawn-test-1", name, args, "")
	if err != nil {
		t.Fatalf("spawn: %v", err)
	}
	if pid <= 0 {
		t.Fatalf("pid = %d, want > 0", pid)
	}
	manager.Cancel("spawn-test-1")
	select {
	case result := <-wait:
		if !result.Canceled && result.ExitCode == 0 {
			t.Fatalf("expected canceled or nonzero exit, got %+v", result)
		}
	case <-time.After(5 * time.Second):
		t.Fatal("timed out waiting for spawn exit after cancel")
	}
}
