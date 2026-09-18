package localcore

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"path/filepath"
	"strings"
	"time"

	"connectrpc.com/connect"

	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
	"github.com/polterware/polterware/backend/platform/observability"
	processpkg "github.com/polterware/polterware/backend/providers/process"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

const localSeedNodeID = "node-local"

// isLocalProcessPlatform reports whether a node platform may host local OS processes.
func isLocalProcessPlatform(platform string) bool {
	normalized := strings.ToLower(strings.TrimSpace(platform))
	if normalized == "" {
		return false
	}
	if normalized == "local" {
		return true
	}
	return strings.HasPrefix(normalized, "local ")
}

func (core *Core) isLocalProcessNode(nodeID string) (bool, error) {
	if nodeID == "" || nodeID == localSeedNodeID {
		return true, nil
	}
	node, err := core.Store.GetNode(nodeID)
	if errors.Is(err, sql.ErrNoRows) {
		return false, nil
	}
	if err != nil {
		return false, err
	}
	return isLocalProcessPlatform(node.Platform) || node.ID == localSeedNodeID, nil
}

func (core *Core) StartProcess(_ context.Context, request *connect.Request[localv1.StartProcessRequest]) (*connect.Response[localv1.StartProcessResponse], error) {
	if request.Msg.ProjectId == "" || request.Msg.Command == "" {
		return nil, invalid("project id and command are required")
	}
	ok, err := core.isLocalProcessNode(request.Msg.NodeId)
	if err != nil {
		return nil, internal(err)
	}
	if !ok {
		return nil, connect.NewError(connect.CodeFailedPrecondition, fmt.Errorf("only the local host can start processes"))
	}

	correlation := observability.NewCorrelationID()
	sessionID := "process-" + correlation[:12]
	displayCommand := strings.Join(append([]string{request.Msg.Command}, request.Msg.Args...), " ")
	name := strings.TrimSpace(request.Msg.Name)
	if name == "" {
		name = filepath.Base(request.Msg.Command)
	}
	nodeID := request.Msg.NodeId
	if nodeID == "" {
		nodeID = localSeedNodeID
	}
	startedAt := time.Now().UTC().Format(time.RFC3339)

	pid, wait, spawnErr := core.processManager().Spawn(sessionID, request.Msg.Command, request.Msg.Args, request.Msg.Cwd)
	if spawnErr != nil {
		session := storepkg.ProcessSession{
			ID:            sessionID,
			ProjectID:     request.Msg.ProjectId,
			EnvironmentID: request.Msg.EnvironmentId,
			Command:       displayCommand,
			Status:        "failed",
			Name:          name,
			NodeID:        nodeID,
			StartedAt:     startedAt,
		}
		_ = core.Store.SaveProcess(session)
		return nil, connect.NewError(connect.CodeInternal, spawnErr)
	}

	session := storepkg.ProcessSession{
		ID:            sessionID,
		ProjectID:     request.Msg.ProjectId,
		EnvironmentID: request.Msg.EnvironmentId,
		Command:       displayCommand,
		Status:        "running",
		Name:          name,
		NodeID:        nodeID,
		PID:           pid,
		StartedAt:     startedAt,
	}
	if err := core.Store.SaveProcess(session); err != nil {
		core.processManager().Cancel(sessionID)
		return nil, internal(err)
	}

	go core.watchProcessExit(sessionID, wait)
	return connect.NewResponse(&localv1.StartProcessResponse{Session: processProto(session)}), nil
}

func (core *Core) watchProcessExit(sessionID string, wait <-chan processpkg.SpawnResult) {
	result, ok := <-wait
	if !ok {
		return
	}
	session, err := core.Store.GetProcess(sessionID)
	if err != nil {
		return
	}
	if session.Status != "running" {
		return
	}
	if result.Canceled {
		session.Status = "stopped"
	} else if result.ExitCode != 0 || result.Err != nil {
		session.Status = "failed"
	} else {
		session.Status = "stopped"
	}
	_ = core.Store.SaveProcess(session)
}

func (core *Core) StopProcess(_ context.Context, request *connect.Request[localv1.StopProcessRequest]) (*connect.Response[localv1.StopProcessResponse], error) {
	session, err := core.Store.GetProcess(request.Msg.Id)
	if err != nil {
		return nil, connect.NewError(connect.CodeNotFound, err)
	}
	core.processManager().Cancel(request.Msg.Id)
	session.Status = "stopped"
	if err := core.Store.SaveProcess(session); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.StopProcessResponse{Session: processProto(session)}), nil
}
