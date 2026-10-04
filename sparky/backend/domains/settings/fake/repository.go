package fake

import (
	"context"

	"github.com/polterware/polterware/sparky/backend/domains/settings"
)

// Repository is a repository-port skeleton. Methods panic until W1 replaces them.
type Repository struct{}

var _ settings.Repository = (*Repository)(nil)

func (*Repository) Get(context.Context, string) (string, bool, error) {
	panic("settings/fake: not implemented")
}

func (*Repository) Set(context.Context, string, string) error {
	panic("settings/fake: not implemented")
}

func (*Repository) List(context.Context) (map[string]string, error) {
	panic("settings/fake: not implemented")
}
