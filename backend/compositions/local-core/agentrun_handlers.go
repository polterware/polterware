package localcore

import (
	"context"
	"errors"
	"time"

	"connectrpc.com/connect"

	"github.com/polterware/polterware/backend/domains/agentrun"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
)

func (core *Core) ListAgentRuns(_ context.Context, request *connect.Request[localv1.ListAgentRunsRequest]) (*connect.Response[localv1.ListAgentRunsResponse], error) {
	if request.Msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	runs, err := core.Store.ListAgentRuns(request.Msg.ProjectId, int(request.Msg.Limit))
	if err != nil {
		return nil, internal(err)
	}
	out := make([]*localv1.AgentRun, 0, len(runs))
	for _, run := range runs {
		out = append(out, agentRunProto(run))
	}
	return connect.NewResponse(&localv1.ListAgentRunsResponse{Runs: out}), nil
}

func (core *Core) GetAgentRun(_ context.Context, request *connect.Request[localv1.GetAgentRunRequest]) (*connect.Response[localv1.GetAgentRunResponse], error) {
	if request.Msg.Id == "" {
		return nil, invalid("agent run id is required")
	}
	record, err := core.Store.GetAgentRunRecord(request.Msg.Id)
	if errors.Is(err, agentrun.ErrUnknownRun) {
		return nil, connect.NewError(connect.CodeNotFound, err)
	}
	if err != nil {
		return nil, internal(err)
	}
	steps := make([]*localv1.AgentRunStep, 0, len(record.Steps))
	for _, step := range record.Steps {
		steps = append(steps, agentRunStepProto(step))
	}
	return connect.NewResponse(&localv1.GetAgentRunResponse{
		Run:   agentRunProto(record.Run),
		Steps: steps,
	}), nil
}

func agentRunProto(run agentrun.Run) *localv1.AgentRun {
	out := &localv1.AgentRun{
		Id: run.ID, ProjectId: run.ProjectID, Task: run.Task, AgentLabel: run.AgentLabel,
		Status: run.Status, CreatedAt: run.CreatedAt.UTC().Format(time.RFC3339),
	}
	if run.CompletedAt != nil {
		out.CompletedAt = run.CompletedAt.UTC().Format(time.RFC3339)
	}
	return out
}

func agentRunStepProto(step agentrun.Step) *localv1.AgentRunStep {
	return &localv1.AgentRunStep{
		Id: step.ID, AgentRunId: step.AgentRunID, Kind: step.Kind,
		Summary: step.Summary, PayloadJson: step.PayloadJSON,
		CreatedAt: step.CreatedAt.UTC().Format(time.RFC3339),
	}
}
