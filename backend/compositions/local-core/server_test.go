package localcore

import (
	"context"
	"net/http"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"connectrpc.com/connect"

	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
	"github.com/polterware/polterware/backend/platform/gen/polterware/local/v1/localv1connect"
	"github.com/polterware/polterware/backend/providers/seed"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

const testToken = "test-session-token"

func startCore(t *testing.T) (*Core, string) {
	t.Helper()
	store, err := storepkg.New(filepath.Join(t.TempDir(), "core.db"))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = store.Close() })
	core := NewCore(store, testToken)
	listener, url, err := Listen("127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	server := ServeConnect(listener, core)
	t.Cleanup(func() { _ = server.Close() })
	return core, url
}

func withToken[T any](request *connect.Request[T], token string) *connect.Request[T] {
	if token != "" {
		request.Header().Set("Authorization", "Bearer "+token)
	}
	return request
}

func TestMintTokenIsRandomHex(t *testing.T) {
	first, err := MintToken()
	if err != nil {
		t.Fatal(err)
	}
	second, _ := MintToken()
	if len(first) != 64 || first == second {
		t.Fatalf("tokens must be 64 hex chars and unique: %q %q", first, second)
	}
	if strings.Trim(first, "0123456789abcdef") != "" {
		t.Fatalf("token is not lowercase hex: %q", first)
	}
}

func TestListenBindsAnEphemeralLoopbackPort(t *testing.T) {
	listener, url, err := Listen("127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	defer listener.Close()
	if !strings.HasPrefix(url, "http://127.0.0.1:") || strings.HasSuffix(url, ":0") {
		t.Fatalf("url = %q", url)
	}
}

func TestHealthCheckNeedsNoToken(t *testing.T) {
	core, url := startCore(t)
	core.Build = "build-under-test"
	client := localv1connect.NewHealthServiceClient(http.DefaultClient, url)
	response, err := client.Check(context.Background(), connect.NewRequest(&localv1.CheckRequest{}))
	if err != nil {
		t.Fatal(err)
	}
	if response.Msg.Status != "ok" || response.Msg.Build != "build-under-test" || response.Msg.Pid == 0 {
		t.Fatalf("check = %+v", response.Msg)
	}
}

func TestServicesRejectMissingOrWrongToken(t *testing.T) {
	_, url := startCore(t)
	client := localv1connect.NewProjectServiceClient(http.DefaultClient, url)
	for name, token := range map[string]string{"no token": "", "wrong token": "nope", "prefix only": testToken[:5]} {
		_, err := client.ListProjects(context.Background(), withToken(connect.NewRequest(&localv1.ListProjectsRequest{}), token))
		if connect.CodeOf(err) != connect.CodeUnauthenticated {
			t.Fatalf("%s: code = %v, err = %v; want unauthenticated", name, connect.CodeOf(err), err)
		}
	}
}

func TestServicesAcceptTheSessionToken(t *testing.T) {
	_, url := startCore(t)
	client := localv1connect.NewProjectServiceClient(http.DefaultClient, url)
	response, err := client.ListProjects(context.Background(), withToken(connect.NewRequest(&localv1.ListProjectsRequest{}), testToken))
	if err != nil {
		t.Fatal(err)
	}
	ids := map[string]bool{}
	for _, project := range response.Msg.Projects {
		ids[project.Id] = true
	}
	if !ids[seed.ProjectID] {
		t.Fatalf("a fresh core must seed defaults, got %v", ids)
	}
}

func TestMCPEndpointRequiresTheToken(t *testing.T) {
	_, url := startCore(t)
	body := `{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"cursor","version":"1"}}}`

	unauthorized, err := http.Post(url+"/mcp", "application/json", strings.NewReader(body))
	if err != nil {
		t.Fatal(err)
	}
	unauthorized.Body.Close()
	if unauthorized.StatusCode != http.StatusUnauthorized {
		t.Fatalf("status without token = %d", unauthorized.StatusCode)
	}

	request, _ := http.NewRequest(http.MethodPost, url+"/mcp", strings.NewReader(body))
	request.Header.Set("Authorization", "Bearer "+testToken)
	response, err := http.DefaultClient.Do(request)
	if err != nil {
		t.Fatal(err)
	}
	response.Body.Close()
	if response.StatusCode != http.StatusOK || response.Header.Get("Mcp-Session-Id") == "" {
		t.Fatalf("status %d, session %q", response.StatusCode, response.Header.Get("Mcp-Session-Id"))
	}
}

func TestShutdownStopsTheCore(t *testing.T) {
	core, url := startCore(t)
	client := localv1connect.NewMaintenanceServiceClient(http.DefaultClient, url)
	if _, err := client.Shutdown(context.Background(), withToken(connect.NewRequest(&localv1.ShutdownRequest{}), testToken)); err != nil {
		t.Fatal(err)
	}
	select {
	case <-core.Done():
	case <-time.After(3 * time.Second):
		t.Fatal("core did not signal done after Shutdown")
	}
	core.Stop() // stopping twice must not panic
}

func TestShutdownRequiresTheToken(t *testing.T) {
	core, url := startCore(t)
	client := localv1connect.NewMaintenanceServiceClient(http.DefaultClient, url)
	_, err := client.Shutdown(context.Background(), connect.NewRequest(&localv1.ShutdownRequest{}))
	if connect.CodeOf(err) != connect.CodeUnauthenticated {
		t.Fatalf("code = %v", connect.CodeOf(err))
	}
	select {
	case <-core.Done():
		t.Fatal("an unauthenticated Shutdown must not stop the core")
	case <-time.After(300 * time.Millisecond):
	}
}

func TestIdleTracksActivity(t *testing.T) {
	core, url := startCore(t)
	if core.Idle(time.Hour) {
		t.Fatal("a just-started core is not idle")
	}
	time.Sleep(30 * time.Millisecond)
	if !core.Idle(10 * time.Millisecond) {
		t.Fatal("a core with no activity must become idle")
	}
	client := localv1connect.NewProjectServiceClient(http.DefaultClient, url)
	if _, err := client.ListProjects(context.Background(), withToken(connect.NewRequest(&localv1.ListProjectsRequest{}), testToken)); err != nil {
		t.Fatal(err)
	}
	if core.Idle(time.Second) {
		t.Fatal("an authenticated call must reset the idle clock")
	}
}

func TestUnauthenticatedCallsDoNotKeepTheCoreAlive(t *testing.T) {
	core, url := startCore(t)
	time.Sleep(30 * time.Millisecond)
	client := localv1connect.NewProjectServiceClient(http.DefaultClient, url)
	_, _ = client.ListProjects(context.Background(), connect.NewRequest(&localv1.ListProjectsRequest{}))
	if !core.Idle(10 * time.Millisecond) {
		t.Fatal("a rejected call must not count as activity")
	}
}

func TestStoreHealthIsReportedOverRPC(t *testing.T) {
	_, url := startCore(t)
	client := localv1connect.NewMaintenanceServiceClient(http.DefaultClient, url)
	response, err := client.GetStoreHealth(context.Background(), withToken(connect.NewRequest(&localv1.GetStoreHealthRequest{}), testToken))
	if err != nil {
		t.Fatal(err)
	}
	if response.Msg.Quarantined {
		t.Fatalf("fresh store must be healthy: %+v", response.Msg)
	}
}
