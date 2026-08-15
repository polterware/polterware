package integration

import (
	"connectrpc.com/connect"
	"context"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
	"github.com/polterware/polterware/backend/platform/gen/polterware/local/v1/localv1connect"
	"testing"
)

func TestOperationCreatesQueryableLog(t *testing.T) {
	endpoint, token, cleanup := startConnectCore(t)
	defer cleanup()
	operations := localv1connect.NewOperationServiceClient(authClient(token), endpoint)
	created, err := operations.RequestOperation(context.Background(), connect.NewRequest(&localv1.RequestOperationRequest{Type: "command", ProjectId: "project-local"}))
	if err != nil {
		t.Fatal(err)
	}
	logs := localv1connect.NewLogServiceClient(authClient(token), endpoint)
	response, err := logs.QueryLogs(context.Background(), connect.NewRequest(&localv1.QueryLogsRequest{OperationId: created.Msg.Operation.Id, Limit: 10}))
	if err != nil {
		t.Fatal(err)
	}
	if len(response.Msg.Chunks) != 1 {
		t.Fatalf("got %d chunks", len(response.Msg.Chunks))
	}
}
