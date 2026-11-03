package localcore

import (
	"context"
	"errors"
	"path/filepath"
	"strings"
	"time"

	"connectrpc.com/connect"

	"github.com/polterware/polterware/backend/compositions/wedge"
	"github.com/polterware/polterware/backend/domains/agentrun"
	"github.com/polterware/polterware/backend/domains/work"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
	"github.com/polterware/polterware/backend/platform/gen/polterware/local/v1/localv1connect"
)

// DispatchHostService is what Polterware Dispatch reads from this core: the
// projects, where a run works and what it works on, the open cards, and the
// AgentRun a run writes under. Dispatch owns its runs; Polterware keeps none.
var _ localv1connect.DispatchHostServiceHandler = (*Core)(nil)

func (core *Core) ListHostProjects(context.Context, *connect.Request[localv1.ListHostProjectsRequest]) (*connect.Response[localv1.ListHostProjectsResponse], error) {
	items, err := core.Store.ListProjects(false)
	if err != nil {
		return nil, internal(err)
	}
	out := make([]*localv1.HostProject, 0, len(items))
	for _, p := range items {
		out = append(out, &localv1.HostProject{Id: p.ID, Name: p.Name})
	}
	return connect.NewResponse(&localv1.ListHostProjectsResponse{Projects: out}), nil
}

// ResolveHostProject is the project with id, or the one whose repository is
// cloned at dir.
func (core *Core) ResolveHostProject(_ context.Context, request *connect.Request[localv1.ResolveHostProjectRequest]) (*connect.Response[localv1.ResolveHostProjectResponse], error) {
	project, err := wedge.ResolveProject(core.Store, request.Msg.GetId(), request.Msg.GetDir())
	if err != nil {
		return nil, connect.NewError(connect.CodeNotFound, err)
	}
	return connect.NewResponse(&localv1.ResolveHostProjectResponse{Project: &localv1.HostProject{Id: project.ID, Name: project.Name}}), nil
}

// ResolveRun settles a run of the project: its directory (the request's, else
// the repository's clone, else the card's component repository, else the
// only one), the component's kinds and technologies, and the card's text.
func (core *Core) ResolveRun(_ context.Context, request *connect.Request[localv1.ResolveRunRequest]) (*connect.Response[localv1.ResolveRunResponse], error) {
	projectID := strings.TrimSpace(request.Msg.GetProjectId())
	if _, err := core.Store.GetProject(projectID); err != nil {
		return nil, invalid("unknown project " + projectID)
	}
	cardSlug := strings.TrimSpace(request.Msg.GetCardSlug())
	dir := wedge.ExpandPath(request.Msg.GetDir())
	if dir == "" {
		picked, err := wedge.RunDir(core.Store, projectID, request.Msg.GetRepo(), cardSlug)
		if err != nil {
			return nil, invalid(err.Error())
		}
		dir = picked
	}
	if abs, err := filepath.Abs(dir); err == nil {
		dir = abs
	}
	out := &localv1.ResolveRunResponse{Dir: dir, CardText: wedge.RunCardText(core.Store, projectID, cardSlug)}
	if node, ok, err := wedge.RunNode(core.Store, projectID, cardSlug, dir); err == nil && ok {
		out.Kinds = splitList(node.Kind)
		out.Tech = splitList(node.Tech)
	}
	return connect.NewResponse(out), nil
}

// ListHostCards lists the project's open cards (not removed, not Done) and
// its repositories, for a run's Card and Repository pills.
func (core *Core) ListHostCards(_ context.Context, request *connect.Request[localv1.ListHostCardsRequest]) (*connect.Response[localv1.ListHostCardsResponse], error) {
	projectID := strings.TrimSpace(request.Msg.GetProjectId())
	snapshot, err := core.Store.LoadWorkSnapshot(projectID)
	if err != nil {
		return nil, internal(err)
	}
	out := &localv1.ListHostCardsResponse{}
	for _, card := range snapshot.Cards {
		if card.Status != work.CardActive || card.Stage == work.StageDelivered {
			continue
		}
		out.Cards = append(out.Cards, &localv1.HostCard{Slug: card.Slug, Title: card.Title, Milestone: card.Milestone, Stage: card.Stage})
	}
	repos, err := core.Store.ListRepositories(projectID)
	if err != nil {
		return nil, internal(err)
	}
	for _, r := range repos {
		out.Repositories = append(out.Repositories, &localv1.HostRepository{Name: r.Name, Remote: r.Remote, LocalPath: r.LocalPath})
	}
	return connect.NewResponse(out), nil
}

// StartHostAgentRun opens the audited AgentRun a Dispatch run's agent writes
// under, so Activity shows one history for both.
func (core *Core) StartHostAgentRun(_ context.Context, request *connect.Request[localv1.StartHostAgentRunRequest]) (*connect.Response[localv1.StartHostAgentRunResponse], error) {
	projectID := strings.TrimSpace(request.Msg.GetProjectId())
	if _, err := core.Store.GetProject(projectID); err != nil {
		return nil, invalid("unknown project " + projectID)
	}
	run, err := agentrun.Start(projectID, hostRunTask(request.Msg.GetTask()), "dispatch", time.Now().UTC())
	if err != nil {
		return nil, invalid(err.Error())
	}
	if err := core.Store.CreateAgentRun(run); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.StartHostAgentRunResponse{AgentRunId: run.ID}), nil
}

// CompleteHostAgentRun closes an AgentRun StartHostAgentRun opened; a closed
// one stays as it is.
func (core *Core) CompleteHostAgentRun(_ context.Context, request *connect.Request[localv1.CompleteHostAgentRunRequest]) (*connect.Response[localv1.CompleteHostAgentRunResponse], error) {
	run, err := core.Store.GetAgentRun(strings.TrimSpace(request.Msg.GetAgentRunId()))
	if err != nil {
		return nil, connect.NewError(connect.CodeNotFound, errors.New("unknown agent run"))
	}
	if run.Status == agentrun.StatusOpen {
		done, err := agentrun.Complete(run, time.Now().UTC())
		if err != nil {
			return nil, invalid(err.Error())
		}
		if err := core.Store.UpdateAgentRun(done); err != nil {
			return nil, internal(err)
		}
	}
	return connect.NewResponse(&localv1.CompleteHostAgentRunResponse{}), nil
}

// hostRunTask is the one-line task an AgentRun shows for a delegation.
func hostRunTask(task string) string {
	line := strings.TrimSpace(strings.SplitN(strings.TrimSpace(task), "\n", 2)[0])
	if len(line) > 160 {
		line = line[:160]
	}
	if line == "" {
		line = "delegated run"
	}
	return line
}

func splitList(value string) []string {
	var out []string
	for _, part := range strings.Split(value, ",") {
		if part = strings.TrimSpace(part); part != "" {
			out = append(out, part)
		}
	}
	return out
}
