// Package handserver is the hand-rolled JSON-RPC spike, shaped like backend/providers/mcp.
package handserver

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"strings"
	"time"

	"github.com/polterware/polterware/spikes/mcp/internal/catalog"
	"github.com/polterware/polterware/spikes/mcp/internal/mem"
)

type rpcRequest struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      json.RawMessage `json:"id"`
	Method  string          `json:"method"`
	Params  json.RawMessage `json:"params"`
}

type rpcResponse struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      json.RawMessage `json:"id,omitempty"`
	Result  any             `json:"result,omitempty"`
	Error   *rpcError       `json:"error,omitempty"`
}

type rpcError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}

type session struct {
	id            string
	clientName    string
	clientVersion string
	protocol      string
	publisher     *publisher
}

// Server dispatches JSON-RPC for one in-memory store.
type Server struct {
	store *mem.Store
}

// New returns a hand-rolled server.
func New(store *mem.Store) *Server { return &Server{store: store} }

func (s *Server) handle(sess *session, line []byte, publish func([]byte)) (body []byte, ok bool) {
	var req rpcRequest
	if err := json.Unmarshal(line, &req); err != nil || len(line) == 0 || line[0] == '[' {
		return mustJSON(rpcResponse{JSONRPC: "2.0", Error: &rpcError{Code: -32700, Message: "parse error"}}), true
	}
	if req.JSONRPC != "" && req.JSONRPC != "2.0" {
		return mustJSON(rpcResponse{JSONRPC: "2.0", ID: req.ID, Error: &rpcError{Code: -32600, Message: "invalid request"}}), req.ID != nil
	}
	if req.ID == nil && strings.HasPrefix(req.Method, "notifications/") {
		return nil, false
	}
	result, rpcErr := s.dispatch(sess, req, publish)
	if req.ID == nil {
		return nil, false
	}
	if rpcErr != nil {
		return mustJSON(rpcResponse{JSONRPC: "2.0", ID: req.ID, Error: rpcErr}), true
	}
	return mustJSON(rpcResponse{JSONRPC: "2.0", ID: req.ID, Result: result}), true
}

func (s *Server) dispatch(sess *session, req rpcRequest, publish func([]byte)) (any, *rpcError) {
	switch req.Method {
	case "initialize":
		return s.initialize(sess, req.Params), nil
	case "ping":
		return map[string]any{}, nil
	case "tools/list":
		return map[string]any{"tools": toolDefinitions()}, nil
	case "tools/call":
		return s.callTool(sess, req.Params, publish)
	case "resources/list":
		return map[string]any{"resources": []any{map[string]any{
			"uri": catalog.ResourceTodayURI, "name": catalog.ResourceTodayName,
			"description": catalog.DescToday, "mimeType": "text/plain",
		}}}, nil
	case "resources/read":
		return map[string]any{"contents": []any{map[string]any{
			"uri": catalog.ResourceTodayURI, "mimeType": "text/plain", "text": s.store.Today(),
		}}}, nil
	case "prompts/list":
		return map[string]any{"prompts": []any{map[string]any{
			"name": catalog.PromptCapture, "description": catalog.DescPromptCapture,
		}}}, nil
	case "prompts/get":
		return map[string]any{
			"description": catalog.DescPromptCapture,
			"messages": []any{map[string]any{
				"role":    "user",
				"content": map[string]any{"type": "text", "text": catalog.PromptCaptureText},
			}},
		}, nil
	default:
		return nil, &rpcError{Code: -32601, Message: "method not found: " + req.Method}
	}
}

func (s *Server) initialize(sess *session, params json.RawMessage) map[string]any {
	var init struct {
		ProtocolVersion string `json:"protocolVersion"`
		ClientInfo      struct {
			Name    string `json:"name"`
			Version string `json:"version"`
		} `json:"clientInfo"`
	}
	_ = json.Unmarshal(params, &init)
	sess.protocol = negotiate(init.ProtocolVersion)
	sess.clientName = init.ClientInfo.Name
	sess.clientVersion = init.ClientInfo.Version
	return map[string]any{
		"protocolVersion": sess.protocol,
		"capabilities": map[string]any{
			"tools":     map[string]any{"listChanged": false},
			"resources": map[string]any{"listChanged": false},
			"prompts":   map[string]any{"listChanged": false},
		},
		"serverInfo":   map[string]any{"name": catalog.ServerName, "version": catalog.ServerVersion},
		"instructions": "Sparky spike, hand-rolled JSON-RPC. Tools: get_current_time, list_memories, create_memories.",
	}
}

func negotiate(client string) string {
	switch client {
	case "2024-11-05", "2025-03-26", "2025-06-18", "2025-11-25":
		return client
	default:
		return "2025-11-25"
	}
}

func (s *Server) callTool(sess *session, params json.RawMessage, publish func([]byte)) (any, *rpcError) {
	var call struct {
		Name      string          `json:"name"`
		Arguments json.RawMessage `json:"arguments"`
		Meta      map[string]any  `json:"_meta"`
	}
	if err := json.Unmarshal(params, &call); err != nil {
		return nil, &rpcError{Code: -32602, Message: "invalid params"}
	}
	var args map[string]any
	if len(call.Arguments) > 0 {
		if err := json.Unmarshal(call.Arguments, &args); err != nil {
			return nil, &rpcError{Code: -32602, Message: "invalid params"}
		}
	}
	text, structured, isError := s.invoke(sess, call.Name, args, call.Meta, publish)
	return map[string]any{
		"content":           []any{map[string]any{"type": "text", "text": text}},
		"structuredContent": structured,
		"isError":           isError,
	}, nil
}

func (s *Server) invoke(sess *session, name string, args map[string]any, meta map[string]any, publish func([]byte)) (string, any, bool) {
	switch name {
	case catalog.NameTime:
		out := currentTime()
		return stringArg(out["now"]), out, false
	case catalog.NameList:
		limit, err := normalizeLimit(intArg(args["limit"]))
		if err != nil {
			return err.Error(), map[string]any{}, true
		}
		page, err := s.store.List(limit, stringArg(args["cursor"]))
		if err != nil {
			return err.Error(), map[string]any{}, true
		}
		out := map[string]any{"memories": page.Memories, "total": page.Total, "nextCursor": page.NextCursor}
		return fmt.Sprintf("%d memories", page.Total), out, false
	case catalog.NameCreate:
		items, err := decodeItems(args["items"])
		if err != nil {
			return err.Error(), map[string]any{}, true
		}
		if err := validateBatch(len(items)); err != nil {
			return err.Error(), map[string]any{}, true
		}
		if token, ok := meta["progressToken"]; ok && publish != nil {
			publish(progressNote(token, 0, len(items), "creating"))
		}
		drafts := make([]mem.Draft, len(items))
		for i, item := range items {
			drafts[i] = mem.Draft{Title: item.Title, Note: item.Note}
		}
		results := s.store.Create(drafts)
		if token, ok := meta["progressToken"]; ok && publish != nil {
			publish(progressNote(token, len(items), len(items), "created"))
		}
		okCount := 0
		for _, item := range results {
			if item.OK {
				okCount++
			}
		}
		actor := "mcp:unknown"
		if sess != nil && sess.clientName != "" {
			actor = "mcp:" + sess.clientName
		}
		return fmt.Sprintf("created %d", okCount), map[string]any{"actor": actor, "results": results}, false
	default:
		return "Unknown tool. Valid: get_current_time, list_memories, create_memories.", map[string]any{}, true
	}
}

func progressNote(token any, progress, total int, message string) []byte {
	return mustJSON(map[string]any{
		"jsonrpc": "2.0",
		"method":  "notifications/progress",
		"params": map[string]any{
			"progressToken": token,
			"progress":      progress,
			"total":         total,
			"message":       message,
		},
	})
}

type itemArg struct {
	Title string `json:"title"`
	Note  string `json:"note"`
}

func decodeItems(raw any) ([]itemArg, error) {
	body, err := json.Marshal(raw)
	if err != nil {
		return nil, fmt.Errorf("create_memories needs at least 1 item")
	}
	var items []itemArg
	if err := json.Unmarshal(body, &items); err != nil {
		return nil, fmt.Errorf("create_memories items must be a list")
	}
	return items, nil
}

func intArg(v any) int {
	switch n := v.(type) {
	case float64:
		return int(n)
	case int:
		return n
	default:
		return 0
	}
}

func stringArg(v any) string {
	s, _ := v.(string)
	return s
}

func toolDefinitions() []any {
	empty := map[string]any{"type": "object", "properties": map[string]any{}}
	return []any{
		map[string]any{"name": catalog.NameTime, "description": catalog.DescTime, "inputSchema": empty},
		map[string]any{"name": catalog.NameList, "description": catalog.DescList, "inputSchema": map[string]any{
			"type": "object",
			"properties": map[string]any{
				"limit":  map[string]any{"type": "integer", "description": catalog.DescLimit},
				"cursor": map[string]any{"type": "string", "description": catalog.DescCursor},
			},
		}},
		map[string]any{"name": catalog.NameCreate, "description": catalog.DescCreate, "inputSchema": map[string]any{
			"type": "object",
			"properties": map[string]any{
				"items": map[string]any{
					"type":        "array",
					"description": catalog.DescItems,
					"items": map[string]any{
						"type": "object",
						"properties": map[string]any{
							"title": map[string]any{"type": "string", "description": catalog.DescTitle},
							"note":  map[string]any{"type": "string", "description": catalog.DescNote},
						},
						"required": []string{"title"},
					},
				},
			},
			"required": []string{"items"},
		}},
	}
}

func currentTime() map[string]any {
	loc := time.Local
	if loc == nil {
		loc = time.UTC
	}
	now := time.Now().In(loc)
	_, week := now.ISOWeek()
	return map[string]any{"now": now.Format(time.RFC3339), "tz": loc.String(), "weekday": now.Weekday().String(), "isoWeek": week}
}

func normalizeLimit(limit int) (int, error) {
	if limit == 0 {
		return 20, nil
	}
	if limit < 1 || limit > catalog.MaxLimit {
		return 0, fmt.Errorf("limit must be between 1 and %d", catalog.MaxLimit)
	}
	return limit, nil
}

func validateBatch(n int) error {
	if n < 1 {
		return fmt.Errorf("create_memories needs at least 1 item")
	}
	if n > catalog.MaxBatch {
		return fmt.Errorf("create_memories accepts at most %d items", catalog.MaxBatch)
	}
	return nil
}

func mustJSON(v any) []byte {
	body, err := json.Marshal(v)
	if err != nil {
		body = []byte(`{"jsonrpc":"2.0","error":{"code":-32603,"message":"internal error"}}`)
	}
	return body
}

// ServeStdio speaks one JSON-RPC message per line. publish is unused on stdio;
// progress notifications are written to out before the tool result.
func (s *Server) ServeStdio(ctx context.Context, in io.Reader, out io.Writer) error {
	sess := &session{id: "stdio", publisher: newPublisher()}
	scanner := bufio.NewScanner(in)
	scanner.Buffer(make([]byte, 0, 64*1024), 1<<20)
	for scanner.Scan() {
		if ctx.Err() != nil {
			return ctx.Err()
		}
		line := scanner.Bytes()
		if len(line) == 0 {
			continue
		}
		var notes [][]byte
		body, ok := s.handle(sess, append([]byte(nil), line...), func(msg []byte) {
			notes = append(notes, append([]byte(nil), msg...))
		})
		for _, note := range notes {
			if _, err := fmt.Fprintf(out, "%s\n", note); err != nil {
				return err
			}
		}
		if ok {
			if _, err := fmt.Fprintf(out, "%s\n", body); err != nil {
				return err
			}
		}
	}
	return scanner.Err()
}
