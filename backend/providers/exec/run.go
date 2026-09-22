package exec

import (
	"bytes"
	"context"
	osexec "os/exec"
	"time"
)

const DefaultTimeout = 10 * time.Minute
const maxStdoutBytes = 64 * 1024

type Result struct {
	ExitCode int
	Stdout   string
	Stderr   string
	Err      error
}

// Run executes a short-lived command with a default timeout when none is set.
func Run(ctx context.Context, name string, args []string, cwd string) Result {
	if _, ok := ctx.Deadline(); !ok {
		var cancel context.CancelFunc
		ctx, cancel = context.WithTimeout(ctx, DefaultTimeout)
		defer cancel()
	}
	cmd := osexec.CommandContext(ctx, name, args...)
	if cwd != "" {
		cmd.Dir = cwd
	}
	var stdout, stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	err := cmd.Run()
	code := 0
	if err != nil {
		if ee, ok := err.(*osexec.ExitError); ok {
			code = ee.ExitCode()
		} else {
			code = -1
		}
	}
	out := stdout.String()
	if len(out) > maxStdoutBytes {
		out = out[:maxStdoutBytes]
	}
	return Result{ExitCode: code, Stdout: out, Stderr: stderr.String(), Err: err}
}
