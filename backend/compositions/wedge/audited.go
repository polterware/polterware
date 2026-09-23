package wedge

import (
	"fmt"
	"time"

	"github.com/polterware/polterware/backend/domains/agentrun"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

// ChangeEnvelope is the justified mutation envelope shared by MCP and Desktop.
type ChangeEnvelope struct {
	Summary    string
	Rationale  string
	AgentRunID string
	ProjectID  string
	Paths      []string
}

// AuditedWriteResult is returned after a successful audited mutation.
type AuditedWriteResult struct {
	AgentRunID string
	SubjectIDs []string
	Result     any
}

// ExecuteAuditedWrite validates the open run (or starts a human run when AgentRunID
// is empty and actorKind is human), runs the mutation, appends a change step, and
// completes auto-started human runs.
func ExecuteAuditedWrite(
	store *storepkg.Store,
	env ChangeEnvelope,
	actorKind string,
	tool string,
	mutate func(run agentrun.Run) ([]string, any, error),
) (AuditedWriteResult, error) {
	kind, err := agentrun.NormalizeActorKind(actorKind)
	if err != nil {
		return AuditedWriteResult{}, err
	}
	if err := agentrun.ValidateChange(env.Summary, env.Rationale); err != nil {
		return AuditedWriteResult{}, err
	}
	if env.ProjectID == "" {
		return AuditedWriteResult{}, agentrun.ErrEmptyProject
	}

	now := time.Now().UTC()
	runID := env.AgentRunID
	autoComplete := false

	if runID == "" {
		if kind != agentrun.ActorKindHuman {
			return AuditedWriteResult{}, agentrun.ErrMissingAgentRunID
		}
		run, startErr := agentrun.StartWithActor(
			env.ProjectID,
			env.Summary,
			"Desktop",
			agentrun.ActorKindHuman,
			now,
		)
		if startErr != nil {
			return AuditedWriteResult{}, startErr
		}
		if err := store.CreateAgentRun(run); err != nil {
			return AuditedWriteResult{}, err
		}
		runID = run.ID
		autoComplete = true
	}

	run, err := store.GetAgentRun(runID)
	if err != nil {
		return AuditedWriteResult{}, err
	}
	if err := agentrun.CanAppend(run); err != nil {
		return AuditedWriteResult{}, err
	}
	if err := agentrun.MatchProject(run, env.ProjectID); err != nil {
		return AuditedWriteResult{}, err
	}
	if run.ActorKind != "" && run.ActorKind != kind {
		return AuditedWriteResult{}, fmt.Errorf("%w: run is %s, write is %s", agentrun.ErrInvalidActorKind, run.ActorKind, kind)
	}

	subjectIDs, result, err := mutate(run)
	if err != nil {
		return AuditedWriteResult{}, err
	}

	if err := store.AppendAgentRunStepIfOpen(runID, func(open agentrun.Run) (agentrun.Step, error) {
		if err := agentrun.MatchProject(open, env.ProjectID); err != nil {
			return agentrun.Step{}, err
		}
		return agentrun.NewMutationChangeStep(
			open.ID,
			env.Summary,
			env.Rationale,
			tool,
			subjectIDs,
			env.Paths,
			now,
		)
	}); err != nil {
		return AuditedWriteResult{}, fmt.Errorf("mutation saved but change step failed: %w", err)
	}

	if autoComplete {
		done, completeErr := agentrun.Complete(run, now)
		if completeErr != nil {
			return AuditedWriteResult{}, fmt.Errorf("mutation saved but complete failed: %w", completeErr)
		}
		if err := store.UpdateAgentRun(done); err != nil {
			return AuditedWriteResult{}, fmt.Errorf("mutation saved but complete failed: %w", err)
		}
	}

	return AuditedWriteResult{
		AgentRunID: runID,
		SubjectIDs: subjectIDs,
		Result:     result,
	}, nil
}
