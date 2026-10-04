// Package search is the full-text and semantic search port.
package search

import "time"

const (
	ModeFTS      = "fts"
	ModeSemantic = "semantic"
	ModeHybrid   = "hybrid"

	EmbedNotConfigured = "not_configured"
	EmbedIndexing      = "indexing"
	EmbedReady         = "ready"
	EmbedUnreachable   = "provider_unreachable"
)

// Document is one memory_fts row. Note and checklist are plain text.
type Document struct {
	MemoryID  string
	Title     string
	Note      string
	Checklist string
	Tags      string
	Mind      string
}

// Hit is one ranked search result.
type Hit struct {
	MemoryID  string
	Title     string
	Score     float64
	Snippet   string
	MatchedIn []string
}

// Filter narrows search before ranking.
type Filter struct {
	MindID             *string
	IncludeDescendants bool
	TagIDs             []string
	Status             string
	Limit              int
}

// Result is SearchService.Search. ModeUsed may differ from the requested mode.
type Result struct {
	Hits     []Hit
	ModeUsed string
	Notice   string
}

// Status is the index and embedding state shown in Settings.
type Status struct {
	FTSDocuments      int
	EmbeddingsIndexed int
	EmbeddingsPending int
	EmbeddingState    string
	EmbeddingProvider string
	EmbeddingModel    string
	EmbeddingDims     int
}

// Vector is one embeddings row. Values are float32 little-endian and L2-normalized.
type Vector struct {
	MemoryID  string
	Model     string
	Dims      int
	Values    []float32
	TextHash  string
	UpdatedAt time.Time
}

// VectorHit is one brute-force neighbor.
type VectorHit struct {
	MemoryID string
	Score    float64
}

// EmbeddingConfig is runtime configuration. APIKey is never persisted.
type EmbeddingConfig struct {
	Enabled  bool
	Provider string
	Model    string
	BaseURL  string
	APIKey   string
}

// EmbeddingTest is the result of embedding a fixed sentence.
type EmbeddingTest struct {
	OK        bool
	Dims      int
	LatencyMs int
	Error     string
}
