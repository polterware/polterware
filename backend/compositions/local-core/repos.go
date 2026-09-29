package localcore

import (
	"context"
	"strings"

	"connectrpc.com/connect"

	"github.com/polterware/polterware/backend/compositions/wedge"
	"github.com/polterware/polterware/backend/domains/projects"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
)

func (core *Core) ListProjectRepos(_ context.Context, request *connect.Request[localv1.ListProjectReposRequest]) (*connect.Response[localv1.ListProjectReposResponse], error) {
	if request.Msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	items, err := core.Store.ListProjectRepos(request.Msg.ProjectId)
	if err != nil {
		return nil, internal(err)
	}
	result := make([]*localv1.ProjectRepo, 0, len(items))
	for _, item := range items {
		result = append(result, repoProto(item))
	}
	return connect.NewResponse(&localv1.ListProjectReposResponse{Repos: result}), nil
}

func (core *Core) AddProjectRepo(_ context.Context, request *connect.Request[localv1.AddProjectRepoRequest]) (*connect.Response[localv1.AddProjectRepoResponse], error) {
	repo, err := wedge.AddProjectRepo(core.Store, request.Msg.ProjectId, request.Msg.Name, request.Msg.Path)
	if err != nil {
		return nil, repoErr(err)
	}
	return connect.NewResponse(&localv1.AddProjectRepoResponse{Repo: repoProto(repo)}), nil
}

func (core *Core) RemoveProjectRepo(_ context.Context, request *connect.Request[localv1.RemoveProjectRepoRequest]) (*connect.Response[localv1.RemoveProjectRepoResponse], error) {
	if err := wedge.RemoveProjectRepo(core.Store, request.Msg.ProjectId, request.Msg.Name); err != nil {
		return nil, repoErr(err)
	}
	return connect.NewResponse(&localv1.RemoveProjectRepoResponse{}), nil
}

func repoProto(item projects.Repository) *localv1.ProjectRepo {
	return &localv1.ProjectRepo{Id: item.ID, ProjectId: item.ProjectID, Name: item.Name, Path: item.Path}
}

func repoErr(err error) error {
	msg := err.Error()
	switch {
	case strings.HasPrefix(msg, "unknown "):
		return connect.NewError(connect.CodeNotFound, err)
	case strings.Contains(msg, "required"),
		strings.Contains(msg, "already"),
		strings.Contains(msg, "must be"),
		strings.Contains(msg, "does not exist"),
		strings.Contains(msg, "not a directory"),
		strings.Contains(msg, "primary checkout"),
		strings.Contains(msg, "empty"):
		return invalid(msg)
	default:
		return internal(err)
	}
}
