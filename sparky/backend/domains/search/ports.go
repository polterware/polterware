package search

import "context"

// Index maintains memory_fts inside the same transaction as the memory write.
type Index interface {
	Upsert(ctx context.Context, doc Document) error
	Delete(ctx context.Context, memoryID string) error
	Search(ctx context.Context, query string, limit int) ([]Hit, error)
	Rebuild(ctx context.Context, docs []Document) error
	Count(ctx context.Context) (int, error)
}

// Service is the search use-case port.
type Service interface {
	Search(ctx context.Context, query string, filter Filter) (Result, error)
	Status(ctx context.Context) (Status, error)
	Reindex(ctx context.Context) (int, error)
}
