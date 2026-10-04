package fake

import (
	"context"

	"github.com/polterware/polterware/sparky/backend/domains/search"
)

// Index is a repository-port skeleton. Methods panic until W1 replaces them.
type Index struct{}

var _ search.Index = (*Index)(nil)

func (*Index) Upsert(context.Context, search.Document) error {
	panic("search/fake: not implemented")
}

func (*Index) Delete(context.Context, string) error {
	panic("search/fake: not implemented")
}

func (*Index) Search(context.Context, string, int) ([]search.Hit, error) {
	panic("search/fake: not implemented")
}

func (*Index) Rebuild(context.Context, []search.Document) error {
	panic("search/fake: not implemented")
}

func (*Index) Count(context.Context) (int, error) {
	panic("search/fake: not implemented")
}

// Vectors is a repository-port skeleton. Methods panic until W1 replaces them.
type Vectors struct{}

var _ search.Vectors = (*Vectors)(nil)

func (*Vectors) Upsert(context.Context, search.Vector) error {
	panic("search/fake: not implemented")
}

func (*Vectors) Delete(context.Context, string) error {
	panic("search/fake: not implemented")
}

func (*Vectors) Search(context.Context, string, []float32, int) ([]search.VectorHit, error) {
	panic("search/fake: not implemented")
}

func (*Vectors) Stats(context.Context) (int, error) {
	panic("search/fake: not implemented")
}

func (*Vectors) DropModel(context.Context, string) error {
	panic("search/fake: not implemented")
}
