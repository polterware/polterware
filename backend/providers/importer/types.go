package importer

// DraftEvidence is file-backed evidence emitted by a parser.
type DraftEvidence struct {
	Path    string
	Summary string
	Source  string
}

// DraftFact is an observed claim keyed for stable IDs.
type DraftFact struct {
	Key       string
	Statement string
	Source    string
}

// FileResult is the parse output for one discovered path.
type FileResult struct {
	Path     string
	Evidence DraftEvidence
	Facts    []DraftFact
	Title    string // Knowledge source title when registering
	Skipped  bool
	Reason   string
}

// Result aggregates a project-root import pass.
type Result struct {
	Files   []FileResult
	Skipped int
}
