package drift

// CheckerNarrow is the B5 finding producer name for drift.check.
const CheckerNarrow = "narrow"

// Finding types for the narrow checker contract.
const (
	TypeDependencyAdded           = "dependency_added"
	TypeDependencyRemoved         = "dependency_removed"
	TypeEdgeDecisionConflict      = "edge_decision_conflict"
	TypeDeclaredObservedConflict  = "declared_observed_conflict"
)

const (
	SeverityInfo     = "info"
	SeverityWarning  = "warning"
	SeverityConflict = "conflict"
)

// Finding is one narrow drift finding.
type Finding struct {
	Type        string            `json:"type"`
	Summary     string            `json:"summary"`
	Severity    string            `json:"severity"`
	SubjectIDs  []string          `json:"subject_ids,omitempty"`
	EvidenceIDs []string          `json:"evidence_ids,omitempty"`
	Details     map[string]string `json:"details,omitempty"`
}

// Result is the stable drift.check response shape.
type Result struct {
	ProjectID string    `json:"project_id"`
	EntityID  string    `json:"entity_id,omitempty"`
	Status    string    `json:"status"`
	Checker   string    `json:"checker"`
	Findings  []Finding `json:"findings"`
}

// Input is everything the narrow checker needs (pure; no I/O).
type Input struct {
	ProjectID     string
	EntityID      string
	ImportReady   bool // when false, skip import dependency diff (root missing / unreadable)
	StoredFacts   []FactRef
	ParsedDeps    []ParsedDep
	Relationships []RelationshipRef
	AcceptedADRs  []AcceptedADR
}

// FactRef is a stored PPM fact used for drift comparison.
type FactRef struct {
	ID         string
	Statement  string
	TruthClass string
	Source     string
	EntityID   string
	EvidenceIDs []string
}

// ParsedDep is one observed dependency key from a fresh import parse.
type ParsedDep struct {
	FactID    string
	Key       string
	Statement string
	Source    string
	Path      string
}

// RelationshipRef is a projected PPM relationship.
type RelationshipRef struct {
	ID              string
	SubjectEntityID string
	Predicate       string
	ObjectEntityID  string
	TruthClass      string
	Source          string
}

// AcceptedADR is an accepted architecture decision linked to entities.
type AcceptedADR struct {
	ID        string
	Title     string
	EntityIDs []string
}
