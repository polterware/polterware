// Package search is the full-text search port.
package search

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

// Result is SearchService.Search.
type Result struct {
	Hits   []Hit
	Notice string
}

// Status is the full-text index state shown in Settings.
type Status struct {
	FTSDocuments int
}
