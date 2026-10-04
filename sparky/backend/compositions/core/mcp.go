package core

import (
	"context"
	"fmt"
	"net"
	"net/http"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/schedule"
	"github.com/polterware/polterware/sparky/backend/domains/settings"
	"github.com/polterware/polterware/sparky/backend/platform/listen"
	"github.com/polterware/polterware/sparky/backend/providers/mcp"
)

type mcpSchedules struct{ ScheduleUseCases }

func (s mcpSchedules) ListOccurrences(ctx context.Context, from, to time.Time, mindID *string, includeCompleted bool, limit int) ([]schedule.Occurrence, error) {
	return s.ScheduleUseCases.ListOccurrences(ctx, OccurrenceQuery{From: from, To: to, MindID: mindID, IncludeCompleted: includeCompleted, Limit: limit})
}

func (s mcpSchedules) Set(ctx context.Context, who actor.Actor, in schedule.Schedule) (schedule.Schedule, error) {
	return s.ScheduleUseCases.Set(ctx, who, in)
}

func (s mcpSchedules) Clear(ctx context.Context, who actor.Actor, memoryID string) error {
	return s.ScheduleUseCases.Clear(ctx, who, memoryID)
}

func (s mcpSchedules) Preview(ctx context.Context, in schedule.Schedule, limit int) ([]time.Time, string, error) {
	return s.ScheduleUseCases.Preview(ctx, in, limit)
}

func bindMCP(services Services, token string, port int, enabled bool, version string) (MCP, error) {
	if token == "" {
		return MCP{Port: port, Enabled: enabled}, nil
	}
	deps := mcp.Deps{
		Version: version, Memories: services.Memories, Minds: services.Minds, Tags: services.Tags,
		Activity: services.Activity, Review: services.Review, Search: services.Search,
		Attachments: services.Attachments, Places: services.Places, Settings: services.Settings,
	}
	if services.ScheduleReader != nil {
		deps.GetSchedule = services.ScheduleReader.Get
	}
	if services.PlaceReader != nil {
		deps.GetPlace = services.PlaceReader.Get
	}
	if services.Schedules != nil {
		deps.Schedules = mcpSchedules{services.Schedules}
	}
	srv, err := mcp.New(deps)
	if err != nil {
		return MCP{}, err
	}
	return MCP{Handler: srv.Handler(), Token: token, Port: port, Enabled: enabled}, nil
}

func (s *Server) mountMCP(connectMux *http.ServeMux, connectPort int) error {
	if s.config.MCP.Handler == nil {
		return nil
	}
	if s.config.MCP.Token == "" {
		return fmt.Errorf("MCP token is required")
	}
	port := s.config.MCP.Port
	if port < 1 || port > 65535 {
		port = settings.DefaultMCPPort
	}
	ln, _, err := listen.Open(fmt.Sprintf("127.0.0.1:%d", port))
	if err != nil {
		ln, _, err = listen.Open("127.0.0.1:0")
	}
	if err != nil {
		wrap, gerr := mcp.NewGuard(s.config.MCP.Token, connectPort, mcp.MaxBodyBytes)
		if gerr != nil {
			return gerr
		}
		connectMux.Handle("/mcp", wrap(s.config.MCP.Handler))
		s.config.MCP.Port = connectPort
		return nil
	}
	actual := ln.Addr().(*net.TCPAddr).Port
	wrap, err := mcp.NewGuard(s.config.MCP.Token, actual, mcp.MaxBodyBytes)
	if err != nil {
		_ = ln.Close()
		return err
	}
	mux := http.NewServeMux()
	mux.Handle("/mcp", wrap(s.config.MCP.Handler))
	s.mcpHTTP = &http.Server{Handler: mux, ReadHeaderTimeout: 5 * time.Second, IdleTimeout: 60 * time.Second, MaxHeaderBytes: 16 << 10, BaseContext: func(net.Listener) context.Context { return s.ctx }}
	s.config.MCP.Port = actual
	go func() { _ = s.mcpHTTP.Serve(ln) }()
	return nil
}
