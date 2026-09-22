package localcore

import (
	"context"
	"database/sql"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"connectrpc.com/connect"

	"github.com/polterware/polterware/backend/domains/agentaccess"
	contextpkg "github.com/polterware/polterware/backend/domains/context"
	"github.com/polterware/polterware/backend/domains/knowledge"
	"github.com/polterware/polterware/backend/domains/ppm"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
	"github.com/polterware/polterware/backend/providers/importer"
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

func (core *Core) ListFacts(_ context.Context, request *connect.Request[localv1.ListFactsRequest]) (*connect.Response[localv1.ListFactsResponse], error) {
	if request.Msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	items, err := core.Store.ListPpmFacts(request.Msg.ProjectId, request.Msg.EntityId)
	if err != nil {
		return nil, internal(err)
	}
	out := make([]*localv1.PpmFact, 0, len(items))
	for _, item := range items {
		out = append(out, ppmFactProto(item))
	}
	return connect.NewResponse(&localv1.ListFactsResponse{Facts: out}), nil
}

func (core *Core) GetFact(_ context.Context, request *connect.Request[localv1.GetFactRequest]) (*connect.Response[localv1.GetFactResponse], error) {
	if request.Msg.Id == "" {
		return nil, invalid("fact id is required")
	}
	fact, err := core.Store.GetPpmFact(request.Msg.Id)
	if errors.Is(err, sql.ErrNoRows) {
		return nil, connect.NewError(connect.CodeNotFound, ppm.ErrUnknownFact)
	}
	if err != nil {
		return nil, internal(err)
	}
	evidence, err := core.Store.ListPpmEvidenceForFact(fact.ID)
	if err != nil {
		return nil, internal(err)
	}
	out := make([]*localv1.PpmEvidence, 0, len(evidence))
	for _, item := range evidence {
		out = append(out, ppmEvidenceProto(item))
	}
	return connect.NewResponse(&localv1.GetFactResponse{Fact: ppmFactProto(fact), Evidence: out}), nil
}

func (core *Core) UpsertFact(_ context.Context, request *connect.Request[localv1.UpsertFactRequest]) (*connect.Response[localv1.UpsertFactResponse], error) {
	msg := request.Msg.GetFact()
	if msg == nil {
		return nil, invalid("fact is required")
	}
	fact, err := ppmFactFromProto(msg)
	if err != nil {
		return nil, invalid(err.Error())
	}
	if fact.EntityID != "" {
		if _, err := core.Store.GetPpmEntity(fact.EntityID); err != nil {
			return nil, invalid("entity not found")
		}
	}
	if err := core.Store.UpsertPpmFact(fact); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.UpsertFactResponse{Fact: ppmFactProto(fact)}), nil
}

func (core *Core) ListEvidence(_ context.Context, request *connect.Request[localv1.ListEvidenceRequest]) (*connect.Response[localv1.ListEvidenceResponse], error) {
	if request.Msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	items, err := core.Store.ListPpmEvidence(request.Msg.ProjectId)
	if err != nil {
		return nil, internal(err)
	}
	out := make([]*localv1.PpmEvidence, 0, len(items))
	for _, item := range items {
		out = append(out, ppmEvidenceProto(item))
	}
	return connect.NewResponse(&localv1.ListEvidenceResponse{Evidence: out}), nil
}

func (core *Core) GetEvidence(_ context.Context, request *connect.Request[localv1.GetEvidenceRequest]) (*connect.Response[localv1.GetEvidenceResponse], error) {
	if request.Msg.Id == "" {
		return nil, invalid("evidence id is required")
	}
	item, err := core.Store.GetPpmEvidence(request.Msg.Id)
	if errors.Is(err, sql.ErrNoRows) {
		return nil, connect.NewError(connect.CodeNotFound, ppm.ErrUnknownEvidence)
	}
	if err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.GetEvidenceResponse{Evidence: ppmEvidenceProto(item)}), nil
}

func (core *Core) UpsertEvidence(_ context.Context, request *connect.Request[localv1.UpsertEvidenceRequest]) (*connect.Response[localv1.UpsertEvidenceResponse], error) {
	msg := request.Msg.GetEvidence()
	if msg == nil {
		return nil, invalid("evidence is required")
	}
	evidence, err := ppmEvidenceFromProto(msg)
	if err != nil {
		return nil, invalid(err.Error())
	}
	if err := core.Store.UpsertPpmEvidence(evidence); err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.UpsertEvidenceResponse{Evidence: ppmEvidenceProto(evidence)}), nil
}

func (core *Core) LinkEvidenceToFact(_ context.Context, request *connect.Request[localv1.LinkEvidenceToFactRequest]) (*connect.Response[localv1.LinkEvidenceToFactResponse], error) {
	if request.Msg.ProjectId == "" || request.Msg.FactId == "" || request.Msg.EvidenceId == "" {
		return nil, invalid("project id, fact id, and evidence id are required")
	}
	err := core.Store.LinkPpmEvidenceToFact(request.Msg.ProjectId, request.Msg.FactId, request.Msg.EvidenceId)
	if errors.Is(err, sql.ErrNoRows) || errors.Is(err, ppm.ErrUnknownFact) || errors.Is(err, ppm.ErrUnknownEvidence) {
		return nil, connect.NewError(connect.CodeNotFound, err)
	}
	if err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.LinkEvidenceToFactResponse{}), nil
}

func (core *Core) ImportProjectSources(_ context.Context, request *connect.Request[localv1.ImportProjectSourcesRequest]) (*connect.Response[localv1.ImportProjectSourcesResponse], error) {
	projectID := request.Msg.ProjectId
	if projectID == "" {
		return nil, invalid("project id is required")
	}
	project, err := core.Store.GetProject(projectID)
	if errors.Is(err, sql.ErrNoRows) {
		return nil, connect.NewError(connect.CodeNotFound, fmt.Errorf("unknown project"))
	}
	if err != nil {
		return nil, internal(err)
	}
	rootPath := expandHome(project.RootPath)
	if rootPath == "" {
		return nil, invalid("project root path is empty")
	}
	discovered, err := importer.Discover(rootPath)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return nil, invalid("project root path does not exist")
		}
		return nil, internal(err)
	}

	now := time.Now().UTC()
	var factCount, evidenceCount, sourceCount int32
	skipped := int32(discovered.Skipped)
	importedFiles := 0

	existingSources, err := core.Store.ListKnowledge(projectID, "source")
	if err != nil {
		return nil, internal(err)
	}
	sourceByPath := map[string]knowledge.Item{}
	for _, item := range existingSources {
		if path := knowledgeSourcePath(item.ExtraJSON); path != "" {
			sourceByPath[path] = item
		}
	}

	for _, file := range discovered.Files {
		if file.Skipped {
			continue
		}
		evidence, facts, err := importer.Materialize(projectID, file, now)
		if err != nil {
			skipped++
			continue
		}
		if existing, err := core.Store.GetPpmEvidence(evidence.ID); err == nil {
			evidence.CreatedAt = existing.CreatedAt
		}
		if err := core.Store.UpsertPpmEvidence(evidence); err != nil {
			return nil, internal(err)
		}
		evidenceCount++
		for _, fact := range facts {
			if existing, err := core.Store.GetPpmFact(fact.ID); err == nil {
				fact.CreatedAt = existing.CreatedAt
			}
			if err := core.Store.UpsertPpmFact(fact); err != nil {
				return nil, internal(err)
			}
			if err := core.Store.LinkPpmEvidenceToFact(projectID, fact.ID, evidence.ID); err != nil {
				return nil, internal(err)
			}
			factCount++
		}
		if err := core.upsertImportedKnowledgeSource(projectID, file, sourceByPath, now); err != nil {
			return nil, internal(err)
		}
		sourceCount++
		importedFiles++
	}

	message := "No matching MVP sources found under project root."
	if importedFiles > 0 {
		message = fmt.Sprintf("Imported %d facts from %d files.", factCount, importedFiles)
	}

	return connect.NewResponse(&localv1.ImportProjectSourcesResponse{
		FactCount:     factCount,
		EvidenceCount: evidenceCount,
		SourceCount:   sourceCount,
		SkippedCount:  skipped,
		Message:       message,
	}), nil
}

func (core *Core) upsertImportedKnowledgeSource(projectID string, file importer.FileResult, sourceByPath map[string]knowledge.Item, now time.Time) error {
	path := filepath.ToSlash(file.Path)
	title := file.Title
	if title == "" {
		title = filepath.Base(path)
	}
	extra, _ := json.Marshal(map[string]string{"path": path})
	if existing, ok := sourceByPath[path]; ok {
		existing.Title = title
		existing.ExtraJSON = string(extra)
		existing.UpdatedAt = now
		if existing.Summary == "" {
			existing.Summary = file.Evidence.Summary
		}
		return core.Store.SaveKnowledge(existing)
	}
	item, err := knowledge.NewItem(importer.StableSourceID(projectID, path), projectID, title, "", "source", nil)
	if err != nil {
		return err
	}
	item.Summary = file.Evidence.Summary
	item.ExtraJSON = string(extra)
	item.CreatedAt = now
	item.UpdatedAt = now
	sourceByPath[path] = item
	return core.Store.SaveKnowledge(item)
}

func knowledgeSourcePath(extraJSON string) string {
	if extraJSON == "" {
		return ""
	}
	var payload struct {
		Path string `json:"path"`
	}
	if err := json.Unmarshal([]byte(extraJSON), &payload); err != nil {
		return ""
	}
	return filepath.ToSlash(payload.Path)
}

func expandHome(path string) string {
	if path == "" {
		return ""
	}
	if path == "~" {
		home, err := os.UserHomeDir()
		if err != nil {
			return path
		}
		return home
	}
	if len(path) >= 2 && path[0] == '~' && (path[1] == '/' || path[1] == filepath.Separator) {
		home, err := os.UserHomeDir()
		if err != nil {
			return path
		}
		return filepath.Join(home, path[2:])
	}
	return path
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
		ToolName:      "context.get",
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
		evidence := make([]*localv1.ContextEvidenceRef, 0, len(item.Evidence))
		for _, ev := range item.Evidence {
			evidence = append(evidence, &localv1.ContextEvidenceRef{
				Id: ev.ID, Kind: ev.Kind, Summary: ev.Summary, Path: ev.Path, Source: ev.Source,
			})
		}
		items = append(items, &localv1.ContextItem{
			Kind: item.Kind, Id: item.ID, Title: item.Title, Excerpt: item.Excerpt,
			TruthClass: item.TruthClass, Source: item.Source, Path: item.Path,
			Reasons: item.Reasons, Evidence: evidence,
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

func ppmFactProto(item ppm.Fact) *localv1.PpmFact {
	return &localv1.PpmFact{
		Id: item.ID, ProjectId: item.ProjectID, Statement: item.Statement, TruthClass: item.TruthClass,
		Source: item.Source, Actor: item.Actor, ObservedAt: item.ObservedAt.Format(time.RFC3339),
		EntityId: item.EntityID, CreatedAt: item.CreatedAt.Format(time.RFC3339),
		UpdatedAt: item.UpdatedAt.Format(time.RFC3339),
	}
}

func ppmEvidenceProto(item ppm.Evidence) *localv1.PpmEvidence {
	return &localv1.PpmEvidence{
		Id: item.ID, ProjectId: item.ProjectID, Kind: item.Kind, Summary: item.Summary,
		Source: item.Source, Path: item.Path, Actor: item.Actor,
		ObservedAt: item.ObservedAt.Format(time.RFC3339), CreatedAt: item.CreatedAt.Format(time.RFC3339),
		UpdatedAt: item.UpdatedAt.Format(time.RFC3339),
	}
}

func ppmFactFromProto(msg *localv1.PpmFact) (ppm.Fact, error) {
	if msg.Id == "" {
		return ppm.NewFact(msg.ProjectId, msg.Statement, msg.TruthClass, msg.Source, msg.Actor, msg.EntityId)
	}
	now := time.Now().UTC()
	fact := ppm.Fact{
		ID: msg.Id, ProjectID: msg.ProjectId, Statement: msg.Statement, TruthClass: msg.TruthClass,
		Source: msg.Source, Actor: msg.Actor, EntityID: msg.EntityId,
		ObservedAt: now, CreatedAt: now, UpdatedAt: now,
	}
	if fact.TruthClass == "" {
		fact.TruthClass = ppm.TruthDeclared
	}
	if fact.Source == "" {
		fact.Source = "polterware"
	}
	if fact.Actor == "" {
		fact.Actor = "system"
	}
	if msg.ObservedAt != "" {
		if parsed, err := time.Parse(time.RFC3339, msg.ObservedAt); err == nil {
			fact.ObservedAt = parsed
		}
	}
	if msg.CreatedAt != "" {
		if parsed, err := time.Parse(time.RFC3339, msg.CreatedAt); err == nil {
			fact.CreatedAt = parsed
		}
	}
	if err := ppm.ValidateFact(fact); err != nil {
		return ppm.Fact{}, err
	}
	fact.UpdatedAt = now
	return fact, nil
}

func ppmEvidenceFromProto(msg *localv1.PpmEvidence) (ppm.Evidence, error) {
	if msg.Id == "" {
		return ppm.NewEvidence(msg.ProjectId, msg.Kind, msg.Summary, msg.Source, msg.Path, msg.Actor)
	}
	now := time.Now().UTC()
	evidence := ppm.Evidence{
		ID: msg.Id, ProjectID: msg.ProjectId, Kind: msg.Kind, Summary: msg.Summary,
		Source: msg.Source, Path: msg.Path, Actor: msg.Actor,
		ObservedAt: now, CreatedAt: now, UpdatedAt: now,
	}
	if evidence.Source == "" {
		evidence.Source = "polterware"
	}
	if evidence.Actor == "" {
		evidence.Actor = "system"
	}
	if msg.ObservedAt != "" {
		if parsed, err := time.Parse(time.RFC3339, msg.ObservedAt); err == nil {
			evidence.ObservedAt = parsed
		}
	}
	if msg.CreatedAt != "" {
		if parsed, err := time.Parse(time.RFC3339, msg.CreatedAt); err == nil {
			evidence.CreatedAt = parsed
		}
	}
	if err := ppm.ValidateEvidence(evidence); err != nil {
		return ppm.Evidence{}, err
	}
	evidence.UpdatedAt = now
	return evidence, nil
}

func knowledgeLinkProto(item knowledge.EntityLink) *localv1.KnowledgeEntityLink {
	return &localv1.KnowledgeEntityLink{
		Id: item.ID, ProjectId: item.ProjectID, KnowledgeId: item.KnowledgeID, EntityId: item.EntityID,
		Predicate: item.Predicate, TruthClass: item.TruthClass, Source: item.Source, Actor: item.Actor,
		ObservedAt: item.ObservedAt.Format(time.RFC3339), CreatedAt: item.CreatedAt.Format(time.RFC3339),
	}
}
