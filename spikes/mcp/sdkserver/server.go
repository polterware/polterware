// Package sdkserver is the official Go MCP SDK spike (github.com/modelcontextprotocol/go-sdk v1.8.0).
package sdkserver

import (
	"context"
	"fmt"
	"net/http"
	"time"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/polterware/polterware/spikes/mcp/internal/catalog"
	"github.com/polterware/polterware/spikes/mcp/internal/mem"
)

// Server is the spike MCP server. One instance is shared by every session.
type Server struct {
	mcp   *mcp.Server
	store *mem.Store
}

// New builds the SDK server with the three spike tools plus one resource and one prompt.
func New(store *mem.Store) *Server {
	s := &Server{
		store: store,
		mcp: mcp.NewServer(&mcp.Implementation{
			Name:    catalog.ServerName,
			Version: catalog.ServerVersion,
		}, nil),
	}
	mcp.AddTool(s.mcp, &mcp.Tool{Name: catalog.NameTime, Description: catalog.DescTime}, s.currentTime)
	mcp.AddTool(s.mcp, &mcp.Tool{Name: catalog.NameList, Description: catalog.DescList}, s.listMemories)
	mcp.AddTool(s.mcp, &mcp.Tool{Name: catalog.NameCreate, Description: catalog.DescCreate}, s.createMemories)
	s.mcp.AddResource(&mcp.Resource{
		URI:         catalog.ResourceTodayURI,
		Name:        catalog.ResourceTodayName,
		Description: catalog.DescToday,
		MIMEType:    "text/plain",
	}, s.readToday)
	s.mcp.AddPrompt(&mcp.Prompt{
		Name:        catalog.PromptCapture,
		Description: catalog.DescPromptCapture,
	}, s.capturePrompt)
	return s
}

// MCP returns the SDK server for stdio.
func (s *Server) MCP() *mcp.Server { return s.mcp }

// Handler is the streamable HTTP handler. Sessions stay stateful so a later
// expectedRev check can remember what this session last read. JSON responses
// keep the stdio proxy simple; progress still travels on the GET SSE stream.
func (s *Server) Handler() http.Handler {
	return mcp.NewStreamableHTTPHandler(func(*http.Request) *mcp.Server { return s.mcp }, &mcp.StreamableHTTPOptions{
		JSONResponse:   true,
		SessionTimeout: 30 * time.Minute,
	})
}

type emptyIn struct{}

type timeOut struct {
	Now     string `json:"now"`
	TZ      string `json:"tz"`
	Weekday string `json:"weekday"`
	ISOWeek int    `json:"isoWeek"`
}

func (s *Server) currentTime(context.Context, *mcp.CallToolRequest, emptyIn) (*mcp.CallToolResult, timeOut, error) {
	out := currentTime()
	return &mcp.CallToolResult{Content: []mcp.Content{&mcp.TextContent{Text: out.Now}}}, out, nil
}

type listIn struct {
	Limit  int    `json:"limit,omitempty" jsonschema:"Page size from 1 to 100. Defaults to 20."`
	Cursor string `json:"cursor,omitempty" jsonschema:"Opaque cursor from the previous page."`
}

type listOut struct {
	Memories   []mem.Summary `json:"memories"`
	Total      int           `json:"total"`
	NextCursor string        `json:"nextCursor"`
}

func (s *Server) listMemories(_ context.Context, _ *mcp.CallToolRequest, in listIn) (*mcp.CallToolResult, listOut, error) {
	limit, err := normalizeLimit(in.Limit)
	if err != nil {
		return nil, listOut{}, err
	}
	page, err := s.store.List(limit, in.Cursor)
	if err != nil {
		return nil, listOut{}, err
	}
	out := listOut{Memories: page.Memories, Total: page.Total, NextCursor: page.NextCursor}
	return &mcp.CallToolResult{Content: []mcp.Content{&mcp.TextContent{Text: fmt.Sprintf("%d memories", page.Total)}}}, out, nil
}

type createItem struct {
	Title string `json:"title" jsonschema:"Memory title."`
	Note  string `json:"note,omitempty" jsonschema:"Optional Markdown note."`
}

type createIn struct {
	Items []createItem `json:"items" jsonschema:"Memories to create, at most 50."`
}

type createOut struct {
	Actor   string           `json:"actor"`
	Results []mem.ItemResult `json:"results"`
}

func (s *Server) createMemories(ctx context.Context, req *mcp.CallToolRequest, in createIn) (*mcp.CallToolResult, createOut, error) {
	if err := validateBatch(len(in.Items)); err != nil {
		return nil, createOut{}, err
	}
	token := req.Params.GetProgressToken()
	if token != nil && req.Session != nil {
		_ = req.Session.NotifyProgress(ctx, &mcp.ProgressNotificationParams{
			ProgressToken: token,
			Progress:      0,
			Total:         float64(len(in.Items)),
			Message:       "creating",
		})
	}
	drafts := make([]mem.Draft, len(in.Items))
	for i, item := range in.Items {
		drafts[i] = mem.Draft{Title: item.Title, Note: item.Note}
	}
	out := createOut{Actor: actor(req), Results: s.store.Create(drafts)}
	if token != nil && req.Session != nil {
		_ = req.Session.NotifyProgress(ctx, &mcp.ProgressNotificationParams{
			ProgressToken: token,
			Progress:      float64(len(in.Items)),
			Total:         float64(len(in.Items)),
			Message:       "created",
		})
	}
	ok := 0
	for _, item := range out.Results {
		if item.OK {
			ok++
		}
	}
	return &mcp.CallToolResult{Content: []mcp.Content{&mcp.TextContent{Text: fmt.Sprintf("created %d", ok)}}}, out, nil
}

func (s *Server) readToday(context.Context, *mcp.ReadResourceRequest) (*mcp.ReadResourceResult, error) {
	return &mcp.ReadResourceResult{Contents: []*mcp.ResourceContents{{
		URI:      catalog.ResourceTodayURI,
		MIMEType: "text/plain",
		Text:     s.store.Today(),
	}}}, nil
}

func (s *Server) capturePrompt(context.Context, *mcp.GetPromptRequest) (*mcp.GetPromptResult, error) {
	return &mcp.GetPromptResult{
		Description: catalog.DescPromptCapture,
		Messages: []*mcp.PromptMessage{{
			Role:    "user",
			Content: &mcp.TextContent{Text: catalog.PromptCaptureText},
		}},
	}, nil
}

func actor(req *mcp.CallToolRequest) string {
	if req != nil && req.Session != nil && req.Session.InitializeParams() != nil && req.Session.InitializeParams().ClientInfo != nil {
		if name := req.Session.InitializeParams().ClientInfo.Name; name != "" {
			return "mcp:" + name
		}
	}
	return "mcp:unknown"
}

func currentTime() timeOut {
	loc := time.Local
	if loc == nil {
		loc = time.UTC
	}
	now := time.Now().In(loc)
	_, week := now.ISOWeek()
	return timeOut{Now: now.Format(time.RFC3339), TZ: loc.String(), Weekday: now.Weekday().String(), ISOWeek: week}
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
