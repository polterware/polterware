package wedge

import (
	"fmt"
	"strings"
	"time"

	"github.com/polterware/polterware/backend/domains/agentrun"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

// SessionRuns keeps one open agent run per project for an agent session (an
// MCP process or a Claude Code session), so writes carry only summary and
// rationale. Runs open lazily on the first write and close with the session.
type SessionRuns struct {
	Label string            `json:"label"`
	Runs  map[string]string `json:"runs"`
}

// RunFor returns the session's open run for projectID, starting one when the
// session has none or its run was completed elsewhere.
func (s *SessionRuns) RunFor(store *storepkg.Store, projectID, task string) (string, error) {
	if s.Runs == nil {
		s.Runs = map[string]string{}
	}
	if id := s.Runs[projectID]; id != "" {
		if run, err := store.GetAgentRun(id); err == nil && agentrun.CanAppend(run) == nil {
			return id, nil
		}
	}
	label := strings.TrimSpace(s.Label)
	if label == "" {
		label = "agent session"
	}
	run, err := agentrun.StartWithActor(projectID, task, label, agentrun.ActorKindAgent, time.Now().UTC())
	if err != nil {
		return "", err
	}
	if err := store.CreateAgentRun(run); err != nil {
		return "", err
	}
	s.Runs[projectID] = run.ID
	return run.ID, nil
}

// Close completes every run the session still holds open.
func (s *SessionRuns) Close(store *storepkg.Store) {
	now := time.Now().UTC()
	for projectID, id := range s.Runs {
		run, err := store.GetAgentRun(id)
		if err == nil && agentrun.CanComplete(run) == nil {
			if done, err := agentrun.Complete(run, now); err == nil {
				_ = store.UpdateAgentRun(done)
			}
		}
		delete(s.Runs, projectID)
	}
}

// ApplyInSession is ApplyPolter for an agent session: without AgentRunID a
// real write joins the session's run for the project. Invalid, no-op, or
// unjustified text is rejected before any run opens.
func ApplyInSession(store *storepkg.Store, session *SessionRuns, in ApplyInput) (ApplyResult, error) {
	if in.AgentRunID != "" || in.DryRun {
		return ApplyPolter(store, in)
	}
	preview := in
	preview.DryRun = true
	planned, err := ApplyPolter(store, preview)
	if err != nil || len(planned.Ops) == 0 {
		return planned, err
	}
	if err := agentrun.ValidateChange(planned.Summary, planned.Rationale); err != nil {
		return planned, fmt.Errorf("%w: add summary and rationale to the frontmatter", agentrun.ErrChangeIncomplete)
	}
	runID, err := session.RunFor(store, in.ProjectID, planned.Summary)
	if err != nil {
		return planned, err
	}
	in.AgentRunID = runID
	return ApplyPolter(store, in)
}
