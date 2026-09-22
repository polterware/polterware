package localcore

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"time"

	"connectrpc.com/connect"

	"github.com/polterware/polterware/backend/domains/agentaccess"
	contextpkg "github.com/polterware/polterware/backend/domains/context"
	"github.com/polterware/polterware/backend/domains/knowledge"
	"github.com/polterware/polterware/backend/domains/ppm"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
)

func (core *Core) ListEntities(_ context.Context, request *connect.Request[localv1.ListEntitiesRequest]) (*connect.Response[localv1.ListEntitiesResponse], error) {
	if request.Msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	_, _ = core.Store.SyncProjectModel(request.Msg.ProjectId)
	items, err := core.Store.ListPpmEntities(request.Msg.ProjectId, request.Msg.Kind)
	if err != nil {
		return nil, internal(err)
	}
	out := make([]*localv1.PpmEntity, 0, len(items))
	for _, item := range items {
		out = append(out, ppmEntityProto(item))
	}
	return connect.NewResponse(&localv1.ListEntitiesResponse{Entities: out}), nil
}

func (core *Core) ListRelationships(_ context.Context, request *connect.Request[localv1.ListRelationshipsRequest]) (*connect.Response[localv1.ListRelationshipsResponse], error) {
	if request.Msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	items, err := core.Store.ListPpmRelationships(request.Msg.ProjectId, request.Msg.EntityId)
	if err != nil {
		return nil, internal(err)
	}
	out := make([]*localv1.PpmRelationship, 0, len(items))
	for _, item := range items {
		out = append(out, ppmRelationshipProto(item))
	}
	return connect.NewResponse(&localv1.ListRelationshipsResponse{Relationships: out}), nil
}

func (core *Core) GetEntity(_ context.Context, request *connect.Request[localv1.GetEntityRequest]) (*connect.Response[localv1.GetEntityResponse], error) {
	if request.Msg.Id == "" {
		return nil, invalid("entity id is required")
	}
	item, err := core.Store.GetPpmEntity(request.Msg.Id)
	if errors.Is(err, sql.ErrNoRows) {
		return nil, connect.NewError(connect.CodeNotFound, ppm.ErrUnknownEntity)
	}
	if err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.GetEntityResponse{Entity: ppmEntityProto(item)}), nil
}

func (core *Core) SyncProjectModel(_ context.Context, request *connect.Request[localv1.SyncProjectModelRequest]) (*connect.Response[localv1.SyncProjectModelResponse], error) {
	if request.Msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	count, err := core.Store.SyncProjectModel(request.Msg.ProjectId)
	if err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.SyncProjectModelResponse{EntityCount: int32(count)}), nil
}

func (core *Core) SearchKnowledge(_ context.Context, request *connect.Request[localv1.SearchKnowledgeRequest]) (*connect.Response[localv1.SearchKnowledgeResponse], error) {
	if request.Msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	hits, err := core.Store.SearchKnowledge(request.Msg.ProjectId, request.Msg.Query, request.Msg.Kind, int(request.Msg.Limit))
	if err != nil {
		return nil, internal(err)
	}
	out := make([]*localv1.KnowledgeSearchHit, 0, len(hits))
	for _, hit := range hits {
		out = append(out, &localv1.KnowledgeSearchHit{
			Item: knowledgeProto(hit.Item), Score: hit.Score, Snippet: hit.Snippet,
			TruthClass: hit.TruthClass, Source: hit.Source,
		})
	}
	return connect.NewResponse(&localv1.SearchKnowledgeResponse{Hits: out}), nil
}

func (core *Core) ListKnowledgeLinks(_ context.Context, request *connect.Request[localv1.ListKnowledgeLinksRequest]) (*connect.Response[localv1.ListKnowledgeLinksResponse], error) {
	if request.Msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	items, err := core.Store.ListKnowledgeLinks(request.Msg.ProjectId, request.Msg.KnowledgeId, request.Msg.EntityId)
	if err != nil {
		return nil, internal(err)
	}
	out := make([]*localv1.KnowledgeEntityLink, 0, len(items))
	for _, item := range items {
		out = append(out, knowledgeLinkProto(item))
	}
	return connect.NewResponse(&localv1.ListKnowledgeLinksResponse{Links: out}), nil
}

func (core *Core) SaveKnowledgeLink(_ context.Context, request *connect.Request[localv1.SaveKnowledgeLinkRequest]) (*connect.Response[localv1.SaveKnowledgeLinkResponse], error) {
	msg := request.Msg.GetLink()
	if msg == nil {
		return nil, invalid("link is required")
	}
	if !ppm.ValidPredicate(msg.Predicate) {
		return nil, invalid("invalid predicate")
	}
	link, err := knowledge.NewEntityLink(msg.ProjectId, msg.KnowledgeId, msg.EntityId, msg.Predicate, msg.TruthClass, msg.Source, msg.Actor)
	if err != nil {
		return nil, invalid(err.Error())
	}
	if msg.Id != "" {
		link.ID = msg.Id
	}
	if _, err := core.Store.GetPpmEntity(link.EntityID); err != nil {
		return nil, invalid("entity not found")
	}
	if err := core.Store.SaveKnowledgeLink(link); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.SaveKnowledgeLinkResponse{Link: knowledgeLinkProto(link)}), nil
}

func (core *Core) DeleteKnowledgeLink(_ context.Context, request *connect.Request[localv1.DeleteKnowledgeLinkRequest]) (*connect.Response[localv1.DeleteKnowledgeLinkResponse], error) {
	if request.Msg.Id == "" {
		return nil, invalid("link id is required")
	}
	if err := core.Store.DeleteKnowledgeLink(request.Msg.Id); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.DeleteKnowledgeLinkResponse{}), nil
}

func (core *Core) CompileContext(_ context.Context, request *connect.Request[localv1.CompileContextRequest]) (*connect.Response[localv1.CompileContextResponse], error) {
	if request.Msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	_, _ = core.Store.SyncProjectModel(request.Msg.ProjectId)
	pkg, err := core.Store.CompileContext(contextpkg.Request{
		ProjectID:   request.Msg.ProjectId,
		Query:       request.Msg.Query,
		EntityID:    request.Msg.EntityId,
		TokenBudget: int(request.Msg.TokenBudget),
	})
	summary := "compile failed"
	ok := err == nil
	if err == nil {
		summary = agentaccess.SummarizeResult(
			fmt.Sprintf("items=%d tokens=%d truncated=%v", len(pkg.Items), pkg.TokensUsed, pkg.Truncated),
			240,
		)
	} else {
		summary = agentaccess.SummarizeResult(err.Error(), 240)
	}
	_ = core.Store.AppendAgentToolCall(agentaccess.ToolCall{
		ProjectID:     request.Msg.ProjectId,
		ToolName:      "compile_context",
		ArgumentsJSON: fmt.Sprintf(`{"query":%q,"entity_id":%q,"token_budget":%d}`, request.Msg.Query, request.Msg.EntityId, request.Msg.TokenBudget),
		ResultSummary: summary,
		OK:            ok,
		Source:        "desktop",
	})
	if err != nil {
		return nil, internal(err)
	}
	items := make([]*localv1.ContextItem, 0, len(pkg.Items))
	for _, item := range pkg.Items {
		items = append(items, &localv1.ContextItem{
			Kind: item.Kind, Id: item.ID, Title: item.Title, Excerpt: item.Excerpt,
			TruthClass: item.TruthClass, Source: item.Source, Path: item.Path,
		})
	}
	return connect.NewResponse(&localv1.CompileContextResponse{
		ProjectId: pkg.ProjectID, Query: pkg.Query, EntityId: pkg.EntityID,
		TokenBudget: int32(pkg.TokenBudget), TokensUsed: int32(pkg.TokensUsed),
		Truncated: pkg.Truncated, Items: items,
	}), nil
}

func (core *Core) ListAgentToolCalls(_ context.Context, request *connect.Request[localv1.ListAgentToolCallsRequest]) (*connect.Response[localv1.ListAgentToolCallsResponse], error) {
	calls, err := core.Store.ListAgentToolCalls(int(request.Msg.Limit))
	if err != nil {
		return nil, internal(err)
	}
	out := make([]*localv1.AgentToolCall, 0, len(calls))
	for _, call := range calls {
		out = append(out, &localv1.AgentToolCall{
			Id: call.ID, ProjectId: call.ProjectID, ToolName: call.ToolName,
			ArgumentsJson: call.ArgumentsJSON, ResultSummary: call.ResultSummary,
			Ok: call.OK, Source: call.Source, CreatedAt: call.CreatedAt.Format(time.RFC3339),
		})
	}
	return connect.NewResponse(&localv1.ListAgentToolCallsResponse{Calls: out}), nil
}

func ppmEntityProto(item ppm.Entity) *localv1.PpmEntity {
	return &localv1.PpmEntity{
		Id: item.ID, ProjectId: item.ProjectID, Kind: item.Kind, RefKind: item.RefKind, RefId: item.RefID,
		Title: item.Title, TruthClass: item.TruthClass, Source: item.Source, Actor: item.Actor,
		ObservedAt: item.ObservedAt.Format(time.RFC3339), CreatedAt: item.CreatedAt.Format(time.RFC3339),
		UpdatedAt: item.UpdatedAt.Format(time.RFC3339),
	}
}

func ppmRelationshipProto(item ppm.Relationship) *localv1.PpmRelationship {
	return &localv1.PpmRelationship{
		Id: item.ID, ProjectId: item.ProjectID, SubjectEntityId: item.SubjectEntityID,
		Predicate: item.Predicate, ObjectEntityId: item.ObjectEntityID, TruthClass: item.TruthClass,
		Source: item.Source, Actor: item.Actor, ObservedAt: item.ObservedAt.Format(time.RFC3339),
		CreatedAt: item.CreatedAt.Format(time.RFC3339),
	}
}

func knowledgeLinkProto(item knowledge.EntityLink) *localv1.KnowledgeEntityLink {
	return &localv1.KnowledgeEntityLink{
		Id: item.ID, ProjectId: item.ProjectID, KnowledgeId: item.KnowledgeID, EntityId: item.EntityID,
		Predicate: item.Predicate, TruthClass: item.TruthClass, Source: item.Source, Actor: item.Actor,
		ObservedAt: item.ObservedAt.Format(time.RFC3339), CreatedAt: item.CreatedAt.Format(time.RFC3339),
	}
}
