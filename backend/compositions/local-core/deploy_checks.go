package localcore

import (
	"context"
	"time"

	"connectrpc.com/connect"

	"github.com/polterware/polterware/backend/compositions/wedge"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
)

// The background deploy check: a first pass shortly after the core starts, then
// one every deployCheckEvery while it runs. Each pass checks only links that are due.
const (
	deployCheckFirst = 30 * time.Second
	deployCheckEvery = 10 * time.Minute
)

// watchDeploys checks every project's deploys on a timer until the core stops.
// The writes reach clients as change events, and failures raise Inbox items.
func (core *Core) watchDeploys() {
	timer := time.NewTimer(deployCheckFirst)
	defer timer.Stop()
	for {
		select {
		case <-core.Done():
			return
		case <-timer.C:
			_ = wedge.CheckAllDeploys(core.Store, time.Now().UTC())
			timer.Reset(deployCheckEvery)
		}
	}
}

func (core *Core) ListDeployTargets(_ context.Context, request *connect.Request[localv1.ListDeployTargetsRequest]) (*connect.Response[localv1.ListDeployTargetsResponse], error) {
	msg := request.Msg
	if msg.GetProjectId() == "" || msg.GetProvider() == "" {
		return nil, invalid("project id and provider are required")
	}
	result, err := wedge.DeployTargets(core.Store, msg.GetProjectId(), msg.GetNodeId(), msg.GetProvider())
	if err != nil {
		return nil, invalid(err.Error())
	}
	out := &localv1.ListDeployTargetsResponse{Message: result.Message}
	for _, target := range result.Targets {
		out.Targets = append(out.Targets, &localv1.DeployTarget{Name: target.Name, Id: target.ID, Kind: target.Kind, Url: target.URL, Detected: target.Detected})
	}
	return connect.NewResponse(out), nil
}

func (core *Core) CheckDeploys(_ context.Context, request *connect.Request[localv1.CheckDeploysRequest]) (*connect.Response[localv1.CheckDeploysResponse], error) {
	msg := request.Msg
	if msg.GetProjectId() == "" {
		return nil, invalid("project id is required")
	}
	checked, err := wedge.CheckDeploys(core.Store, wedge.CheckDeploysInput{ProjectID: msg.GetProjectId(), NodeID: msg.GetNodeId(), Force: msg.GetForce()})
	if err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.CheckDeploysResponse{Checked: int32(len(checked))}), nil
}
