package operations

import (
	"errors"
	"time"
)

type Status string

const (
	StatusQueued    Status = "queued"
	StatusRunning   Status = "running"
	StatusSucceeded Status = "succeeded"
	StatusFailed    Status = "failed"
	StatusCancelled Status = "cancelled"
	StatusTimeout   Status = "timeout"
)

type Operation struct {
	ID             string
	CorrelationID  string
	ProjectID      string
	EnvironmentID  string
	Type           string
	Status         Status
	IdempotencyKey string
	Params         map[string]string
	ResultSummary  string
	ErrorMessage   string
	RequestedAt    time.Time
	StartedAt      *time.Time
	FinishedAt     *time.Time
}

var (
	ErrInvalidTransition = errors.New("invalid operation transition")
	ErrTerminal          = errors.New("operation already terminal")
)

func (o *Operation) IsTerminal() bool {
	switch o.Status {
	case StatusSucceeded, StatusFailed, StatusCancelled, StatusTimeout:
		return true
	default:
		return false
	}
}

func (o *Operation) Transition(to Status) error {
	if o.IsTerminal() {
		return ErrTerminal
	}
	allowed := map[Status][]Status{
		StatusQueued:  {StatusRunning, StatusCancelled},
		StatusRunning: {StatusSucceeded, StatusFailed, StatusCancelled, StatusTimeout},
	}
	for _, candidate := range allowed[o.Status] {
		if candidate != to {
			continue
		}
		o.Status = to
		now := time.Now().UTC()
		if to == StatusRunning && o.StartedAt == nil {
			o.StartedAt = &now
		}
		if o.IsTerminal() {
			o.FinishedAt = &now
		}
		return nil
	}
	return ErrInvalidTransition
}
