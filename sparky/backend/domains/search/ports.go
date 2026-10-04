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

// Vectors stores embedding blobs and runs top-K search.
type Vectors interface {
	Upsert(ctx context.Context, v Vector) error
	Delete(ctx context.Context, memoryID string) error
	Search(ctx context.Context, model string, query []float32, k int) ([]VectorHit, error)
	Stats(ctx context.Context) (indexed int, err error)
	DropModel(ctx context.Context, model string) error
}

// Embedder turns text into L2-normalized vectors. Implemented by the embeddings provider.
type Embedder interface {
	Embed(ctx context.Context, texts []string) ([][]float32, error)
	Dims() int
	Name() string
}

// Service is the search use-case port.
type Service interface {
	Search(ctx context.Context, query, mode string, filter Filter) (Result, error)
	Status(ctx context.Context) (Status, error)
	Reindex(ctx context.Context) (int, error)
	ConfigureEmbedding(ctx context.Context, cfg EmbeddingConfig) (Status, error)
	TestEmbedding(ctx context.Context) (EmbeddingTest, error)
}
