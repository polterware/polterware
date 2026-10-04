package spike_test

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/polterware/polterware/spikes/mcp/handserver"
	"github.com/polterware/polterware/spikes/mcp/internal/guard"
	"github.com/polterware/polterware/spikes/mcp/internal/mem"
	"github.com/polterware/polterware/spikes/mcp/proxy"
	"github.com/polterware/polterware/spikes/mcp/sdkserver"
)

func TestSDKAndHandRoll(t *testing.T) {
	sdkURL, token := serveMCP(t, sdkserver.New(mem.New()).Handler())
	handURL, _ := serveMCP(t, handserver.NewHTTP(handserver.New(mem.New())))

	t.Run("sdk", func(t *testing.T) {
		exercise(t, sdkURL, token, true)
	})
	t.Run("hand", func(t *testing.T) {
		exercise(t, handURL, token, true)
	})

	sdkBytes := toolsListBytes(t, sdkURL, token)
	handBytes := toolsListBytes(t, handURL, token)
	t.Logf("tools/list bytes sdk=%d hand=%d", sdkBytes, handBytes)
	if handBytes > 6*1024 {
		t.Fatalf("hand-rolled tools/list is %d bytes", handBytes)
	}
	if sdkBytes == 0 || handBytes == 0 {
		t.Fatal("empty tools/list")
	}
}

func TestProxy(t *testing.T) {
	base, token := serveMCP(t, handserver.NewHTTP(handserver.New(mem.New())))
	in := strings.NewReader(strings.Join([]string{
		`{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","clientInfo":{"name":"proxy-test","version":"0"}}}`,
		`{"jsonrpc":"2.0","method":"notifications/initialized"}`,
		`{"jsonrpc":"2.0","id":2,"method":"tools/list"}`,
		`{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"create_memories","arguments":{"items":[{"title":"Milk"}]}}}`,
	}, "\n") + "\n")
	var out bytes.Buffer
	if err := proxy.Run(context.Background(), base+"/mcp", token, in, &out); err != nil {
		t.Fatal(err)
	}
	lines := nonEmpty(out.String())
	if len(lines) != 3 {
		t.Fatalf("proxy lines = %d\n%s", len(lines), out.String())
	}
	if !strings.Contains(lines[2], `"created 1"`) && !strings.Contains(lines[2], `"ok":true`) {
		t.Fatalf("create via proxy = %s", lines[2])
	}
}

func TestRebinding(t *testing.T) {
	base, token := serveMCP(t, sdkserver.New(mem.New()).Handler())
	body := `{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"curl","version":"0"}}}`
	cases := []struct {
		name   string
		host   string
		origin string
		auth   string
		want   int
	}{
		{name: "bad host", host: "evil.example", auth: "Bearer " + token, want: http.StatusForbidden},
		{name: "bad origin", origin: "https://evil.example", auth: "Bearer " + token, want: http.StatusForbidden},
		{name: "missing token", want: http.StatusUnauthorized},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			req, err := http.NewRequest(http.MethodPost, base+"/mcp", strings.NewReader(body))
			if err != nil {
				t.Fatal(err)
			}
			req.Header.Set("Content-Type", "application/json")
			if tc.host != "" {
				req.Host = tc.host
			}
			if tc.origin != "" {
				req.Header.Set("Origin", tc.origin)
			}
			if tc.auth != "" {
				req.Header.Set("Authorization", tc.auth)
			}
			res, err := http.DefaultClient.Do(req)
			if err != nil {
				t.Fatal(err)
			}
			defer res.Body.Close()
			_, _ = io.Copy(io.Discard, res.Body)
			if res.StatusCode != tc.want {
				t.Fatalf("status = %d, want %d", res.StatusCode, tc.want)
			}
			if res.Header.Get("Access-Control-Allow-Origin") != "" {
				t.Fatal("CORS header")
			}
		})
	}
}

func TestLoopbackOnly(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	err := guard.Serve(ctx, "0.0.0.0:0", "spike-token", http.NotFoundHandler())
	if err == nil || !strings.Contains(err.Error(), "loopback") {
		t.Fatalf("err = %v", err)
	}
}

func exercise(t *testing.T, base, token string, wantProgress bool) {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	var notesMu sync.Mutex
	var notes []string
	client := mcp.NewClient(&mcp.Implementation{Name: "spike-client", Version: "0"}, &mcp.ClientOptions{
		ProgressNotificationHandler: func(_ context.Context, req *mcp.ProgressNotificationClientRequest) {
			notesMu.Lock()
			notes = append(notes, req.Params.Message)
			notesMu.Unlock()
		},
	})
	session, err := client.Connect(ctx, &mcp.StreamableClientTransport{
		Endpoint:   base + "/mcp",
		HTTPClient: &http.Client{Transport: bearerTrip{token: token}},
	}, &mcp.ClientSessionOptions{ProtocolVersion: "2025-11-25"})
	if err != nil {
		t.Fatalf("connect: %v", err)
	}
	defer session.Close()

	listed, err := session.ListTools(ctx, nil)
	if err != nil {
		t.Fatal(err)
	}
	got := map[string]bool{}
	for _, tool := range listed.Tools {
		got[tool.Name] = true
		if len(tool.Description) > 300 {
			t.Errorf("%s description is %d chars", tool.Name, len(tool.Description))
		}
	}
	for _, name := range []string{"get_current_time", "list_memories", "create_memories"} {
		if !got[name] {
			t.Errorf("missing %s", name)
		}
	}

	created, err := session.CallTool(ctx, &mcp.CallToolParams{
		Name:      "create_memories",
		Arguments: map[string]any{"items": []any{map[string]any{"title": "Milk", "note": "oat"}}},
		Meta:      mcp.Meta{"progressToken": "t1"},
	})
	if err != nil {
		t.Fatal(err)
	}
	if created.IsError {
		t.Fatalf("create error: %+v", created)
	}
	raw, _ := json.Marshal(created.StructuredContent)
	if !bytes.Contains(raw, []byte(`"mcp:spike-client"`)) {
		t.Fatalf("actor missing: %s", raw)
	}

	page, err := session.CallTool(ctx, &mcp.CallToolParams{Name: "list_memories", Arguments: map[string]any{}})
	if err != nil {
		t.Fatal(err)
	}
	pageRaw, _ := json.Marshal(page.StructuredContent)
	if !bytes.Contains(pageRaw, []byte(`"Milk"`)) || !bytes.Contains(pageRaw, []byte(`"total"`)) {
		t.Fatalf("list = %s", pageRaw)
	}

	resources, err := session.ListResources(ctx, nil)
	if err != nil {
		t.Fatal(err)
	}
	if len(resources.Resources) != 1 || resources.Resources[0].URI != "sparky://today" {
		t.Fatalf("resources = %+v", resources.Resources)
	}
	prompts, err := session.ListPrompts(ctx, nil)
	if err != nil {
		t.Fatal(err)
	}
	if len(prompts.Prompts) != 1 || prompts.Prompts[0].Name != "capture" {
		t.Fatalf("prompts = %+v", prompts.Prompts)
	}

	notesMu.Lock()
	gotNotes := len(notes)
	notesMu.Unlock()
	if wantProgress && gotNotes == 0 {
		t.Fatal("progress notification was not delivered")
	}
}

func toolsListBytes(t *testing.T, base, token string) int {
	t.Helper()
	initBody := `{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"measure","version":"0"}}}`
	status, session, body := postMCP(t, base, token, "", initBody)
	if status != http.StatusOK {
		t.Fatalf("initialize status %d body %s", status, body)
	}
	status, _, body = postMCP(t, base, token, session, `{"jsonrpc":"2.0","id":2,"method":"tools/list"}`)
	if status != http.StatusOK {
		t.Fatalf("tools/list status %d body %s", status, body)
	}
	return len(body)
}

func serveMCP(t *testing.T, mcpHandler http.Handler) (string, string) {
	t.Helper()
	const token = "spike-token"
	g := &guard.Guard{Token: token}
	mux := http.NewServeMux()
	mux.Handle("/mcp", mcpHandler)
	ts := httptest.NewServer(g.Handler(mux))
	t.Cleanup(ts.Close)
	g.Port = strings.TrimPrefix(ts.URL, "http://127.0.0.1:")
	if g.Port == ts.URL {
		t.Fatalf("test server is not ipv4 loopback: %s", ts.URL)
	}
	return ts.URL, token
}

func postMCP(t *testing.T, base, token, session, body string) (int, string, []byte) {
	t.Helper()
	req, err := http.NewRequest(http.MethodPost, base+"/mcp", strings.NewReader(body))
	if err != nil {
		t.Fatal(err)
	}
	req.Host = strings.TrimPrefix(base, "http://")
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Accept", "application/json, text/event-stream")
	req.Header.Set("Authorization", "Bearer "+token)
	if session != "" {
		req.Header.Set("Mcp-Session-Id", session)
	}
	res, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	defer res.Body.Close()
	got, err := io.ReadAll(res.Body)
	if err != nil {
		t.Fatal(err)
	}
	return res.StatusCode, res.Header.Get("Mcp-Session-Id"), got
}

type bearerTrip struct{ token string }

func (b bearerTrip) RoundTrip(req *http.Request) (*http.Response, error) {
	cloned := req.Clone(req.Context())
	cloned.Header.Set("Authorization", "Bearer "+b.token)
	return http.DefaultTransport.RoundTrip(cloned)
}

func nonEmpty(s string) []string {
	var out []string
	sc := bufio.NewScanner(strings.NewReader(s))
	for sc.Scan() {
		if line := strings.TrimSpace(sc.Text()); line != "" {
			out = append(out, line)
		}
	}
	return out
}
