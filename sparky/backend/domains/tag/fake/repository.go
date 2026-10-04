package fake

import (
	"context"

	"github.com/polterware/polterware/sparky/backend/domains/tag"
)

// Repository is a repository-port skeleton. Methods panic until W1 replaces them.
type Repository struct{}

var _ tag.Repository = (*Repository)(nil)

func (*Repository) List(context.Context) ([]tag.Tag, error) {
	panic("tag/fake: not implemented")
}

func (*Repository) Get(context.Context, string) (tag.Tag, error) {
	panic("tag/fake: not implemented")
}

func (*Repository) GetByName(context.Context, string) (tag.Tag, error) {
	panic("tag/fake: not implemented")
}

func (*Repository) Insert(context.Context, tag.Tag) error {
	panic("tag/fake: not implemented")
}

func (*Repository) Update(context.Context, int64, tag.Tag) error {
	panic("tag/fake: not implemented")
}

func (*Repository) Delete(context.Context, string) error {
	panic("tag/fake: not implemented")
}

func (*Repository) Retarget(context.Context, string, string) (int, error) {
	panic("tag/fake: not implemented")
}

func (*Repository) Names(context.Context) ([]string, error) {
	panic("tag/fake: not implemented")
}
