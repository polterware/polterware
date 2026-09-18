package process

import (
	"context"
	"os/exec"
)

// SpawnResult is the outcome of a long-lived spawned process.
type SpawnResult struct {
	ExitCode int
	Err      error
	Canceled bool
}

// Spawn starts a long-lived process without the default operations timeout.
// The PID is returned immediately. Cancel(id) kills the child via context.
// The wait channel receives exactly one result when the process exits, then closes.
func (m *Manager) Spawn(id, name string, args []string, cwd string) (pid int, wait <-chan SpawnResult, err error) {
	ctx, cancel := context.WithCancel(context.Background())
	cmd := exec.CommandContext(ctx, name, args...)
	if cwd != "" {
		cmd.Dir = cwd
	}
	if err := cmd.Start(); err != nil {
		cancel()
		return 0, nil, err
	}
	pid = cmd.Process.Pid
	m.RegisterCancel(id, cancel)

	ch := make(chan SpawnResult, 1)
	go func() {
		defer close(ch)
		defer m.ClearCancel(id)
		waitErr := cmd.Wait()
		result := SpawnResult{}
		if waitErr != nil {
			result.Err = waitErr
			if ee, ok := waitErr.(*exec.ExitError); ok {
				result.ExitCode = ee.ExitCode()
			} else {
				result.ExitCode = -1
			}
		}
		if ctx.Err() != nil {
			result.Canceled = true
		}
		ch <- result
	}()
	return pid, ch, nil
}
