package db

import (
	"context"
	"database/sql"
	"fmt"

	"github.com/polterware/polterware/sparky/backend/platform/changes"
)

type ctxKey struct{}

// Querier is the subset of *sql.DB / *sql.Tx repositories use.
type Querier interface {
	ExecContext(context.Context, string, ...any) (sql.Result, error)
	QueryContext(context.Context, string, ...any) (*sql.Rows, error)
	QueryRowContext(context.Context, string, ...any) *sql.Row
}

// TxFrom returns the transaction stored by Within, if any.
func TxFrom(ctx context.Context) (*sql.Tx, bool) {
	tx, ok := ctx.Value(ctxKey{}).(*sql.Tx)
	return tx, ok
}

func withTx(ctx context.Context, tx *sql.Tx) context.Context {
	return context.WithValue(ctx, ctxKey{}, tx)
}

// Q returns the open transaction or the writer/reader handle.
func (h *Handle) Q(ctx context.Context, write bool) Querier {
	if tx, ok := TxFrom(ctx); ok {
		return tx
	}
	if write {
		return h.writer
	}
	return h.reader
}

// Within runs fn in one writer transaction. Nested calls join the outer tx.
// After a successful commit, AfterCommit receives the entity changes (may be nil).
func (h *Handle) Within(ctx context.Context, fn func(context.Context) error) error {
	if _, ok := TxFrom(ctx); ok {
		return fn(ctx)
	}
	tx, err := h.writer.BeginTx(ctx, nil)
	if err != nil {
		return fmt.Errorf("begin tx: %w", err)
	}
	if err := fn(withTx(ctx, tx)); err != nil {
		_ = tx.Rollback()
		h.discardCommitted()
		return err
	}
	if err := tx.Commit(); err != nil {
		h.discardCommitted()
		return fmt.Errorf("commit tx: %w", err)
	}
	h.flushCommit()
	return nil
}

func (h *Handle) flushCommit() {
	batch := h.takeCommitted()
	if h.AfterCommit == nil {
		return
	}
	h.AfterCommit(batch)
}

// AfterCommit is invoked once per committed writer transaction with entity changes.
// Empty batches are still delivered so callers can distinguish "no-op commit".
type AfterCommitFunc func([]changes.Change)
