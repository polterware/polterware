package localcore

import (
	"context"
	"crypto/rand"
	"crypto/subtle"
	"encoding/hex"
	"fmt"
	"net"
	"net/http"
	"strings"
	"sync"
	"time"

	"connectrpc.com/connect"
	"golang.org/x/net/http2"
	"golang.org/x/net/http2/h2c"

	"github.com/polterware/polterware/backend/domains/integrations"
	"github.com/polterware/polterware/backend/domains/knowledge"
	"github.com/polterware/polterware/backend/domains/operations"
	"github.com/polterware/polterware/backend/domains/projects"
	"github.com/polterware/polterware/backend/domains/resources"
	"github.com/polterware/polterware/backend/domains/workflows"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
	"github.com/polterware/polterware/backend/platform/gen/polterware/local/v1/localv1connect"
	"github.com/polterware/polterware/backend/platform/observability"
	processpkg "github.com/polterware/polterware/backend/providers/process"
	"github.com/polterware/polterware/backend/providers/seed"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
	terminalpkg "github.com/polterware/polterware/backend/providers/terminal"
)

type Core struct {
	Store          *storepkg.Store
	Token          string
	Started        time.Time
	Runner         *processpkg.Manager
	runnerOnce     sync.Once
	Terminals      *terminalpkg.Manager
	terminalOnce   sync.Once
	scheduleMu     sync.Mutex
	scheduleCancel context.CancelFunc
	scheduleFiring map[string]struct{}
}

func NewCore(store *storepkg.Store, token string) *Core {
	_ = seed.Defaults(store)
	return &Core{Store: store, Token: token, Started: time.Now().UTC()}
}

func MintToken() (string, error) {
	var value [32]byte
	if _, err := rand.Read(value[:]); err != nil {
		return "", err
	}
	return hex.EncodeToString(value[:]), nil
}

func Listen(address string) (net.Listener, string, error) {
	listener, err := net.Listen("tcp", address)
	if err != nil {
		return nil, "", err
	}
	return listener, "http://" + listener.Addr().String(), nil
}

func (core *Core) authInterceptor() connect.UnaryInterceptorFunc {
	return func(next connect.UnaryFunc) connect.UnaryFunc {
		return func(ctx context.Context, request connect.AnyRequest) (connect.AnyResponse, error) {
			header := request.Header().Get("Authorization")
			candidate := strings.TrimPrefix(header, "Bearer ")
			if !strings.HasPrefix(header, "Bearer ") || subtle.ConstantTimeCompare([]byte(candidate), []byte(core.Token)) != 1 {
				return nil, connect.NewError(connect.CodeUnauthenticated, fmt.Errorf("invalid or missing session token"))
			}
			return next(ctx, request)
		}
	}
}

func ServeConnect(listener net.Listener, core *Core) *http.Server {
	mux := http.NewServeMux()
	options := []connect.HandlerOption{connect.WithInterceptors(core.authInterceptor())}
	register := func(path string, handler http.Handler) { mux.Handle(path, handler) }
	path, handler := localv1connect.NewHealthServiceHandler(core)
	register(path, handler)
	path, handler = localv1connect.NewProjectServiceHandler(core, options...)
	register(path, handler)
	path, handler = localv1connect.NewArchitectureServiceHandler(core, options...)
	register(path, handler)
	path, handler = localv1connect.NewNodeServiceHandler(core, options...)
	register(path, handler)
	path, handler = localv1connect.NewKnowledgeServiceHandler(core, options...)
	register(path, handler)
	path, handler = localv1connect.NewOperationServiceHandler(core, options...)
	register(path, handler)
	path, handler = localv1connect.NewWorkflowServiceHandler(core, options...)
	register(path, handler)
	path, handler = localv1connect.NewIntegrationServiceHandler(core, options...)
	register(path, handler)
	path, handler = localv1connect.NewProcessServiceHandler(core, options...)
	register(path, handler)
	path, handler = localv1connect.NewTerminalServiceHandler(core, options...)
	register(path, handler)
	path, handler = localv1connect.NewLogServiceHandler(core, options...)
	register(path, handler)
	path, handler = localv1connect.NewMaintenanceServiceHandler(core, options...)
	register(path, handler)

	server := &http.Server{Handler: h2c.NewHandler(mux, &http2.Server{})}
	go func() { _ = server.Serve(listener) }()
	return server
}

func (core *Core) Check(context.Context, *connect.Request[localv1.CheckRequest]) (*connect.Response[localv1.CheckResponse], error) {
	return connect.NewResponse(&localv1.CheckResponse{Status: "ok", Version: "0.1.0", UptimeSec: time.Since(core.Started).Seconds()}), nil
}

func (core *Core) ListProjects(_ context.Context, request *connect.Request[localv1.ListProjectsRequest]) (*connect.Response[localv1.ListProjectsResponse], error) {
	items, err := core.Store.ListProjects(request.Msg.GetIncludeArchived())
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.Project, 0, len(items))
	for _, item := range items {
		result = append(result, projectProto(item))
	}
	return connect.NewResponse(&localv1.ListProjectsResponse{Projects: result}), nil
}

func (core *Core) SaveProject(_ context.Context, request *connect.Request[localv1.SaveProjectRequest]) (*connect.Response[localv1.SaveProjectResponse], error) {
	item := request.Msg.GetProject()
	if item == nil {
		return nil, invalid("project is required")
	}
	project, err := projects.NewProject(item.Id, item.Name, item.RootPath)
	if err != nil {
		return nil, invalid(err.Error())
	}
	if err := core.Store.SaveProject(project); err != nil {
		return nil, internal(err)
	}
	if _, err := core.Store.GetLayout(project.ID); err != nil {
		if _, err := core.Store.UpdateLayout(project.ID, 0, `{"nodes":[],"edges":[]}`); err != nil {
			return nil, internal(err)
		}
	}
	if _, err := core.Store.GetProjectSettings(project.ID); err != nil {
		if err := core.Store.SaveProjectSettings(projects.Settings{ProjectID: project.ID}); err != nil {
			return nil, internal(err)
		}
	}
	return connect.NewResponse(&localv1.SaveProjectResponse{Project: projectProto(project)}), nil
}

func (core *Core) ArchiveProject(_ context.Context, request *connect.Request[localv1.ArchiveProjectRequest]) (*connect.Response[localv1.ArchiveProjectResponse], error) {
	if request.Msg.Id == "" {
		return nil, invalid("project id is required")
	}
	project, err := core.Store.ArchiveProject(request.Msg.Id)
	if err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.ArchiveProjectResponse{Project: projectProto(project)}), nil
}

func (core *Core) ListEnvironments(_ context.Context, request *connect.Request[localv1.ListEnvironmentsRequest]) (*connect.Response[localv1.ListEnvironmentsResponse], error) {
	items, err := core.Store.ListEnvironments(request.Msg.ProjectId)
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.Environment, 0, len(items))
	for _, item := range items {
		result = append(result, environmentProto(item))
	}
	return connect.NewResponse(&localv1.ListEnvironmentsResponse{Environments: result}), nil
}

func (core *Core) SaveEnvironment(_ context.Context, request *connect.Request[localv1.SaveEnvironmentRequest]) (*connect.Response[localv1.SaveEnvironmentResponse], error) {
	item := request.Msg.GetEnvironment()
	if item == nil {
		return nil, invalid("environment is required")
	}
	environment, err := projects.NewEnvironment(item.Id, item.ProjectId, item.Name, item.Kind)
	if err != nil {
		return nil, invalid(err.Error())
	}
	environment.Protection = item.Protection
	environment.IsDefault = item.IsDefault
	environment.Description = item.Description
	if err := core.Store.SaveEnvironment(environment); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.SaveEnvironmentResponse{Environment: environmentProto(environment)}), nil
}

func (core *Core) DeleteEnvironment(_ context.Context, request *connect.Request[localv1.DeleteEnvironmentRequest]) (*connect.Response[localv1.DeleteEnvironmentResponse], error) {
	if request.Msg.Id == "" || request.Msg.ProjectId == "" {
		return nil, invalid("environment id and project id are required")
	}
	if err := core.Store.DeleteEnvironment(request.Msg.Id, request.Msg.ProjectId); err != nil {
		return nil, invalid(err.Error())
	}
	return connect.NewResponse(&localv1.DeleteEnvironmentResponse{}), nil
}

func (core *Core) GetLayout(_ context.Context, request *connect.Request[localv1.GetLayoutRequest]) (*connect.Response[localv1.GetLayoutResponse], error) {
	item, err := core.Store.GetLayout(request.Msg.ProjectId)
	if err != nil {
		return nil, connect.NewError(connect.CodeNotFound, err)
	}
	return connect.NewResponse(&localv1.GetLayoutResponse{Layout: &localv1.Layout{Id: item.ID, ProjectId: item.ProjectID, Revision: item.Revision, BodyJson: item.BodyJSON}}), nil
}

func (core *Core) UpdateLayout(_ context.Context, request *connect.Request[localv1.UpdateLayoutRequest]) (*connect.Response[localv1.UpdateLayoutResponse], error) {
	item, err := core.Store.UpdateLayout(request.Msg.ProjectId, request.Msg.ExpectedRevision, request.Msg.BodyJson)
	if err != nil {
		return nil, connect.NewError(connect.CodeFailedPrecondition, err)
	}
	return connect.NewResponse(&localv1.UpdateLayoutResponse{Layout: &localv1.Layout{Id: item.ID, ProjectId: item.ProjectID, Revision: item.Revision, BodyJson: item.BodyJSON}}), nil
}

func (core *Core) ListResources(_ context.Context, request *connect.Request[localv1.ListResourcesRequest]) (*connect.Response[localv1.ListResourcesResponse], error) {
	items, err := core.Store.ListResources(request.Msg.ProjectId)
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.Resource, 0, len(items))
	for _, item := range items {
		result = append(result, resourceProto(item))
	}
	return connect.NewResponse(&localv1.ListResourcesResponse{Resources: result}), nil
}

func (core *Core) SaveResource(_ context.Context, request *connect.Request[localv1.SaveResourceRequest]) (*connect.Response[localv1.SaveResourceResponse], error) {
	item := request.Msg.GetResource()
	if item == nil {
		return nil, invalid("resource is required")
	}
	resource := resources.Resource{ID: item.Id, ProjectID: item.ProjectId, EnvironmentID: item.EnvironmentId, Name: item.Name, Type: item.Type, ExternalKey: item.ExternalKey, Source: item.Source, Provider: item.Provider, Status: item.Status, Region: item.Region, Address: item.Address, Tags: item.Tags, Description: item.Description}
	if resource.ID == "" || resource.ProjectID == "" || resource.Name == "" {
		return nil, invalid("resource id, project id, and name are required")
	}
	if err := core.Store.SaveResource(resource); err != nil {
		return nil, connect.NewError(connect.CodeFailedPrecondition, err)
	}
	return connect.NewResponse(&localv1.SaveResourceResponse{Resource: resourceProto(resource)}), nil
}

func (core *Core) SaveKnowledge(_ context.Context, request *connect.Request[localv1.SaveKnowledgeRequest]) (*connect.Response[localv1.SaveKnowledgeResponse], error) {
	item := request.Msg.GetItem()
	if item == nil {
		return nil, invalid("knowledge item is required")
	}
	created, err := knowledge.NewItem(item.Id, item.ProjectId, item.Title, item.Body, item.Kind, item.Tags)
	if err != nil {
		return nil, invalid(err.Error())
	}
	created.Summary = item.Summary
	created.Owner = item.Owner
	created.Status = item.Status
	created.ExtraJSON = item.ExtraJson
	if err := core.Store.SaveKnowledge(created); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.SaveKnowledgeResponse{Item: knowledgeProto(created)}), nil
}

func (core *Core) ListKnowledge(_ context.Context, request *connect.Request[localv1.ListKnowledgeRequest]) (*connect.Response[localv1.ListKnowledgeResponse], error) {
	items, err := core.Store.ListKnowledge(request.Msg.ProjectId, request.Msg.Kind)
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.KnowledgeItem, 0, len(items))
	for _, item := range items {
		result = append(result, knowledgeProto(item))
	}
	return connect.NewResponse(&localv1.ListKnowledgeResponse{Items: result}), nil
}

func (core *Core) DeleteKnowledge(_ context.Context, request *connect.Request[localv1.DeleteKnowledgeRequest]) (*connect.Response[localv1.DeleteKnowledgeResponse], error) {
	if err := core.Store.DeleteKnowledge(request.Msg.Id); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.DeleteKnowledgeResponse{}), nil
}

func (core *Core) RequestOperation(_ context.Context, request *connect.Request[localv1.RequestOperationRequest]) (*connect.Response[localv1.RequestOperationResponse], error) {
	if request.Msg.ProjectId == "" || request.Msg.Type == "" {
		return nil, invalid("project id and type are required")
	}
	correlation := observability.NewCorrelationID()
	now := time.Now().UTC()
	params := request.Msg.Params
	if params == nil {
		params = map[string]string{}
	}
	if request.Msg.Cwd != "" {
		params["cwd"] = request.Msg.Cwd
	}
	operation := &operations.Operation{
		ID:             "op-" + correlation[:12],
		CorrelationID:  correlation,
		ProjectID:      request.Msg.ProjectId,
		EnvironmentID:  request.Msg.EnvironmentId,
		Type:           request.Msg.Type,
		Status:         operations.StatusQueued,
		IdempotencyKey: request.Msg.IdempotencyKey,
		Params:         params,
		RequestedAt:    now,
		Title:          request.Msg.Title,
		Target:         request.Msg.Target,
		Command:        request.Msg.Command,
		StepsJSON:      "[]",
	}
	if err := core.Store.SaveOperation(operation); err != nil {
		return nil, internal(err)
	}
	_, _ = core.Store.AppendLog(operation.ID, "Operation queued for local execution.", false)
	core.enqueueOperationExecution(operation.ID)
	return connect.NewResponse(&localv1.RequestOperationResponse{Operation: operationProto(operation)}), nil
}

func (core *Core) GetOperation(_ context.Context, request *connect.Request[localv1.GetOperationRequest]) (*connect.Response[localv1.GetOperationResponse], error) {
	item, err := core.Store.LoadOperation(request.Msg.Id)
	if err != nil {
		return nil, connect.NewError(connect.CodeNotFound, err)
	}
	return connect.NewResponse(&localv1.GetOperationResponse{Operation: operationProto(item)}), nil
}

func (core *Core) ListOperations(_ context.Context, request *connect.Request[localv1.ListOperationsRequest]) (*connect.Response[localv1.ListOperationsResponse], error) {
	items, err := core.Store.ListOperations(request.Msg.ProjectId, int(request.Msg.PageSize))
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.Operation, 0, len(items))
	for _, item := range items {
		result = append(result, operationProto(item))
	}
	return connect.NewResponse(&localv1.ListOperationsResponse{Operations: result}), nil
}

func (core *Core) CancelOperation(_ context.Context, request *connect.Request[localv1.CancelOperationRequest]) (*connect.Response[localv1.CancelOperationResponse], error) {
	item, err := core.Store.LoadOperation(request.Msg.Id)
	if err != nil {
		return nil, connect.NewError(connect.CodeNotFound, err)
	}
	if !item.IsTerminal() {
		core.processManager().Cancel(item.ID)
		_ = item.Transition(operations.StatusCancelled)
		if err := core.Store.SaveOperation(item); err != nil {
			return nil, internal(err)
		}
		_, _ = core.Store.AppendLog(item.ID, "Operation cancelled.", false)
	}
	return connect.NewResponse(&localv1.CancelOperationResponse{Operation: operationProto(item)}), nil
}

func (core *Core) SaveWorkflow(_ context.Context, request *connect.Request[localv1.SaveWorkflowRequest]) (*connect.Response[localv1.SaveWorkflowResponse], error) {
	item := request.Msg.GetWorkflow()
	if item == nil {
		return nil, invalid("workflow is required")
	}
	workflow := workflowFromProto(item)
	if err := core.Store.SaveWorkflow(workflow); err != nil {
		return nil, invalid(err.Error())
	}
	return connect.NewResponse(&localv1.SaveWorkflowResponse{Workflow: workflowProto(workflow)}), nil
}

func (core *Core) ListWorkflows(_ context.Context, request *connect.Request[localv1.ListWorkflowsRequest]) (*connect.Response[localv1.ListWorkflowsResponse], error) {
	items, err := core.Store.ListWorkflows(request.Msg.ProjectId)
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.Workflow, 0, len(items))
	for _, item := range items {
		result = append(result, workflowProto(item))
	}
	return connect.NewResponse(&localv1.ListWorkflowsResponse{Workflows: result}), nil
}

func (core *Core) RunWorkflow(_ context.Context, request *connect.Request[localv1.RunWorkflowRequest]) (*connect.Response[localv1.RunWorkflowResponse], error) {
	runID, operationIDs, err := core.startWorkflowRun(request.Msg.Id, request.Msg.EnvironmentId)
	if err != nil {
		return nil, err
	}
	return connect.NewResponse(&localv1.RunWorkflowResponse{RunId: runID, Status: "running", OperationIds: operationIDs}), nil
}

func (core *Core) startWorkflowRun(workflowID, environmentID string) (runID string, operationIDs []string, err error) {
	workflow, err := core.Store.GetWorkflow(workflowID)
	if err != nil {
		return "", nil, connect.NewError(connect.CodeNotFound, err)
	}
	correlation := observability.NewCorrelationID()
	operationIDs = make([]string, 0, len(workflow.Steps))
	for index, step := range workflow.Steps {
		params := step.Params
		if params == nil {
			params = map[string]string{}
		}
		command := params["command"]
		operation := &operations.Operation{
			ID:            fmt.Sprintf("op-%s-%d", correlation[:8], index+1),
			CorrelationID: correlation,
			ProjectID:     workflow.ProjectID,
			EnvironmentID: environmentID,
			Type:          step.OperationType,
			Status:        operations.StatusQueued,
			Params:        params,
			RequestedAt:   time.Now().UTC(),
			Title:         step.Name,
			Command:       command,
		}
		if err := core.Store.SaveOperation(operation); err != nil {
			return "", nil, internal(err)
		}
		_, _ = core.Store.AppendLog(operation.ID, "Workflow step queued for local execution.", false)
		operationIDs = append(operationIDs, operation.ID)
		core.enqueueOperationExecution(operation.ID)
	}
	return "run-" + correlation[:12], operationIDs, nil
}

func (core *Core) ListCliProviders(context.Context, *connect.Request[localv1.ListCliProvidersRequest]) (*connect.Response[localv1.ListCliProvidersResponse], error) {
	items := make([]*localv1.CliProvider, 0, len(integrations.CLIProviders))
	for _, item := range integrations.CLIProviders {
		items = append(items, &localv1.CliProvider{Id: item.ID, Name: item.Name, Binary: item.Binary, AuthCommands: item.AuthCommands, Description: item.Description, Category: item.Category})
	}
	return connect.NewResponse(&localv1.ListCliProvidersResponse{Providers: items}), nil
}

func (core *Core) ListIntegrationConnections(_ context.Context, request *connect.Request[localv1.ListIntegrationConnectionsRequest]) (*connect.Response[localv1.ListIntegrationConnectionsResponse], error) {
	items, err := core.Store.ListConnections(request.Msg.ProjectId)
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.IntegrationConnection, 0, len(items))
	for _, item := range items {
		result = append(result, connectionProto(item))
	}
	return connect.NewResponse(&localv1.ListIntegrationConnectionsResponse{Connections: result}), nil
}

func (core *Core) SaveIntegrationConnection(_ context.Context, request *connect.Request[localv1.SaveIntegrationConnectionRequest]) (*connect.Response[localv1.SaveIntegrationConnectionResponse], error) {
	item := request.Msg.GetConnection()
	if item == nil {
		return nil, invalid("connection is required")
	}
	connection := integrations.Connection{ID: item.Id, ProjectID: item.ProjectId, ProviderID: item.ProviderId, Status: item.Status, Label: item.Label, EnvironmentID: item.EnvironmentId, LastSyncAt: item.LastSyncAt, Summary: item.Summary, CLICommand: item.CliCommand, SyncInterval: item.SyncInterval, Freshness: item.Freshness, ErrorMessage: item.ErrorMessage}
	if connection.ID == "" || connection.ProjectID == "" || connection.ProviderID == "" {
		return nil, invalid("connection id, project id, and provider id are required")
	}
	if err := core.Store.SaveConnection(connection); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.SaveIntegrationConnectionResponse{Connection: connectionProto(connection)}), nil
}

func (core *Core) GetProcess(_ context.Context, request *connect.Request[localv1.GetProcessRequest]) (*connect.Response[localv1.GetProcessResponse], error) {
	session, err := core.Store.GetProcess(request.Msg.Id)
	if err != nil {
		return nil, connect.NewError(connect.CodeNotFound, err)
	}
	return connect.NewResponse(&localv1.GetProcessResponse{Session: processProto(session)}), nil
}

func (core *Core) ListProcesses(_ context.Context, request *connect.Request[localv1.ListProcessesRequest]) (*connect.Response[localv1.ListProcessesResponse], error) {
	items, err := core.Store.ListProcesses(request.Msg.ProjectId)
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.ProcessSession, 0, len(items))
	for _, item := range items {
		result = append(result, processProto(item))
	}
	return connect.NewResponse(&localv1.ListProcessesResponse{Sessions: result}), nil
}

func (core *Core) SubscribeLogs(_ context.Context, request *connect.Request[localv1.SubscribeLogsRequest], stream *connect.ServerStream[localv1.SubscribeLogsResponse]) error {
	items, err := core.Store.QueryLogs(request.Msg.OperationId, 1000)
	if err != nil {
		return internal(err)
	}
	for _, item := range items {
		if item.Sequence <= request.Msg.SinceSequence {
			continue
		}
		if err := stream.Send(logProto(item)); err != nil {
			return err
		}
	}
	return nil
}

func (core *Core) QueryLogs(_ context.Context, request *connect.Request[localv1.QueryLogsRequest]) (*connect.Response[localv1.QueryLogsResponse], error) {
	items, err := core.Store.QueryLogs(request.Msg.OperationId, int(request.Msg.Limit))
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.SubscribeLogsResponse, 0, len(items))
	for _, item := range items {
		result = append(result, logProto(item))
	}
	return connect.NewResponse(&localv1.QueryLogsResponse{Chunks: result}), nil
}

func (core *Core) ResetLocalStore(context.Context, *connect.Request[localv1.ResetLocalStoreRequest]) (*connect.Response[localv1.ResetLocalStoreResponse], error) {
	if err := core.Store.Reset(); err != nil {
		return nil, internal(err)
	}
	if err := seed.Defaults(core.Store); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.ResetLocalStoreResponse{Status: "reset"}), nil
}

func (core *Core) GetStoreHealth(context.Context, *connect.Request[localv1.GetStoreHealthRequest]) (*connect.Response[localv1.GetStoreHealthResponse], error) {
	quarantined, message, err := core.Store.StoreHealth()
	if err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.GetStoreHealthResponse{Quarantined: quarantined, Message: message}), nil
}

func projectProto(item projects.Project) *localv1.Project {
	archivedAt := ""
	if !item.ArchivedAt.IsZero() {
		archivedAt = item.ArchivedAt.Format(time.RFC3339)
	}
	status := item.Status
	if status == "" {
		status = projects.StatusActive
	}
	return &localv1.Project{
		Id:         item.ID,
		Name:       item.Name,
		RootPath:   item.RootPath,
		CreatedAt:  item.CreatedAt.Format(time.RFC3339),
		UpdatedAt:  item.UpdatedAt.Format(time.RFC3339),
		Status:     status,
		ArchivedAt: archivedAt,
	}
}
func environmentProto(item projects.Environment) *localv1.Environment {
	return &localv1.Environment{Id: item.ID, ProjectId: item.ProjectID, Name: item.Name, Kind: item.Kind, Protection: item.Protection, IsDefault: item.IsDefault, Description: item.Description}
}
func resourceProto(item resources.Resource) *localv1.Resource {
	return &localv1.Resource{Id: item.ID, ProjectId: item.ProjectID, EnvironmentId: item.EnvironmentID, Name: item.Name, Type: item.Type, ExternalKey: item.ExternalKey, Source: item.Source, Provider: item.Provider, Status: item.Status, Region: item.Region, Address: item.Address, Tags: item.Tags, Description: item.Description}
}
func knowledgeProto(item knowledge.Item) *localv1.KnowledgeItem {
	return &localv1.KnowledgeItem{Id: item.ID, ProjectId: item.ProjectID, Title: item.Title, Body: item.Body, Kind: item.Kind, Tags: item.Tags, Summary: item.Summary, Owner: item.Owner, Status: item.Status, ExtraJson: item.ExtraJSON, CreatedAt: item.CreatedAt.Format(time.RFC3339), UpdatedAt: item.UpdatedAt.Format(time.RFC3339)}
}
func operationProto(item *operations.Operation) *localv1.Operation {
	return &localv1.Operation{Id: item.ID, CorrelationId: item.CorrelationID, ProjectId: item.ProjectID, EnvironmentId: item.EnvironmentID, Type: item.Type, Status: string(item.Status), ResultSummary: item.ResultSummary, ErrorMessage: item.ErrorMessage, RequestedAt: item.RequestedAt.Format(time.RFC3339), StartedAt: formatTime(item.StartedAt), FinishedAt: formatTime(item.FinishedAt), Params: item.Params, Title: item.Title, Target: item.Target, Command: item.Command, StepsJson: item.StepsJSON}
}
func workflowFromProto(item *localv1.Workflow) workflows.Workflow {
	steps := make([]workflows.Step, 0, len(item.Steps))
	for _, step := range item.Steps {
		steps = append(steps, workflows.Step{ID: step.Id, Name: step.Name, OperationType: step.OperationType, Params: step.Params})
	}
	return workflows.Workflow{ID: item.Id, ProjectID: item.ProjectId, Name: item.Name, Status: item.Status, Steps: steps, Description: item.Description, Trigger: item.Trigger, LastRunAt: item.LastRunAt}
}
func workflowProto(item workflows.Workflow) *localv1.Workflow {
	steps := make([]*localv1.WorkflowStep, 0, len(item.Steps))
	for _, step := range item.Steps {
		steps = append(steps, &localv1.WorkflowStep{Id: step.ID, Name: step.Name, OperationType: step.OperationType, Params: step.Params})
	}
	return &localv1.Workflow{Id: item.ID, ProjectId: item.ProjectID, Name: item.Name, Status: item.Status, Steps: steps, Description: item.Description, Trigger: item.Trigger, LastRunAt: item.LastRunAt}
}
func connectionProto(item integrations.Connection) *localv1.IntegrationConnection {
	return &localv1.IntegrationConnection{Id: item.ID, ProjectId: item.ProjectID, ProviderId: item.ProviderID, Status: item.Status, Label: item.Label, EnvironmentId: item.EnvironmentID, LastSyncAt: item.LastSyncAt, Summary: item.Summary, CliCommand: item.CLICommand, SyncInterval: item.SyncInterval, Freshness: item.Freshness, ErrorMessage: item.ErrorMessage}
}
func processProto(item storepkg.ProcessSession) *localv1.ProcessSession {
	return &localv1.ProcessSession{Id: item.ID, ProjectId: item.ProjectID, EnvironmentId: item.EnvironmentID, Command: item.Command, Status: item.Status, Name: item.Name, NodeId: item.NodeID, Pid: int32(item.PID), StartedAt: item.StartedAt}
}
func logProto(item storepkg.LogChunk) *localv1.SubscribeLogsResponse {
	return &localv1.SubscribeLogsResponse{OperationId: item.OperationID, Sequence: item.Sequence, Body: item.Body, Truncated: item.Truncated, At: item.CreatedAt.Format(time.RFC3339)}
}
func formatTime(value *time.Time) string {
	if value == nil {
		return ""
	}
	return value.Format(time.RFC3339)
}
func invalid(message string) error {
	return connect.NewError(connect.CodeInvalidArgument, fmt.Errorf("%s", message))
}
func internal(err error) error { return connect.NewError(connect.CodeInternal, err) }
