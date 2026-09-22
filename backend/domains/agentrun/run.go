package agentrun

import (
	"crypto/rand"
	"encoding/hex"
	"errors"
	"fmt"
	"strings"
	"time"
)

const (
	StatusOpen      = "open"
	StatusCompleted = "completed"
)

const (
	KindContext  = "context"
	KindChange   = "change"
	KindEvidence = "evidence"
	KindDrift    = "drift"
)

var (
	ErrUnknownRun      = errors.New("unknown agent run")
	ErrRunNotOpen      = errors.New("agent run is not open")
	ErrInvalidKind     = errors.New("invalid agent run step kind")
	ErrChangeIncomplete = errors.New("change step requires summary and rationale")
	ErrEmptyTask       = errors.New("task is required")
	ErrEmptyProject    = errors.New("project_id is required")
)

// Run is a minimal AgentRun audit header (task lives here, not as a step).
type Run struct {
	ID          string
	ProjectID   string
	Task        string
	AgentLabel  string
	Status      string
	CreatedAt   time.Time
	CompletedAt *time.Time
}

// Step is one link in context → change → evidence → drift.
type Step struct {
	ID         string
	AgentRunID string
	Kind       string
	Summary    string
	PayloadJSON string
	CreatedAt  time.Time
}

// Record is a run plus ordered steps for reconstruction.
type Record struct {
	Run   Run
	Steps []Step
}

func NewRunID() string {
	return "arun-" + randomHex(12)
}

func NewStepID() string {
	return "astep-" + randomHex(12)
}

func randomHex(n int) string {
	buf := make([]byte, n)
	_, _ = rand.Read(buf)
	return hex.EncodeToString(buf)
}

func NormalizeKind(kind string) (string, error) {
	switch strings.ToLower(strings.TrimSpace(kind)) {
	case KindContext:
		return KindContext, nil
	case KindChange:
		return KindChange, nil
	case KindEvidence:
		return KindEvidence, nil
	case KindDrift:
		return KindDrift, nil
	default:
		return "", fmt.Errorf("%w: %q", ErrInvalidKind, kind)
	}
}

func ValidateStart(projectID, task string) error {
	if strings.TrimSpace(projectID) == "" {
		return ErrEmptyProject
	}
	if strings.TrimSpace(task) == "" {
		return ErrEmptyTask
	}
	return nil
}

func ValidateChange(summary, rationale string) error {
	if strings.TrimSpace(summary) == "" || strings.TrimSpace(rationale) == "" {
		return ErrChangeIncomplete
	}
	return nil
}

func CanAppend(run Run) error {
	if run.Status != StatusOpen {
		return ErrRunNotOpen
	}
	return nil
}

func CanComplete(run Run) error {
	if run.Status != StatusOpen {
		return ErrRunNotOpen
	}
	return nil
}

func Start(projectID, task, agentLabel string, now time.Time) (Run, error) {
	if err := ValidateStart(projectID, task); err != nil {
		return Run{}, err
	}
	if now.IsZero() {
		now = time.Now().UTC()
	}
	return Run{
		ID:         NewRunID(),
		ProjectID:  strings.TrimSpace(projectID),
		Task:       strings.TrimSpace(task),
		AgentLabel: strings.TrimSpace(agentLabel),
		Status:     StatusOpen,
		CreatedAt:  now.UTC(),
	}, nil
}

func Complete(run Run, now time.Time) (Run, error) {
	if err := CanComplete(run); err != nil {
		return Run{}, err
	}
	if now.IsZero() {
		now = time.Now().UTC()
	}
	completed := now.UTC()
	run.Status = StatusCompleted
	run.CompletedAt = &completed
	return run, nil
}
