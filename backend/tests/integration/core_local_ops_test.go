package integration

import (
	"context"
	"net/http"
	"path/filepath"
	"testing"

	"connectrpc.com/connect"
	localcore "github.com/polterware/polterware/backend/compositions/local-core"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
	"github.com/polterware/polterware/backend/platform/gen/polterware/local/v1/localv1connect"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func startConnectCore(t *testing.T) (endpoint, token string, cleanup func()) {
	t.Helper()
	store, err := storepkg.New(filepath.Join(t.TempDir(), "core.db"))
	if err != nil {
		t.Fatal(err)
	}
	token, err = localcore.MintToken()
	if err != nil {
		t.Fatal(err)
	}
	listener, endpoint, err := localcore.Listen("127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	server := localcore.ServeConnect(listener, localcore.NewCore(store, token))
	return endpoint, token, func() { _ = server.Close(); _ = store.Close() }
}

type roundTripperFunc func(*http.Request) (*http.Response, error)

func (function roundTripperFunc) RoundTrip(request *http.Request) (*http.Response, error) {
	return function(request)
}

func authClient(token string) *http.Client {
	return &http.Client{Transport: roundTripperFunc(func(request *http.Request) (*http.Response, error) {
		request.Header.Set("Authorization", "Bearer "+token)
		return http.DefaultTransport.RoundTrip(request)
	})}
}

func TestLocalCoreQueuesDirectOperation(t *testing.T) {
	endpoint, token, cleanup := startConnectCore(t)
	defer cleanup()
	client := localv1connect.NewOperationServiceClient(authClient(token), endpoint)
	response, err := client.RequestOperation(context.Background(), connect.NewRequest(&localv1.RequestOperationRequest{Type: "command", ProjectId: "project-local"}))
	if err != nil {
		t.Fatal(err)
	}
	if response.Msg.Operation.Status != "queued" {
		t.Fatalf("got %s", response.Msg.Operation.Status)
	}
}

func TestUnauthenticatedConnect(t *testing.T) {
	endpoint, _, cleanup := startConnectCore(t)
	defer cleanup()
	client := localv1connect.NewHealthServiceClient(http.DefaultClient, endpoint)
	if _, err := client.Check(context.Background(), connect.NewRequest(&localv1.CheckRequest{})); err == nil {
		t.Fatal("expected unauthenticated")
	}
}
