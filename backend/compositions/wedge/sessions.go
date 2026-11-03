package wedge

import (
	"sort"
	"time"

	"github.com/polterware/polterware/backend/domains/agentrun"
	"github.com/polterware/polterware/backend/domains/work"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

const (
	sessionRunWindow  = 200
	recentHandoffsMax = 10
)

// SessionClaim is one task an agent holds.
type SessionClaim struct {
	Slug   string
	Number int
	Title  string
	Scope  string
}

// AgentSession is one agent working on the project now: an open agent run
// and the claims its label holds. A label with claims and no open run still
// shows, with an empty RunID.
type AgentSession struct {
	RunID     string
	Agent     string
	Task      string
	StartedAt time.Time
	Claims    []SessionClaim
}

// SessionHandoff is one recorded handoff, newest first in AgentSessions.
type SessionHandoff struct {
	Slug       string
	Branch     string
	Goal       string
	Done       string
	InProgress string
	Next       string
	Agent      string
	At         time.Time
}

// AgentSessions lists the live sessions of a project and its most recent
// handoffs. Branch is the project checkout's branch, shared by all sessions
// because the core does not track a checkout per session.
func AgentSessions(store *storepkg.Store, projectID string) (sessions []AgentSession, handoffs []SessionHandoff, branch string, err error) {
	snapshot, err := store.LoadWorkSnapshot(projectID)
	if err != nil {
		return nil, nil, "", err
	}
	runs, err := store.ListAgentRuns(projectID, sessionRunWindow)
	if err != nil {
		return nil, nil, "", err
	}

	claims := map[string][]SessionClaim{}
	for _, card := range snapshot.Cards {
		if card.Status != work.CardActive {
			continue
		}
		for _, task := range card.Tasks {
			if task.Claim == nil {
				continue
			}
			claims[task.Claim.Holder] = append(claims[task.Claim.Holder], SessionClaim{
				Slug: card.Slug, Number: task.Number, Title: task.Title, Scope: task.Claim.Scope,
			})
		}
	}

	seen := map[string]bool{}
	for _, run := range runs {
		if run.Status != agentrun.StatusOpen || run.ActorKind == agentrun.ActorKindHuman {
			continue
		}
		seen[run.AgentLabel] = true
		sessions = append(sessions, AgentSession{
			RunID: run.ID, Agent: run.AgentLabel, Task: run.Task, StartedAt: run.CreatedAt,
			Claims: claims[run.AgentLabel],
		})
	}
	for holder, held := range claims {
		if !seen[holder] {
			sessions = append(sessions, AgentSession{Agent: holder, Claims: held})
		}
	}
	sort.SliceStable(sessions, func(i, j int) bool {
		if sessions[i].StartedAt.Equal(sessions[j].StartedAt) {
			return sessions[i].Agent < sessions[j].Agent
		}
		return sessions[i].StartedAt.After(sessions[j].StartedAt)
	})

	for _, card := range snapshot.Cards {
		for _, h := range card.Handoffs {
			handoffs = append(handoffs, SessionHandoff{
				Slug: card.Slug, Branch: h.Branch, Goal: h.Goal, Done: h.Done,
				InProgress: h.InProgress, Next: h.Next, Agent: h.Agent, At: h.CreatedAt,
			})
		}
	}
	sort.SliceStable(handoffs, func(i, j int) bool { return handoffs[i].At.After(handoffs[j].At) })
	if len(handoffs) > recentHandoffsMax {
		handoffs = handoffs[:recentHandoffsMax]
	}
	return sessions, handoffs, ProjectGit(store, projectID).Branch, nil
}
