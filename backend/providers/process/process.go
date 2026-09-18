package process

import (
	"bytes"
	"context"
	"os/exec"
	"sync"
	"time"

	"github.com/polterware/polterware/backend/domains/logs"
)

const DefaultTimeout = 10 * time.Minute

type Result struct {
	ExitCode int
	Stdout   string
	Stderr   string
	Err      error
}

func Run(ctx context.Context, name string, args []string, cwd string) Result {
	if _, ok := ctx.Deadline(); !ok {
		var cancel context.CancelFunc
		ctx, cancel = context.WithTimeout(ctx, DefaultTimeout)
		defer cancel()
	}
	cmd := exec.CommandContext(ctx, name, args...)
	if cwd != "" {
		cmd.Dir = cwd
	}
	var stdout, stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	err := cmd.Run()
	code := 0
	if err != nil {
		if ee, ok := err.(*exec.ExitError); ok {
			code = ee.ExitCode()
		} else {
			code = -1
		}
	}
	out := stdout.String()
	if len(out) > logs.DefaultChunkBytes {
		out = out[:logs.DefaultChunkBytes]
	}
	return Result{ExitCode: code, Stdout: out, Stderr: stderr.String(), Err: err}
}

type Manager struct {
	mu   sync.Mutex
	runs map[string]context.CancelFunc
}

func NewManager() *Manager {
	return &Manager{runs: map[string]context.CancelFunc{}}
}

func (m *Manager) Start(id string, name string, args []string, cwd string) (<-chan string, context.CancelFunc) {
	ctx, cancel := context.WithTimeout(context.Background(), DefaultTimeout)
	ch := make(chan string, logs.DefaultQueueDepth)
	m.mu.Lock()
	m.runs[id] = cancel
	m.mu.Unlock()
	go func() {
		defer close(ch)
		defer func() {
			m.mu.Lock()
			delete(m.runs, id)
			m.mu.Unlock()
		}()
		res := Run(ctx, name, args, cwd)
		select {
		case ch <- res.Stdout:
		default:
		}
		if res.Stderr != "" {
			select {
			case ch <- res.Stderr:
			default:
			}
		}
	}()
	return ch, cancel
}

func (m *Manager) RegisterCancel(id string, cancel context.CancelFunc) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.runs[id] = cancel
}

func (m *Manager) ClearCancel(id string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	delete(m.runs, id)
}

func (m *Manager) Cancel(id string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if c, ok := m.runs[id]; ok {
		c()
	}
}
