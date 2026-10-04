package core

import (
	"connectrpc.com/connect"
	"context"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	v1 "github.com/polterware/polterware/sparky/backend/platform/gen/sparky/v1"
)

type memoryHandler struct{ services Services }

func (h memoryHandler) List(ctx context.Context, r *connect.Request[v1.ListMemoriesRequest]) (*connect.Response[v1.ListMemoriesResponse], error) {
	if h.services.Memories == nil {
		return nil, unavailable("memories")
	}
	in := r.Msg
	f := memory.ListFilter{MindID: stringPtr(in.MindId), IncludeDescendants: in.IncludeDescendants, InboxOnly: in.InboxOnly, Status: in.Status, Pinned: in.Pinned, TagIDs: in.TagIds, DueFrom: instantPtr(in.DueFrom), DueTo: instantPtr(in.DueTo), UpdatedFrom: instantPtr(in.UpdatedFrom), UpdatedTo: instantPtr(in.UpdatedTo), HasSchedule: in.HasSchedule, HasPlace: in.HasPlace, Query: in.Query, Order: in.Order, Limit: int(in.Limit), Cursor: in.Cursor}
	page, err := h.services.Memories.List(ctx, f)
	out := &v1.ListMemoriesResponse{NextCursor: page.NextCursor, Total: int32(page.Total)}
	for _, m := range page.Memories {
		out.Memories = append(out.Memories, summaryProto(m))
	}
	return response(out, err)
}
func (h memoryHandler) Get(ctx context.Context, r *connect.Request[v1.GetMemoryRequest]) (*connect.Response[v1.GetMemoryResponse], error) {
	m, err := h.get(ctx, r.Msg.Id, r.Msg.Include)
	return response(&v1.GetMemoryResponse{Memory: m}, err)
}
func (h memoryHandler) get(ctx context.Context, id string, include *v1.MemoryInclude) (*v1.Memory, error) {
	if h.services.Memories == nil {
		return nil, unavailable("memories")
	}
	i := memory.Include{Note: include.GetNote(), Checklist: include.GetChecklist(), Completions: include.GetCompletions(), Tags: include.GetTags()}
	m, err := h.services.Memories.Get(ctx, id, i)
	if err != nil {
		return nil, err
	}
	out := memoryProto(m.Memory)
	if !i.Note {
		out.Note = ""
	}
	for _, item := range m.Checklist {
		out.Checklist = append(out.Checklist, checkProto(item))
	}
	for _, c := range m.Completions {
		out.Completions = append(out.Completions, completionProto(c))
	}
	if i.Tags {
		tags, err := h.tags(ctx, m.TagIDs)
		if err != nil {
			return nil, err
		}
		out.Tags = tags
	}
	if include.GetSchedule() {
		if h.services.ScheduleReader == nil {
			return nil, unavailable("schedule reader")
		}
		s, err := h.services.ScheduleReader.Get(ctx, id)
		if err != nil && !fault.IsCode(err, fault.CodeNotFound) {
			return nil, err
		}
		if err == nil {
			out.Schedule = scheduleProto(s)
		}
	}
	if include.GetPlace() {
		if h.services.PlaceReader == nil {
			return nil, unavailable("place reader")
		}
		p, err := h.services.PlaceReader.Get(ctx, id)
		if err != nil && !fault.IsCode(err, fault.CodeNotFound) {
			return nil, err
		}
		if err == nil {
			out.Place = placeProto(p)
		}
	}
	if include.GetAttachments() {
		if h.services.Attachments == nil {
			return nil, unavailable("attachments")
		}
		items, err := h.services.Attachments.List(ctx, id)
		if err != nil {
			return nil, err
		}
		for _, a := range items {
			out.Attachments = append(out.Attachments, attachmentProto(a))
		}
	}
	return out, nil
}
func (h memoryHandler) tags(ctx context.Context, ids []string) ([]*v1.Tag, error) {
	if len(ids) == 0 {
		return nil, nil
	}
	if h.services.Tags == nil {
		return nil, unavailable("tags")
	}
	all, err := h.services.Tags.List(ctx)
	if err != nil {
		return nil, err
	}
	wanted := make(map[string]bool, len(ids))
	for _, id := range ids {
		wanted[id] = true
	}
	out := make([]*v1.Tag, 0, len(ids))
	for _, tag := range all {
		if wanted[tag.ID] {
			out = append(out, tagProto(tag))
		}
	}
	return out, nil
}
func (h memoryHandler) Create(ctx context.Context, r *connect.Request[v1.CreateMemoryRequest]) (*connect.Response[v1.CreateMemoryResponse], error) {
	if h.services.Memories == nil {
		return nil, unavailable("memories")
	}
	in := r.Msg
	m, err := h.services.Memories.Create(ctx, caller(ctx), memory.Create{MindID: stringPtr(in.MindId), Title: in.Title, Note: in.Note, Pinned: in.Pinned, Priority: intPtr(in.Priority), DueAt: instantPtr(in.DueAt), DueAllDay: in.DueAllDay, SortOrder: int(in.SortOrder), AutoCompleteOnChecklist: in.AutoCompleteOnChecklist, TagIDs: in.TagIds, Checklist: checkInputs(in.Checklist)})
	return response(&v1.CreateMemoryResponse{Memory: memoryProto(m)}, err)
}
func (h memoryHandler) Update(ctx context.Context, r *connect.Request[v1.UpdateMemoryRequest]) (*connect.Response[v1.UpdateMemoryResponse], error) {
	if h.services.Memories == nil {
		return nil, unavailable("memories")
	}
	in := r.Msg
	p := memory.Patch{Title: in.Title, Note: in.Note, MindID: in.MindId, ClearMindID: in.ClearMindId, Priority: intPtr(in.Priority), ClearPriority: in.ClearPriority, DueAt: instantPtr(in.DueAt), ClearDueAt: in.ClearDueAt, DueAllDay: in.DueAllDay, SortOrder: intPtr(in.SortOrder), Pinned: in.Pinned, AutoCompleteOnChecklist: in.AutoCompleteOnChecklist}
	m, err := h.services.Memories.Update(ctx, caller(ctx), in.Id, in.GetExpectedRev(), p)
	return response(&v1.UpdateMemoryResponse{Memory: memoryProto(m)}, err)
}
func (h memoryHandler) Delete(ctx context.Context, r *connect.Request[v1.DeleteMemoryRequest]) (*connect.Response[v1.DeleteMemoryResponse], error) {
	if h.services.Memories == nil {
		return nil, unavailable("memories")
	}
	err := h.services.Memories.Delete(ctx, caller(ctx), r.Msg.Id)
	return response(&v1.DeleteMemoryResponse{}, err)
}
func (h memoryHandler) SetStatus(ctx context.Context, r *connect.Request[v1.SetMemoryStatusRequest]) (*connect.Response[v1.SetMemoryStatusResponse], error) {
	if h.services.Memories == nil {
		return nil, unavailable("memories")
	}
	in := r.Msg
	m, err := h.services.Memories.SetStatus(ctx, caller(ctx), in.Id, in.GetExpectedRev(), in.Status, instantPtr(in.OccurrenceAt))
	return response(&v1.SetMemoryStatusResponse{Memory: memoryProto(m)}, err)
}
func (h memoryHandler) SetPinned(ctx context.Context, r *connect.Request[v1.SetMemoryPinnedRequest]) (*connect.Response[v1.SetMemoryPinnedResponse], error) {
	if h.services.Memories == nil {
		return nil, unavailable("memories")
	}
	in := r.Msg
	m, err := h.services.Memories.SetPinned(ctx, caller(ctx), in.Id, in.GetExpectedRev(), in.Pinned)
	return response(&v1.SetMemoryPinnedResponse{Memory: memoryProto(m)}, err)
}
func (h memoryHandler) Reorder(ctx context.Context, r *connect.Request[v1.ReorderMemoriesRequest]) (*connect.Response[v1.ReorderMemoriesResponse], error) {
	if h.services.Memories == nil {
		return nil, unavailable("memories")
	}
	err := h.services.Memories.Reorder(ctx, caller(ctx), stringPtr(r.Msg.MindId), r.Msg.OrderedIds)
	return response(&v1.ReorderMemoriesResponse{}, err)
}
func (h memoryHandler) ToggleCheckItem(ctx context.Context, r *connect.Request[v1.ToggleCheckItemRequest]) (*connect.Response[v1.ToggleCheckItemResponse], error) {
	if h.services.Memories == nil {
		return nil, unavailable("memories")
	}
	in := r.Msg
	c, err := h.services.Memories.ToggleCheckItem(ctx, caller(ctx), in.MemoryId, in.ItemId, in.Done)
	if err != nil {
		return nil, err
	}
	m, err := h.get(ctx, in.MemoryId, nil)
	return response(&v1.ToggleCheckItemResponse{Item: checkProto(c), Memory: m}, err)
}
func (h memoryHandler) ReplaceChecklist(ctx context.Context, r *connect.Request[v1.ReplaceChecklistRequest]) (*connect.Response[v1.ReplaceChecklistResponse], error) {
	if h.services.Memories == nil {
		return nil, unavailable("memories")
	}
	in := r.Msg
	items, err := h.services.Memories.ReplaceChecklist(ctx, caller(ctx), in.MemoryId, in.GetExpectedRev(), checkInputs(in.Items))
	if err != nil {
		return nil, err
	}
	out := &v1.ReplaceChecklistResponse{}
	for _, c := range items {
		out.Items = append(out.Items, checkProto(c))
	}
	out.Memory, err = h.get(ctx, in.MemoryId, nil)
	return response(out, err)
}
func (h memoryHandler) SetTags(ctx context.Context, r *connect.Request[v1.SetMemoryTagsRequest]) (*connect.Response[v1.SetMemoryTagsResponse], error) {
	if h.services.Memories == nil {
		return nil, unavailable("memories")
	}
	in := r.Msg
	if len(in.TagIds) > 0 && h.services.Tags == nil {
		return nil, unavailable("tags")
	}
	if err := h.services.Memories.SetTags(ctx, caller(ctx), in.MemoryId, in.GetExpectedRev(), in.TagIds); err != nil {
		return nil, err
	}
	m, err := h.get(ctx, in.MemoryId, &v1.MemoryInclude{Tags: true})
	if err != nil {
		return nil, err
	}
	return response(&v1.SetMemoryTagsResponse{Memory: m, Tags: m.Tags}, nil)
}
func (h memoryHandler) BulkAction(ctx context.Context, r *connect.Request[v1.BulkMemoryActionRequest]) (*connect.Response[v1.BulkMemoryActionResponse], error) {
	if h.services.Memories == nil {
		return nil, unavailable("memories")
	}
	in := r.Msg
	items, err := h.services.Memories.BulkAction(ctx, caller(ctx), memory.BulkAction{IDs: in.Ids, Action: in.Action, MindID: stringPtr(in.MindId), TagIDs: in.TagIds})
	out := &v1.BulkMemoryActionResponse{}
	for _, item := range items {
		result := &v1.BulkMemoryItemResult{Id: item.ID, Ok: item.Err == nil}
		if item.Err != nil {
			result.Error = mapError(item.Err).Error()
		}
		out.Results = append(out.Results, result)
	}
	return response(out, err)
}
