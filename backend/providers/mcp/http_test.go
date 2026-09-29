package mcp

import (
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/polterware/polterware/backend/domains/agentrun"
	"github.com/polterware/polterware/backend/providers/demoseed"
)

type rpcReply struct {
	Result json.RawMessage `json:"result"`
	Error  *rpcError       `json:"error"`
}

func newSessions(t *testing.T) (*Sessions, *httptest.Server) {
	t.Helper()
	sessions := NewSessions(testServer(t).Store)
	ts := httptest.NewServer(sessions)
	t.Cleanup(ts.Close)
	return sessions, ts
}

func post(t *testing.T, url, session, body string) (*http.Response, []byte) {
	t.Helper()
	req, err := http.NewRequest(http.MethodPost, url, strings.NewReader(body))
	if err != nil {
		t.Fatal(err)
	}
	if session != "" {
		req.Header.Set(SessionHeader, session)
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	data, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatal(err)
	}
	return resp, data
}

func initialize(t *testing.T, url, client string) string {
	t.Helper()
	body := `{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"` + client + `","version":"1.2.3"}}}`
	resp, _ := post(t, url, "", body)
	id := resp.Header.Get(SessionHeader)
	if resp.StatusCode != http.StatusOK || id == "" {
		t.Fatalf("initialize: status %d, session %q", resp.StatusCode, id)
	}
	return id
}

func toolText(t *testing.T, raw []byte) (string, bool) {
	t.Helper()
	var reply rpcReply
	if err := json.Unmarshal(raw, &reply); err != nil || reply.Error != nil {
		t.Fatalf("unexpected reply %s (%v)", raw, err)
	}
	var result struct {
		Content []struct {
			Text string `json:"text"`
		} `json:"content"`
		IsError bool `json:"isError"`
	}
	if err := json.Unmarshal(reply.Result, &result); err != nil || len(result.Content) == 0 {
		t.Fatalf("unexpected tool result %s (%v)", raw, err)
	}
	return result.Content[0].Text, result.IsError
}

func TestHTTPInitializeOpensSessionAndListsTools(t *testing.T) {
	sessions, ts := newSessions(t)
	id := initialize(t, ts.URL, "cursor")

	resp, body := post(t, ts.URL, id, `{"jsonrpc":"2.0","id":2,"method":"tools/list"}`)
	if resp.StatusCode != http.StatusOK || resp.Header.Get("Content-Type") != "application/json" {
		t.Fatalf("status %d, content-type %q", resp.StatusCode, resp.Header.Get("Content-Type"))
	}
	var reply rpcReply
	if err := json.Unmarshal(body, &reply); err != nil || reply.Error != nil {
		t.Fatalf("reply %s (%v)", body, err)
	}
	if !strings.Contains(string(reply.Result), ToolPolterApply) {
		t.Fatalf("tools/list is missing %s: %s", ToolPolterApply, reply.Result)
	}

	live := sessions.List()
	if len(live) != 1 || live[0].ID != id || live[0].ClientName != "cursor" || live[0].ClientVersion != "1.2.3" {
		t.Fatalf("sessions = %+v", live)
	}
}

func TestHTTPRequiresKnownSession(t *testing.T) {
	_, ts := newSessions(t)
	for name, session := range map[string]string{"missing header": "", "unknown id": "01NOSUCHSESSION"} {
		resp, _ := post(t, ts.URL, session, `{"jsonrpc":"2.0","id":2,"method":"tools/list"}`)
		if resp.StatusCode != http.StatusNotFound {
			t.Fatalf("%s: status %d; want 404 so the client initializes again", name, resp.StatusCode)
		}
	}
}

func TestHTTPNotificationsReturnAccepted(t *testing.T) {
	_, ts := newSessions(t)
	id := initialize(t, ts.URL, "codex")
	resp, body := post(t, ts.URL, id, `{"jsonrpc":"2.0","method":"notifications/initialized"}`)
	if resp.StatusCode != http.StatusAccepted || len(body) != 0 {
		t.Fatalf("status %d, body %q", resp.StatusCode, body)
	}
}

func TestHTTPMalformedJSONIsAParseError(t *testing.T) {
	_, ts := newSessions(t)
	id := initialize(t, ts.URL, "codex")
	resp, body := post(t, ts.URL, id, `{not json`)
	var reply rpcReply
	if err := json.Unmarshal(body, &reply); err != nil {
		t.Fatalf("body %s: %v", body, err)
	}
	if resp.StatusCode != http.StatusOK || reply.Error == nil || reply.Error.Code != -32700 {
		t.Fatalf("status %d, reply %s", resp.StatusCode, body)
	}
}

func TestHTTPUnknownMethodIsAnError(t *testing.T) {
	_, ts := newSessions(t)
	id := initialize(t, ts.URL, "codex")
	_, body := post(t, ts.URL, id, `{"jsonrpc":"2.0","id":5,"method":"no/such"}`)
	var reply rpcReply
	if err := json.Unmarshal(body, &reply); err != nil || reply.Error == nil || !strings.Contains(reply.Error.Message, "method not found") {
		t.Fatalf("reply %s (%v)", body, err)
	}
}

func TestHTTPToolErrorsAreReportedInBandWithIsError(t *testing.T) {
	_, ts := newSessions(t)
	id := initialize(t, ts.URL, "codex")
	_, body := post(t, ts.URL, id, `{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"no.such.tool","arguments":{}}}`)
	text, isError := toolText(t, body)
	if !isError || text == "" {
		t.Fatalf("an unknown tool must set isError, got %q (isError=%v)", text, isError)
	}
}

func TestHTTPRejectsUnsupportedMethods(t *testing.T) {
	_, ts := newSessions(t)
	resp, err := http.Get(ts.URL)
	if err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	if resp.StatusCode != http.StatusMethodNotAllowed || resp.Header.Get("Allow") != "POST, DELETE" {
		t.Fatalf("status %d, Allow %q", resp.StatusCode, resp.Header.Get("Allow"))
	}
}

func TestHTTPDeleteClosesSession(t *testing.T) {
	sessions, ts := newSessions(t)
	id := initialize(t, ts.URL, "cursor")

	req, _ := http.NewRequest(http.MethodDelete, ts.URL, nil)
	req.Header.Set(SessionHeader, id)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	if resp.StatusCode != http.StatusNoContent {
		t.Fatalf("status %d", resp.StatusCode)
	}
	if len(sessions.List()) != 0 {
		t.Fatalf("session still live: %+v", sessions.List())
	}
	if again, _ := post(t, ts.URL, id, `{"jsonrpc":"2.0","id":2,"method":"ping"}`); again.StatusCode != http.StatusNotFound {
		t.Fatalf("closed session must 404, got %d", again.StatusCode)
	}
}

func TestHTTPSessionsAreIsolated(t *testing.T) {
	sessions, ts := newSessions(t)
	first := initialize(t, ts.URL, "claude-code")
	second := initialize(t, ts.URL, "codex")
	if first == second {
		t.Fatal("sessions must get distinct ids")
	}
	live := sessions.List()
	if len(live) != 2 || live[0].ClientName != "claude-code" || live[1].ClientName != "codex" {
		t.Fatalf("sessions must list oldest first: %+v", live)
	}
}

func TestHTTPWriteJoinsSessionRunAndCloseAllCompletesIt(t *testing.T) {
	sessions, ts := newSessions(t)
	id := initialize(t, ts.URL, "claude-code")
	seeded, _ := sessions.Store.ListAgentRuns(demoseed.ProjectID, 50)

	args, _ := json.Marshal(map[string]any{
		"name": ToolPolterApply,
		"arguments": map[string]any{
			"project_id": demoseed.ProjectID,
			"text":       polterRoadmap,
		},
	})
	_, body := post(t, ts.URL, id, `{"jsonrpc":"2.0","id":7,"method":"tools/call","params":`+string(args)+`}`)
	if text, isError := toolText(t, body); isError {
		t.Fatalf("apply failed: %s", text)
	}

	known := map[string]bool{}
	for _, run := range seeded {
		known[run.ID] = true
	}
	all, _ := sessions.Store.ListAgentRuns(demoseed.ProjectID, 50)
	var opened []agentrun.Run
	for _, run := range all {
		if !known[run.ID] {
			opened = append(opened, run)
		}
	}
	if len(opened) != 1 || opened[0].Status != agentrun.StatusOpen || opened[0].AgentLabel != "claude-code" {
		t.Fatalf("session run = %+v", opened)
	}

	sessions.CloseAll()
	run, err := sessions.Store.GetAgentRun(opened[0].ID)
	if err != nil || run.Status != agentrun.StatusCompleted {
		t.Fatalf("run after CloseAll = %+v, %v", run, err)
	}
	if len(sessions.List()) != 0 {
		t.Fatal("CloseAll must drop every session")
	}
}
