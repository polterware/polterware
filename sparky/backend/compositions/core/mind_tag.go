package core

import (
	"connectrpc.com/connect"
	"context"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/tag"
	v1 "github.com/polterware/polterware/sparky/backend/platform/gen/sparky/v1"
)

type mindHandler struct{ s mind.Service }

func (h mindHandler) List(ctx context.Context, _ *connect.Request[v1.ListMindsRequest]) (*connect.Response[v1.ListMindsResponse], error) {
	if h.s == nil {
		return nil, unavailable("minds")
	}
	items, err := h.s.List(ctx)
	out := &v1.ListMindsResponse{}
	for _, m := range items {
		out.Minds = append(out.Minds, mindProto(m))
	}
	return response(out, err)
}
func (h mindHandler) Get(ctx context.Context, r *connect.Request[v1.GetMindRequest]) (*connect.Response[v1.GetMindResponse], error) {
	if h.s == nil {
		return nil, unavailable("minds")
	}
	m, err := h.s.Get(ctx, r.Msg.Id)
	return response(&v1.GetMindResponse{Mind: mindProto(m)}, err)
}
func (h mindHandler) Create(ctx context.Context, r *connect.Request[v1.CreateMindRequest]) (*connect.Response[v1.CreateMindResponse], error) {
	if h.s == nil {
		return nil, unavailable("minds")
	}
	in := r.Msg
	m, err := h.s.Create(ctx, caller(ctx), mind.Create{ParentID: stringPtr(in.ParentId), Name: in.Name, Color: stringPtr(in.Color), Icon: stringPtr(in.Icon), SortOrder: int(in.SortOrder)})
	return response(&v1.CreateMindResponse{Mind: mindProto(m)}, err)
}
func (h mindHandler) Update(ctx context.Context, r *connect.Request[v1.UpdateMindRequest]) (*connect.Response[v1.UpdateMindResponse], error) {
	if h.s == nil {
		return nil, unavailable("minds")
	}
	in := r.Msg
	m, err := h.s.Update(ctx, caller(ctx), in.Id, in.GetExpectedRev(), mind.Patch{Name: in.Name, ParentID: in.ParentId, ClearParent: in.ClearParent, Color: in.Color, ClearColor: in.ClearColor, Icon: in.Icon, ClearIcon: in.ClearIcon, SortOrder: intPtr(in.SortOrder)})
	return response(&v1.UpdateMindResponse{Mind: mindProto(m)}, err)
}
func (h mindHandler) Delete(ctx context.Context, r *connect.Request[v1.DeleteMindRequest]) (*connect.Response[v1.DeleteMindResponse], error) {
	if h.s == nil {
		return nil, unavailable("minds")
	}
	d, err := h.s.Delete(ctx, caller(ctx), r.Msg.Id, r.Msg.GetExpectedRev())
	return response(&v1.DeleteMindResponse{DeletedMindIds: d.DeletedMindIDs, MovedMemoryCount: int32(d.MovedMemoryCount)}, err)
}
func (h mindHandler) Reorder(ctx context.Context, r *connect.Request[v1.ReorderMindsRequest]) (*connect.Response[v1.ReorderMindsResponse], error) {
	if h.s == nil {
		return nil, unavailable("minds")
	}
	items, err := h.s.Reorder(ctx, caller(ctx), stringPtr(r.Msg.ParentId), r.Msg.OrderedIds)
	out := &v1.ReorderMindsResponse{}
	for _, m := range items {
		out.Minds = append(out.Minds, mindProto(m))
	}
	return response(out, err)
}

type tagHandler struct{ s tag.Service }

func (h tagHandler) List(ctx context.Context, _ *connect.Request[v1.ListTagsRequest]) (*connect.Response[v1.ListTagsResponse], error) {
	if h.s == nil {
		return nil, unavailable("tags")
	}
	items, err := h.s.List(ctx)
	out := &v1.ListTagsResponse{}
	for _, t := range items {
		out.Tags = append(out.Tags, tagProto(t))
	}
	return response(out, err)
}
func (h tagHandler) Create(ctx context.Context, r *connect.Request[v1.CreateTagRequest]) (*connect.Response[v1.CreateTagResponse], error) {
	if h.s == nil {
		return nil, unavailable("tags")
	}
	t, err := h.s.Create(ctx, caller(ctx), r.Msg.Name, stringPtr(r.Msg.Color))
	return response(&v1.CreateTagResponse{Tag: tagProto(t)}, err)
}
func (h tagHandler) Update(ctx context.Context, r *connect.Request[v1.UpdateTagRequest]) (*connect.Response[v1.UpdateTagResponse], error) {
	if h.s == nil {
		return nil, unavailable("tags")
	}
	in := r.Msg
	t, err := h.s.Update(ctx, caller(ctx), in.Id, in.GetExpectedRev(), tag.Patch{Name: in.Name, Color: in.Color, ClearColor: in.ClearColor})
	return response(&v1.UpdateTagResponse{Tag: tagProto(t)}, err)
}
func (h tagHandler) Delete(ctx context.Context, r *connect.Request[v1.DeleteTagRequest]) (*connect.Response[v1.DeleteTagResponse], error) {
	if h.s == nil {
		return nil, unavailable("tags")
	}
	err := h.s.Delete(ctx, caller(ctx), r.Msg.Id)
	return response(&v1.DeleteTagResponse{}, err)
}
func (h tagHandler) Merge(ctx context.Context, r *connect.Request[v1.MergeTagsRequest]) (*connect.Response[v1.MergeTagsResponse], error) {
	if h.s == nil {
		return nil, unavailable("tags")
	}
	m, err := h.s.Merge(ctx, caller(ctx), r.Msg.SourceId, r.Msg.TargetId)
	return response(&v1.MergeTagsResponse{Target: tagProto(m.Target), MovedMemoryCount: int32(m.MovedMemoryCount)}, err)
}
