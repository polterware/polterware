package localcore

import (
	"context"
	"database/sql"
	"errors"
	"time"

	"connectrpc.com/connect"

	"github.com/polterware/polterware/backend/domains/architecture"
	"github.com/polterware/polterware/backend/domains/integrations"
	"github.com/polterware/polterware/backend/domains/operations"
	"github.com/polterware/polterware/backend/domains/projects"
	"github.com/polterware/polterware/backend/domains/resources"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
	cliprobe "github.com/polterware/polterware/backend/providers/cli"
)

func (core *Core) GetProjectSettings(_ context.Context, request *connect.Request[localv1.GetProjectSettingsRequest]) (*connect.Response[localv1.GetProjectSettingsResponse], error) {
	item, err := core.Store.GetProjectSettings(request.Msg.ProjectId)
	if errors.Is(err, sql.ErrNoRows) {
		item = projects.Settings{ProjectID: request.Msg.ProjectId}
		return connect.NewResponse(&localv1.GetProjectSettingsResponse{Settings: settingsProto(item)}), nil
	}
	if err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.GetProjectSettingsResponse{Settings: settingsProto(item)}), nil
}

func (core *Core) SaveProjectSettings(_ context.Context, request *connect.Request[localv1.SaveProjectSettingsRequest]) (*connect.Response[localv1.SaveProjectSettingsResponse], error) {
	item := request.Msg.GetSettings()
	if item == nil || item.ProjectId == "" {
		return nil, invalid("project settings are required")
	}
	settings := projects.Settings{ProjectID: item.ProjectId, Description: item.Description, DefaultEnvironmentID: item.DefaultEnvironmentId, Repository: item.Repository}
	if err := core.Store.SaveProjectSettings(settings); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.SaveProjectSettingsResponse{Settings: settingsProto(settings)}), nil
}

func (core *Core) ListNodes(_ context.Context, request *connect.Request[localv1.ListNodesRequest]) (*connect.Response[localv1.ListNodesResponse], error) {
	items, err := core.Store.ListNodes(request.Msg.ProjectId)
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.Node, 0, len(items))
	for _, item := range items {
		result = append(result, nodeProto(item))
	}
	return connect.NewResponse(&localv1.ListNodesResponse{Nodes: result}), nil
}

func (core *Core) SaveNode(_ context.Context, request *connect.Request[localv1.SaveNodeRequest]) (*connect.Response[localv1.SaveNodeResponse], error) {
	item := request.Msg.GetNode()
	if item == nil || item.Id == "" || item.ProjectId == "" || item.Name == "" {
		return nil, invalid("node id, project id, and name are required")
	}
	node := resources.Node{
		ID: item.Id, ProjectID: item.ProjectId, EnvironmentID: item.EnvironmentId, Name: item.Name,
		Platform: item.Platform, Shell: item.Shell, Status: item.Status, Address: item.Address,
		CPU: item.Cpu, Memory: item.Memory, LastSeenAt: item.LastSeenAt,
		ConnectionKind: item.ConnectionKind, SSHUser: item.SshUser, SSHPort: int(item.SshPort), SSHIdentityFile: item.SshIdentityFile,
	}
	resources.NormalizeNode(&node)
	if err := resources.ValidateNode(node); err != nil {
		return nil, invalid(err.Error())
	}
	if err := core.Store.SaveNode(node); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.SaveNodeResponse{Node: nodeProto(node)}), nil
}

func (core *Core) ListViews(_ context.Context, request *connect.Request[localv1.ListViewsRequest]) (*connect.Response[localv1.ListViewsResponse], error) {
	items, err := core.Store.ListViews(request.Msg.ProjectId)
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.ArchitectureView, 0, len(items))
	for _, item := range items {
		result = append(result, viewProto(item))
	}
	return connect.NewResponse(&localv1.ListViewsResponse{Views: result}), nil
}

func (core *Core) SaveView(_ context.Context, request *connect.Request[localv1.SaveViewRequest]) (*connect.Response[localv1.SaveViewResponse], error) {
	item := request.Msg.GetView()
	if item == nil || item.Id == "" || item.ProjectId == "" || item.Name == "" {
		return nil, invalid("view id, project id, and name are required")
	}
	view := viewFromProto(item)
	if err := core.Store.SaveView(view); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.SaveViewResponse{View: viewProto(view)}), nil
}

func (core *Core) DeleteView(_ context.Context, request *connect.Request[localv1.DeleteViewRequest]) (*connect.Response[localv1.DeleteViewResponse], error) {
	if err := core.Store.DeleteView(request.Msg.Id); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.DeleteViewResponse{}), nil
}

func (core *Core) ListSchedules(_ context.Context, request *connect.Request[localv1.ListSchedulesRequest]) (*connect.Response[localv1.ListSchedulesResponse], error) {
	items, err := core.Store.ListSchedules(request.Msg.ProjectId)
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.OperationSchedule, 0, len(items))
	for _, item := range items {
		result = append(result, scheduleProto(item))
	}
	return connect.NewResponse(&localv1.ListSchedulesResponse{Schedules: result}), nil
}

func (core *Core) SaveSchedule(_ context.Context, request *connect.Request[localv1.SaveScheduleRequest]) (*connect.Response[localv1.SaveScheduleResponse], error) {
	item := request.Msg.GetSchedule()
	if item == nil || item.Id == "" || item.ProjectId == "" || item.WorkflowId == "" {
		return nil, invalid("schedule id, project id, and workflow id are required")
	}
	timezone := item.Timezone
	if timezone == "" {
		timezone = "UTC"
	}
	if _, _, err := ParseScheduleCron(item.Expression, timezone); err != nil {
		return nil, invalid(err.Error())
	}
	nextRunAt := item.NextRunAt
	if nextRunAt == "" {
		computed, err := NextRunAtString(item.Expression, timezone, time.Now().UTC())
		if err != nil {
			return nil, invalid(err.Error())
		}
		nextRunAt = computed
	} else if _, err := ParseScheduleInstant(nextRunAt); err != nil {
		computed, err := NextRunAtString(item.Expression, timezone, time.Now().UTC())
		if err != nil {
			return nil, invalid(err.Error())
		}
		nextRunAt = computed
	}
	schedule := operations.Schedule{
		ID:            item.Id,
		ProjectID:     item.ProjectId,
		EnvironmentID: item.EnvironmentId,
		Name:          item.Name,
		Description:   item.Description,
		Status:        item.Status,
		WorkflowID:    item.WorkflowId,
		Expression:    item.Expression,
		Timezone:      timezone,
		NextRunAt:     nextRunAt,
		LastRunAt:     item.LastRunAt,
	}
	if err := core.Store.SaveSchedule(schedule); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.SaveScheduleResponse{Schedule: scheduleProto(schedule)}), nil
}

func (core *Core) ListIntegrationResources(_ context.Context, request *connect.Request[localv1.ListIntegrationResourcesRequest]) (*connect.Response[localv1.ListIntegrationResourcesResponse], error) {
	items, err := core.Store.ListIntegrationResources(request.Msg.ProjectId)
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.IntegrationResource, 0, len(items))
	for _, item := range items {
		result = append(result, integrationResourceProto(item))
	}
	return connect.NewResponse(&localv1.ListIntegrationResourcesResponse{Resources: result}), nil
}

func (core *Core) ListIntegrationSyncRuns(_ context.Context, request *connect.Request[localv1.ListIntegrationSyncRunsRequest]) (*connect.Response[localv1.ListIntegrationSyncRunsResponse], error) {
	items, err := core.Store.ListIntegrationSyncRuns(request.Msg.ProjectId)
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.IntegrationSyncRun, 0, len(items))
	for _, item := range items {
		result = append(result, syncRunProto(item))
	}
	return connect.NewResponse(&localv1.ListIntegrationSyncRunsResponse{Runs: result}), nil
}

func (core *Core) SaveIntegrationSyncRun(_ context.Context, request *connect.Request[localv1.SaveIntegrationSyncRunRequest]) (*connect.Response[localv1.SaveIntegrationSyncRunResponse], error) {
	item := request.Msg.GetRun()
	if item == nil || item.Id == "" || item.ConnectionId == "" {
		return nil, invalid("sync run id and connection id are required")
	}
	run := integrations.SyncRun{
		ID:           item.Id,
		ConnectionID: item.ConnectionId,
		StartedAt:    item.StartedAt,
		FinishedAt:   item.FinishedAt,
		Result:       item.Result,
		Imported:     int(item.Imported),
		Changed:      int(item.Changed),
		Message:      item.Message,
	}
	if err := core.Store.SaveIntegrationSyncRun(run); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.SaveIntegrationSyncRunResponse{Run: syncRunProto(run)}), nil
}

func (core *Core) ProbeCliProvider(_ context.Context, request *connect.Request[localv1.ProbeCliProviderRequest]) (*connect.Response[localv1.ProbeCliProviderResponse], error) {
	if request.Msg.ProviderId == "" {
		return nil, invalid("provider id is required")
	}
	result := cliprobe.Probe(request.Msg.ProviderId)
	return connect.NewResponse(&localv1.ProbeCliProviderResponse{
		ProviderId:   result.ProviderID,
		Available:    result.Available,
		BinaryPath:   result.BinaryPath,
		Status:       result.Status,
		AccountLabel: result.AccountLabel,
		Message:      result.Message,
	}), nil
}

func (core *Core) BeginCliAuth(_ context.Context, request *connect.Request[localv1.BeginCliAuthRequest]) (*connect.Response[localv1.BeginCliAuthResponse], error) {
	if request.Msg.ProviderId == "" {
		return nil, invalid("provider id is required")
	}
	plan, err := cliprobe.BeginAuth(request.Msg.ProviderId)
	if err != nil {
		return nil, invalid("unknown provider")
	}
	return connect.NewResponse(&localv1.BeginCliAuthResponse{
		ProviderId:  plan.ProviderID,
		AuthCommand: plan.AuthCommand,
		DocsUrl:     plan.DocsURL,
		Message:     plan.Message,
	}), nil
}

func settingsProto(item projects.Settings) *localv1.ProjectSettings {
	return &localv1.ProjectSettings{ProjectId: item.ProjectID, Description: item.Description, DefaultEnvironmentId: item.DefaultEnvironmentID, Repository: item.Repository}
}

func nodeProto(item resources.Node) *localv1.Node {
	return &localv1.Node{
		Id: item.ID, ProjectId: item.ProjectID, EnvironmentId: item.EnvironmentID, Name: item.Name,
		Platform: item.Platform, Shell: item.Shell, Status: item.Status, Address: item.Address,
		Cpu: item.CPU, Memory: item.Memory, LastSeenAt: item.LastSeenAt,
		ConnectionKind: item.ConnectionKind, SshUser: item.SSHUser, SshPort: int32(item.SSHPort), SshIdentityFile: item.SSHIdentityFile,
	}
}

func viewFromProto(item *localv1.ArchitectureView) architecture.View {
	members := make([]architecture.ViewMember, 0, len(item.Members))
	for _, member := range item.Members {
		members = append(members, architecture.ViewMember{NodeID: member.NodeId, Name: member.Name, Type: member.Type, Layer: member.Layer})
	}
	return architecture.View{ID: item.Id, ProjectID: item.ProjectId, Name: item.Name, Description: item.Description, Owner: item.Owner, Purpose: item.Purpose, Notes: item.Notes, FilterSummary: item.FilterSummary, Environments: item.Environments, Tags: item.Tags, UpdatedAt: item.UpdatedAt, Members: members}
}

func viewProto(item architecture.View) *localv1.ArchitectureView {
	members := make([]*localv1.ArchitectureViewMember, 0, len(item.Members))
	for _, member := range item.Members {
		members = append(members, &localv1.ArchitectureViewMember{NodeId: member.NodeID, Name: member.Name, Type: member.Type, Layer: member.Layer})
	}
	return &localv1.ArchitectureView{Id: item.ID, ProjectId: item.ProjectID, Name: item.Name, Description: item.Description, Owner: item.Owner, Purpose: item.Purpose, Notes: item.Notes, FilterSummary: item.FilterSummary, Environments: item.Environments, Tags: item.Tags, UpdatedAt: item.UpdatedAt, Members: members}
}

func scheduleProto(item operations.Schedule) *localv1.OperationSchedule {
	return &localv1.OperationSchedule{Id: item.ID, ProjectId: item.ProjectID, EnvironmentId: item.EnvironmentID, Name: item.Name, Description: item.Description, Status: item.Status, WorkflowId: item.WorkflowID, Expression: item.Expression, Timezone: item.Timezone, NextRunAt: item.NextRunAt, LastRunAt: item.LastRunAt}
}

func integrationResourceProto(item integrations.Resource) *localv1.IntegrationResource {
	return &localv1.IntegrationResource{Id: item.ID, ConnectionId: item.ConnectionID, Name: item.Name, Kind: item.Kind, ExternalId: item.ExternalID, EnvironmentId: item.EnvironmentID, Status: item.Status, InventoryResourceId: item.InventoryResourceID}
}

func syncRunProto(item integrations.SyncRun) *localv1.IntegrationSyncRun {
	return &localv1.IntegrationSyncRun{Id: item.ID, ConnectionId: item.ConnectionID, StartedAt: item.StartedAt, FinishedAt: item.FinishedAt, Result: item.Result, Imported: int32(item.Imported), Changed: int32(item.Changed), Message: item.Message}
}
