package fake

import (
	"context"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/focus"
)

// Repository is a repository-port skeleton. Methods panic until W1 replaces them.
type Repository struct{}

var _ focus.Repository = (*Repository)(nil)

func (*Repository) InsertSession(context.Context, focus.Session) error {
	panic("focus/fake: not implemented")
}

func (*Repository) ListSessions(context.Context, time.Time, time.Time) ([]focus.Session, error) {
	panic("focus/fake: not implemented")
}
