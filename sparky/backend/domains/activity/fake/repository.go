package fake

import (
	"context"

	"github.com/polterware/polterware/sparky/backend/domains/activity"
)

// Repository is a repository-port skeleton. Methods panic until W1 replaces them.
type Repository struct{}

var _ activity.Repository = (*Repository)(nil)

func (*Repository) Append(context.Context, activity.Event) error {
	panic("activity/fake: not implemented")
}

func (*Repository) List(context.Context, string, string, int) (activity.Page, error) {
	panic("activity/fake: not implemented")
}
