package core

import "context"

// Runtime owns provider resources. Close is called after HTTP handlers have stopped.
type Runtime struct {
	Services Services
	MCP      MCP
	Close    func() error
}

// OpenRuntime is the build-time binding file. W1 provider constructors are not yet
// present in this checkout (requests/w1-core-1.md). A merged build replaces this
// binding with the SQLite/domain/planner constructors; no provider needs to edit
// the transport or CLI. Missing ports explicitly return UNAVAILABLE, never fake data.
func OpenRuntime(_ context.Context, _ string) (Runtime, error) {
	return Runtime{Close: func() error { return nil }}, nil
}
