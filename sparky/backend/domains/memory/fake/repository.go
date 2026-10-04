package fake

import (
	"context"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/memory"
)

// Repository is a repository-port skeleton. Methods panic until W1 replaces them.
type Repository struct{}

var _ memory.Repository = (*Repository)(nil)

func (*Repository) List(context.Context, memory.ListFilter) (memory.ListPage, error) {
	panic("memory/fake: not implemented")
}

func (*Repository) Get(context.Context, string) (memory.Memory, error) {
	panic("memory/fake: not implemented")
}

func (*Repository) Insert(context.Context, memory.Memory) error {
	panic("memory/fake: not implemented")
}

func (*Repository) Update(context.Context, int64, memory.Memory) error {
	panic("memory/fake: not implemented")
}

func (*Repository) Delete(context.Context, string) error {
	panic("memory/fake: not implemented")
}

func (*Repository) ListCheckItems(context.Context, string) ([]memory.CheckItem, error) {
	panic("memory/fake: not implemented")
}

func (*Repository) ReplaceCheckItems(context.Context, string, []memory.CheckItem) error {
	panic("memory/fake: not implemented")
}

func (*Repository) GetCheckItem(context.Context, string) (memory.CheckItem, error) {
	panic("memory/fake: not implemented")
}

func (*Repository) UpdateCheckItem(context.Context, memory.CheckItem) error {
	panic("memory/fake: not implemented")
}

func (*Repository) ListTagIDs(context.Context, string) ([]string, error) {
	panic("memory/fake: not implemented")
}

func (*Repository) SetTagIDs(context.Context, string, []string) error {
	panic("memory/fake: not implemented")
}

func (*Repository) ListCompletions(context.Context, string) ([]memory.Completion, error) {
	panic("memory/fake: not implemented")
}

func (*Repository) InsertCompletion(context.Context, memory.Completion) error {
	panic("memory/fake: not implemented")
}

func (*Repository) DeleteCompletion(context.Context, string, time.Time) error {
	panic("memory/fake: not implemented")
}

func (*Repository) CountByMinds(context.Context, []string) (int, error) {
	panic("memory/fake: not implemented")
}
