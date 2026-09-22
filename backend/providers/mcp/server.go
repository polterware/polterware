package mcp

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"

	"github.com/polterware/polterware/backend/domains/agentaccess"
	contextpkg "github.com/polterware/polterware/backend/domains/context"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

// Experimental tool names — not a product contract.
const (
	ToolSearch         = "search"
	ToolCompileContext = "compile_context"
	ToolGetEntity      = "get_entity"
)

type Server struct {
	Store *storepkg.Store
	In    io.Reader
	Out   io.Writer
	Err   io.Writer
}

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

func Run(store *storepkg.Store) error {
	server := &Server{Store: store, In: os.Stdin, Out: os.Stdout, Err: os.Stderr}
	return server.Serve()
}

func (s *Server) Serve() error {
	scanner := bufio.NewScanner(s.In)
	scanner.Buffer(make([]byte, 0, 64*1024), 1024*1024)
	for scanner.Scan() {
		line := scanner.Bytes()
		if len(line) == 0 {
			continue
		}
		var req rpcRequest
		if err := json.Unmarshal(line, &req); err != nil {
			_ = s.write(rpcResponse{JSONRPC: "2.0", Error: &rpcError{Code: -32700, Message: "parse error"}})
			continue
		}
		if req.Method == "notifications/initialized" || req.Method == "notifications/cancelled" {
			continue
		}
		result, err := s.dispatch(req)
		if req.ID == nil {
			continue
		}
		if err != nil {
			_ = s.write(rpcResponse{JSONRPC: "2.0", ID: req.ID, Error: &rpcError{Code: -32000, Message: err.Error()}})
			continue
		}
		_ = s.write(rpcResponse{JSONRPC: "2.0", ID: req.ID, Result: result})
	}
	return scanner.Err()
}

func (s *Server) write(resp rpcResponse) error {
	body, err := json.Marshal(resp)
	if err != nil {
		return err
	}
	_, err = fmt.Fprintf(s.Out, "%s\n", body)
	return err
}

func (s *Server) dispatch(req rpcRequest) (any, error) {
	switch req.Method {
	case "initialize":
		return map[string]any{
			"protocolVersion": "2024-11-05",
			"capabilities":    map[string]any{"tools": map[string]any{}},
			"serverInfo":      map[string]any{"name": "polterware-core-mcp", "version": "0.1.0-experimental"},
			"instructions":    "Experimental Polterware MCP. Tool names are provisional and not a product API contract.",
		}, nil
	case "tools/list":
		return map[string]any{"tools": []map[string]any{
			{
				"name":        ToolSearch,
				"description": "Exact id and full-text search over project knowledge (experimental).",
				"inputSchema": map[string]any{
					"type": "object",
					"properties": map[string]any{
						"project_id": map[string]any{"type": "string"},
						"query":      map[string]any{"type": "string"},
						"kind":       map[string]any{"type": "string"},
						"limit":      map[string]any{"type": "integer"},
					},
					"required": []string{"project_id", "query"},
				},
			},
			{
				"name":        ToolCompileContext,
				"description": "Compile a bounded, provenance-aware project context package with reasons and evidence refs (experimental).",
				"inputSchema": map[string]any{
					"type": "object",
					"properties": map[string]any{
						"project_id":   map[string]any{"type": "string"},
						"query":        map[string]any{"type": "string"},
						"entity_id":    map[string]any{"type": "string"},
						"token_budget": map[string]any{"type": "integer"},
					},
					"required": []string{"project_id"},
				},
			},
			{
				"name":        ToolGetEntity,
				"description": "Fetch one PPM entity by id (experimental).",
				"inputSchema": map[string]any{
					"type": "object",
					"properties": map[string]any{
						"id": map[string]any{"type": "string"},
					},
					"required": []string{"id"},
				},
			},
		}}, nil
	case "tools/call":
		var params struct {
			Name      string         `json:"name"`
			Arguments map[string]any `json:"arguments"`
		}
		if err := json.Unmarshal(req.Params, &params); err != nil {
			return nil, err
		}
		text, err := s.callTool(params.Name, params.Arguments)
		if err != nil {
			return map[string]any{
				"content": []map[string]any{{"type": "text", "text": err.Error()}},
				"isError": true,
			}, nil
		}
		return map[string]any{
			"content": []map[string]any{{"type": "text", "text": text}},
		}, nil
	case "ping":
		return map[string]any{}, nil
	default:
		return nil, fmt.Errorf("method not found: %s", req.Method)
	}
}

func (s *Server) callTool(name string, args map[string]any) (string, error) {
	projectID := stringFrom(args["project_id"])
	argsJSON, _ := json.Marshal(args)
	text, err := s.invokeTool(name, args)
	summary := agentaccess.SummarizeResult(text, 240)
	ok := err == nil
	if err != nil {
		summary = agentaccess.SummarizeResult(err.Error(), 240)
	}
	_ = s.Store.AppendAgentToolCall(agentaccess.ToolCall{
		ProjectID:     projectID,
		ToolName:      name,
		ArgumentsJSON: string(argsJSON),
		ResultSummary: summary,
		OK:            ok,
		Source:        "mcp",
	})
	return text, err
}

func (s *Server) invokeTool(name string, args map[string]any) (string, error) {
	switch name {
	case ToolSearch:
		projectID, _ := args["project_id"].(string)
		query, _ := args["query"].(string)
		kind, _ := args["kind"].(string)
		limit := intFrom(args["limit"], 20)
		hits, err := s.Store.SearchKnowledge(projectID, query, kind, limit)
		if err != nil {
			return "", err
		}
		return marshalPretty(hits)
	case ToolCompileContext:
		projectID, _ := args["project_id"].(string)
		_, _ = s.Store.SyncProjectModel(projectID)
		pkg, err := s.Store.CompileContext(contextpkg.Request{
			ProjectID:   projectID,
			Query:       stringFrom(args["query"]),
			EntityID:    stringFrom(args["entity_id"]),
			TokenBudget: intFrom(args["token_budget"], contextpkg.DefaultTokenBudget),
		})
		if err != nil {
			return "", err
		}
		return marshalPretty(pkg)
	case ToolGetEntity:
		id, _ := args["id"].(string)
		entity, err := s.Store.GetPpmEntity(id)
		if err != nil {
			return "", err
		}
		return marshalPretty(entity)
	default:
		return "", fmt.Errorf("unknown experimental tool %q", name)
	}
}

func marshalPretty(value any) (string, error) {
	body, err := json.MarshalIndent(value, "", "  ")
	if err != nil {
		return "", err
	}
	return string(body), nil
}

func stringFrom(value any) string {
	text, _ := value.(string)
	return text
}

func intFrom(value any, fallback int) int {
	switch typed := value.(type) {
	case float64:
		return int(typed)
	case int:
		return typed
	case int32:
		return int(typed)
	case json.Number:
		n, err := typed.Int64()
		if err == nil {
			return int(n)
		}
	}
	return fallback
}
