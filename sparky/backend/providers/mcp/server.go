package mcp

import (
	"context"
	"fmt"
	"net/http"
	"sync"
	"time"

	mcpsdk "github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
)

// Server is the stateful official-SDK MCP server.
type Server struct {
	deps     Deps
	sdk      *mcpsdk.Server
	handler  http.Handler
	mu       sync.Mutex
	sessions map[string]*MemorySession
}

// New registers the compact catalog on a stateful SDK server.
func New(deps Deps) (*Server, error) {
	if deps.Memories == nil || deps.Minds == nil {
		return nil, fmt.Errorf("MCP requires memory and mind services")
	}
	version := deps.Version
	if version == "" {
		version = defaultMCPVer
	}
	s := &Server{deps: deps, sessions: make(map[string]*MemorySession)}
	s.sdk = mcpsdk.NewServer(&mcpsdk.Implementation{Name: ServerName, Version: version}, nil)
	tools, err := Catalog()
	if err != nil {
		return nil, err
	}
	handlers := s.handlers()
	for _, tool := range tools {
		handler, ok := handlers[tool.Name]
		if !ok {
			return nil, fmt.Errorf("missing handler for %s", tool.Name)
		}
		s.sdk.AddTool(&mcpsdk.Tool{Name: tool.Name, Description: tool.Description, InputSchema: tool.InputSchema}, handler)
	}
	s.addResourcesAndPrompts()
	s.handler = mcpsdk.NewStreamableHTTPHandler(func(*http.Request) *mcpsdk.Server { return s.sdk }, &mcpsdk.StreamableHTTPOptions{
		JSONResponse:   true,
		SessionTimeout: 30 * time.Minute,
	})
	return s, nil
}

func (s *Server) SDK() *mcpsdk.Server { return s.sdk }

// Handler is the unguarded streamable HTTP handler. Core wraps it with NewGuard.
func (s *Server) Handler() http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if err := s.requireEnabled(r.Context()); err != nil {
			http.Error(w, sentence(err), http.StatusForbidden)
			return
		}
		s.handler.ServeHTTP(w, r)
	})
}

func (s *Server) requireEnabled(ctx context.Context) error {
	if s.deps.Settings == nil {
		return nil
	}
	doc, err := s.deps.Settings.Get(ctx)
	if err != nil {
		return fault.Unavailable("MCP settings are unavailable")
	}
	if !doc.AgentsMCPEnabled {
		return fmt.Errorf("MCP is disabled. Enable it in Settings > Agents")
	}
	return nil
}

func (s *Server) fileAttachAllowed(ctx context.Context) (bool, error) {
	if s.deps.Settings == nil {
		return false, nil
	}
	doc, err := s.deps.Settings.Get(ctx)
	if err != nil {
		return false, err
	}
	return doc.AgentsAllowFileAttach, nil
}

func (s *Server) session(req *mcpsdk.CallToolRequest) *MemorySession {
	if req == nil {
		return s.sessionFor(nil)
	}
	return s.sessionFor(req.Session)
}

func (s *Server) sessionFor(transport *mcpsdk.ServerSession) *MemorySession {
	if transport == nil {
		return NewMemorySession(s.deps.Memories, "unknown")
	}
	key := fmt.Sprintf("%p", transport)
	name := "unknown"
	if params := transport.InitializeParams(); params != nil && params.ClientInfo != nil && params.ClientInfo.Name != "" {
		name = params.ClientInfo.Name
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	sess := s.sessions[key]
	if sess == nil {
		sess = NewMemorySession(s.deps.Memories, name)
		s.sessions[key] = sess
		go func() {
			_ = transport.Wait() // closes on DELETE, expiration, or transport termination
			s.mu.Lock()
			defer s.mu.Unlock()
			delete(s.sessions, key)
		}()
	}
	return sess
}

func (s *Server) handlers() map[string]mcpsdk.ToolHandler {
	return map[string]mcpsdk.ToolHandler{
		"get_current_time":  s.getCurrentTime,
		"list_minds":        s.listMinds,
		"list_tags":         s.listTags,
		"list_memories":     s.listMemories,
		"search_memories":   s.searchMemories,
		"get_memory":        s.getMemory,
		"list_occurrences":  s.listOccurrences,
		"get_review":        s.getReview,
		"list_activity":     s.listActivity,
		"create_memories":   s.createMemories,
		"update_memories":   s.updateMemories,
		"set_memory_status": s.setMemoryStatus,
		"toggle_check_item": s.toggleCheckItem,
		"delete_memories":   s.deleteMemories,
		"create_mind":       s.createMind,
		"update_mind":       s.updateMind,
		"delete_mind":       s.deleteMind,
		"add_attachments":   s.addAttachments,
	}
}
