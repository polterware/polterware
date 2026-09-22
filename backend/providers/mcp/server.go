package mcp

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"

	"github.com/polterware/polterware/backend/domains/agentaccess"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
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
			"serverInfo":      map[string]any{"name": "polterware-core-mcp", "version": "0.2.0"},
			"instructions":    "Polterware MCP read contract: context.get, entity.get, evidence.get, drift.check. Prefer context.get for bounded project context with reasons and evidence.",
		}, nil
	case "tools/list":
		return map[string]any{"tools": toolDefinitions()}, nil
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
	if args == nil {
		args = map[string]any{}
	}
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
