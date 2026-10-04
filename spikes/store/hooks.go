package store

import (
	"database/sql"
	"fmt"
	"strings"
	"sync"
	"sync/atomic"

	"modernc.org/sqlite"
)

// SQLite preupdate operation codes (sqlite3.SQLITE_INSERT / DELETE / UPDATE).
const (
	opDelete = 9
	opInsert = 18
	opUpdate = 23
)

// Change is one entity write observed inside a committed transaction.
type Change struct {
	Entity string
	ID     string
	Op     string
}

// ChangeRecorder receives the entity changes of one committed transaction.
// It runs inside the SQLite commit hook and must not touch the same connection.
type ChangeRecorder func(changes []Change)

var recorder atomic.Pointer[ChangeRecorder]

// SetChangeRecorder installs fn for hook-driver connections. nil disables emission.
func SetChangeRecorder(fn ChangeRecorder) {
	if fn == nil {
		recorder.Store(nil)
		return
	}
	recorder.Store(&fn)
}

func init() {
	var d sqlite.Driver
	d.RegisterConnectionHook(func(conn sqlite.ExecQuerierContext, _ string) error {
		hooks, ok := conn.(sqlite.HookRegisterer)
		if !ok {
			return fmt.Errorf("sqlite connection does not implement hooks")
		}
		tx := &pendingTx{}
		hooks.RegisterPreUpdateHook(tx.preUpdate)
		hooks.RegisterCommitHook(tx.commit)
		hooks.RegisterRollbackHook(tx.reset)
		return nil
	})
	sql.Register(hookDriver, &d)
}

type pendingTx struct {
	mu      sync.Mutex
	changes []Change
}

func (p *pendingTx) preUpdate(data sqlite.SQLitePreUpdateData) {
	if recorder.Load() == nil || data.DatabaseName != "main" {
		return
	}
	if skipTable(data.TableName) {
		return
	}
	id := entityID(&data)
	op, ok := opName(data.Op)
	if !ok {
		return
	}
	p.mu.Lock()
	p.changes = append(p.changes, Change{Entity: data.TableName, ID: id, Op: op})
	p.mu.Unlock()
}

func (p *pendingTx) commit() int32 {
	p.mu.Lock()
	changes := p.changes
	p.changes = nil
	p.mu.Unlock()
	if len(changes) == 0 {
		return 0
	}
	if fn := recorder.Load(); fn != nil {
		(*fn)(changes)
	}
	return 0
}

func (p *pendingTx) reset() {
	p.mu.Lock()
	p.changes = nil
	p.mu.Unlock()
}

func skipTable(name string) bool {
	return strings.HasPrefix(name, "sqlite_") || strings.Contains(name, "fts")
}

func opName(op int32) (string, bool) {
	switch op {
	case opInsert:
		return "create", true
	case opUpdate:
		return "update", true
	case opDelete:
		return "delete", true
	default:
		return "", false
	}
}

func entityID(data *sqlite.SQLitePreUpdateData) string {
	cols := make([]any, 1)
	var err error
	if data.Op == opDelete {
		err = data.Old(cols...)
	} else {
		err = data.New(cols...)
	}
	if err != nil || cols[0] == nil {
		if data.Op == opDelete {
			return fmt.Sprintf("%d", data.OldRowID)
		}
		return fmt.Sprintf("%d", data.NewRowID)
	}
	switch v := cols[0].(type) {
	case string:
		return v
	default:
		return fmt.Sprint(v)
	}
}
