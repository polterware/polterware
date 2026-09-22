package localcore

import (
	"context"
	"fmt"
	"strings"
	"time"

	"connectrpc.com/connect"

	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
	"github.com/polterware/polterware/backend/platform/observability"
	terminalpkg "github.com/polterware/polterware/backend/providers/terminal"
)

func (core *Core) terminalManager() *terminalpkg.Manager {
	core.terminalOnce.Do(func() {
		core.Terminals = terminalpkg.NewManager()
	})
	return core.Terminals
}

func (core *Core) CreateTerminalSession(_ context.Context, request *connect.Request[localv1.CreateTerminalSessionRequest]) (*connect.Response[localv1.CreateTerminalSessionResponse], error) {
	if request.Msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	id := "term-" + observability.NewCorrelationID()[:12]
	cols := uint16(request.Msg.Cols)
	rows := uint16(request.Msg.Rows)
	targetID := request.Msg.TargetId
	shell := request.Msg.Shell
	cwd := request.Msg.Cwd

	if !isSyntheticLocalTarget(targetID) {
		return nil, invalid("terminal target not found")
	}

	session, err := core.terminalManager().Create(id, request.Msg.ProjectId, syntheticTargetLabel(targetID), shell, cwd, cols, rows)
	if err != nil {
		return nil, internal(fmt.Errorf("create terminal: %w", err))
	}
	return connect.NewResponse(&localv1.CreateTerminalSessionResponse{Session: terminalProto(session)}), nil
}

// isSyntheticLocalTarget treats empty, "local", and Connect auth-* targets as local PTYs
// without consulting a nodes table.
func isSyntheticLocalTarget(targetID string) bool {
	trimmed := strings.TrimSpace(targetID)
	return trimmed == "" || trimmed == "local" || strings.HasPrefix(trimmed, "auth-")
}

func syntheticTargetLabel(targetID string) string {
	trimmed := strings.TrimSpace(targetID)
	if trimmed == "" {
		return "local"
	}
	return trimmed
}

func (core *Core) WriteTerminalSession(_ context.Context, request *connect.Request[localv1.WriteTerminalSessionRequest]) (*connect.Response[localv1.WriteTerminalSessionResponse], error) {
	if request.Msg.Id == "" {
		return nil, invalid("session id is required")
	}
	if err := core.terminalManager().Write(request.Msg.Id, request.Msg.Data); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.WriteTerminalSessionResponse{}), nil
}

func (core *Core) ResizeTerminalSession(_ context.Context, request *connect.Request[localv1.ResizeTerminalSessionRequest]) (*connect.Response[localv1.ResizeTerminalSessionResponse], error) {
	session, err := core.terminalManager().Resize(request.Msg.Id, uint16(request.Msg.Cols), uint16(request.Msg.Rows))
	if err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.ResizeTerminalSessionResponse{Session: terminalProto(session)}), nil
}

func (core *Core) CloseTerminalSession(_ context.Context, request *connect.Request[localv1.CloseTerminalSessionRequest]) (*connect.Response[localv1.CloseTerminalSessionResponse], error) {
	session, err := core.terminalManager().Close(request.Msg.Id)
	if err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.CloseTerminalSessionResponse{Session: terminalProto(session)}), nil
}

func (core *Core) ReadTerminalOutput(_ context.Context, request *connect.Request[localv1.ReadTerminalOutputRequest]) (*connect.Response[localv1.ReadTerminalOutputResponse], error) {
	data, next, closed, err := core.terminalManager().Read(request.Msg.Id, request.Msg.Offset)
	if err != nil {
		return nil, connect.NewError(connect.CodeNotFound, err)
	}
	return connect.NewResponse(&localv1.ReadTerminalOutputResponse{Data: data, NextOffset: next, Closed: closed}), nil
}

func (core *Core) ListTerminalSessions(_ context.Context, request *connect.Request[localv1.ListTerminalSessionsRequest]) (*connect.Response[localv1.ListTerminalSessionsResponse], error) {
	items := core.terminalManager().List(request.Msg.ProjectId)
	result := make([]*localv1.TerminalSession, 0, len(items))
	for _, item := range items {
		result = append(result, terminalProto(item))
	}
	return connect.NewResponse(&localv1.ListTerminalSessionsResponse{Sessions: result}), nil
}

func terminalProto(item *terminalpkg.Session) *localv1.TerminalSession {
	return &localv1.TerminalSession{
		Id:        item.ID,
		ProjectId: item.ProjectID,
		TargetId:  item.TargetID,
		Status:    item.Status,
		Shell:     item.Shell,
		Cwd:       item.Cwd,
		StartedAt: item.StartedAt.Format(time.RFC3339),
		Cols:      int32(item.Cols),
		Rows:      int32(item.Rows),
	}
}
