package fake

import (
	"context"

	"github.com/polterware/polterware/sparky/backend/domains/mind"
)

// Repository is a repository-port skeleton. Methods panic until W1 replaces them.
type Repository struct{}

var _ mind.Repository = (*Repository)(nil)

func (*Repository) List(context.Context) ([]mind.Mind, error) {
	panic("mind/fake: not implemented")
}

func (*Repository) Get(context.Context, string) (mind.Mind, error) {
	panic("mind/fake: not implemented")
}

func (*Repository) Insert(context.Context, mind.Mind) error {
	panic("mind/fake: not implemented")
}

func (*Repository) Update(context.Context, int64, mind.Mind) error {
	panic("mind/fake: not implemented")
}

func (*Repository) Delete(context.Context, string) error {
	panic("mind/fake: not implemented")
}
