package fake

import (
	"context"
	"encoding/base64"
	"encoding/json"
	store "github.com/polterware/polterware/sparky/backend/domains/activity/fake"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/focus"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/tag"
	"slices"
	"sort"
	"strings"
	"time"
)

type Repository struct{ Store *store.Store }

func New(s *store.Store) *Repository { return &Repository{s} }

var _ memory.Repository = (*Repository)(nil)

func (r *Repository) Get(ctx context.Context, id string) (memory.Memory, error) {
	v, ok, err := store.Read[memory.Memory](ctx, r.Store, "memories", id)
	if err == nil && !ok {
		err = fault.NotFound("memory", id)
	}
	return v, err
}
func (r *Repository) checkMind(ctx context.Context, m memory.Memory) error {
	if m.MindID != nil {
		_, ok, err := store.Read[mind.Mind](ctx, r.Store, "minds", *m.MindID)
		if err != nil {
			return err
		}
		if !ok {
			return fault.NotFound("mind", *m.MindID)
		}
	}
	return nil
}
func (r *Repository) Insert(ctx context.Context, m memory.Memory) error {
	return r.Store.Within(ctx, func(ctx context.Context) error {
		if _, err := r.Get(ctx, m.ID); err == nil {
			return fault.Exists("memory", m.ID)
		} else if !fault.IsCode(err, fault.CodeNotFound) {
			return err
		}
		if err := r.checkMind(ctx, m); err != nil {
			return err
		}
		return r.Store.Put(ctx, "memories", m.ID, m)
	})
}
func (r *Repository) Update(ctx context.Context, rev int64, m memory.Memory) error {
	return r.Store.Within(ctx, func(ctx context.Context) error {
		old, err := r.Get(ctx, m.ID)
		if err != nil {
			return err
		}
		if rev != 0 && old.Rev != rev {
			return fault.Conflict("memory", m.ID, old.Rev, old)
		}
		if err := r.checkMind(ctx, m); err != nil {
			return err
		}
		return r.Store.Put(ctx, "memories", m.ID, m)
	})
}
func (r *Repository) Delete(ctx context.Context, rev int64, id string) error {
	return r.Store.Within(ctx, func(ctx context.Context) error {
		old, err := r.Get(ctx, id)
		if err != nil {
			return err
		}
		if rev != 0 && old.Rev != rev {
			return fault.Conflict("memory", id, old.Rev, old)
		}
		items, err := r.ListCheckItems(ctx, id)
		if err != nil {
			return err
		}
		for _, item := range items {
			if err = r.Store.Remove(ctx, "check_items", item.ID); err != nil {
				return err
			}
		}
		cs, err := r.ListCompletions(ctx, id)
		if err != nil {
			return err
		}
		for _, c := range cs {
			if err = r.Store.Remove(ctx, "completions", c.ID); err != nil {
				return err
			}
		}
		attachments, err := store.All[struct{ ID, MemoryID string }](ctx, r.Store, "attachments")
		if err != nil {
			return err
		}
		for _, v := range attachments {
			if v.MemoryID == id {
				if err = r.Store.Remove(ctx, "attachments", v.ID); err != nil {
					return err
				}
			}
		}
		sessions, err := store.All[focus.Session](ctx, r.Store, "focus_sessions")
		if err != nil {
			return err
		}
		for _, v := range sessions {
			if v.MemoryID != nil && *v.MemoryID == id {
				v.MemoryID = nil
				if err = r.Store.Put(ctx, "focus_sessions", v.ID, v); err != nil {
					return err
				}
			}
		}
		for _, table := range []string{"memory_tags", "schedules", "places"} {
			if err = r.Store.Remove(ctx, table, id); err != nil {
				return err
			}
		}
		return r.Store.Remove(ctx, "memories", id)
	})
}
func (r *Repository) ListCheckItems(ctx context.Context, id string) ([]memory.CheckItem, error) {
	all, err := store.All[memory.CheckItem](ctx, r.Store, "check_items")
	out := []memory.CheckItem{}
	for _, v := range all {
		if v.MemoryID == id {
			out = append(out, v)
		}
	}
	sort.Slice(out, func(i, j int) bool {
		if out[i].SortOrder == out[j].SortOrder {
			return out[i].ID < out[j].ID
		}
		return out[i].SortOrder < out[j].SortOrder
	})
	return out, err
}
func (r *Repository) ReplaceCheckItems(ctx context.Context, id string, items []memory.CheckItem) error {
	return r.Store.Within(ctx, func(ctx context.Context) error {
		if _, err := r.Get(ctx, id); err != nil {
			return err
		}
		seen := map[string]bool{}
		for _, v := range items {
			if v.MemoryID != id || seen[v.ID] {
				return fault.Invalid(fault.Violation{Field: "checklist", Reason: "invalid identity"})
			}
			seen[v.ID] = true
			old, ok, err := store.Read[memory.CheckItem](ctx, r.Store, "check_items", v.ID)
			if err != nil {
				return err
			}
			if ok && old.MemoryID != id {
				return fault.Exists("check_item", v.ID)
			}
		}
		old, err := r.ListCheckItems(ctx, id)
		if err != nil {
			return err
		}
		for _, v := range old {
			if err = r.Store.Remove(ctx, "check_items", v.ID); err != nil {
				return err
			}
		}
		for _, v := range items {
			if err = r.Store.Put(ctx, "check_items", v.ID, v); err != nil {
				return err
			}
		}
		return nil
	})
}
func (r *Repository) GetCheckItem(ctx context.Context, id string) (memory.CheckItem, error) {
	v, ok, err := store.Read[memory.CheckItem](ctx, r.Store, "check_items", id)
	if err == nil && !ok {
		err = fault.NotFound("check_item", id)
	}
	return v, err
}
func (r *Repository) UpdateCheckItem(ctx context.Context, v memory.CheckItem) error {
	return r.Store.Within(ctx, func(ctx context.Context) error {
		old, err := r.GetCheckItem(ctx, v.ID)
		if err != nil {
			return err
		}
		if old.MemoryID != v.MemoryID {
			return fault.Invalid(fault.Violation{Field: "memory_id", Reason: "cannot move checklist item"})
		}
		return r.Store.Put(ctx, "check_items", v.ID, v)
	})
}

type tagLinks struct {
	MemoryID string
	IDs      []string
}

func (r *Repository) ListTagIDs(ctx context.Context, id string) ([]string, error) {
	v, _, err := store.Read[tagLinks](ctx, r.Store, "memory_tags", id)
	return v.IDs, err
}
func (r *Repository) SetTagIDs(ctx context.Context, id string, ids []string) error {
	return r.Store.Within(ctx, func(ctx context.Context) error {
		if _, err := r.Get(ctx, id); err != nil {
			return err
		}
		unique := []string{}
		for _, v := range ids {
			_, ok, err := store.Read[tag.Tag](ctx, r.Store, "tags", v)
			if err != nil {
				return err
			}
			if !ok {
				return fault.NotFound("tag", v)
			}
			if !slices.Contains(unique, v) {
				unique = append(unique, v)
			}
		}
		return r.Store.Put(ctx, "memory_tags", id, tagLinks{id, unique})
	})
}
func (r *Repository) ListCompletions(ctx context.Context, id string) ([]memory.Completion, error) {
	all, err := store.All[memory.Completion](ctx, r.Store, "completions")
	out := []memory.Completion{}
	for _, v := range all {
		if v.MemoryID == id {
			out = append(out, v)
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i].OccurrenceAt.Before(out[j].OccurrenceAt) })
	return out, err
}
func (r *Repository) InsertCompletion(ctx context.Context, c memory.Completion) error {
	return r.Store.Within(ctx, func(ctx context.Context) error {
		if _, err := r.Get(ctx, c.MemoryID); err != nil {
			return err
		}
		rows, err := r.ListCompletions(ctx, c.MemoryID)
		if err != nil {
			return err
		}
		for _, v := range rows {
			if v.OccurrenceAt.Equal(c.OccurrenceAt) {
				return fault.Exists("completion", v.ID)
			}
		}
		_, ok, err := store.Read[memory.Completion](ctx, r.Store, "completions", c.ID)
		if err != nil {
			return err
		}
		if ok {
			return fault.Exists("completion", c.ID)
		}
		return r.Store.Put(ctx, "completions", c.ID, c)
	})
}
func (r *Repository) DeleteCompletion(ctx context.Context, id string, at time.Time) error {
	rows, err := r.ListCompletions(ctx, id)
	if err != nil {
		return err
	}
	for _, v := range rows {
		if v.OccurrenceAt.Equal(at) {
			return r.Store.Remove(ctx, "completions", v.ID)
		}
	}
	return nil
}
func (r *Repository) CountByMinds(ctx context.Context, ids []string) (int, error) {
	rows, err := store.All[memory.Memory](ctx, r.Store, "memories")
	n := 0
	for _, v := range rows {
		if v.MindID != nil && slices.Contains(ids, *v.MindID) {
			n++
		}
	}
	return n, err
}
func less(a, b memory.Summary, order string) bool {
	switch order {
	case memory.OrderManual:
		if a.SortOrder != b.SortOrder {
			return a.SortOrder < b.SortOrder
		}
	case memory.OrderDue:
		if a.DueAt == nil && b.DueAt != nil {
			return false
		}
		if a.DueAt != nil && b.DueAt == nil {
			return true
		}
		if a.DueAt != nil && !a.DueAt.Equal(*b.DueAt) {
			return a.DueAt.Before(*b.DueAt)
		}
	case memory.OrderCreated:
		if !a.CreatedAt.Equal(b.CreatedAt) {
			return a.CreatedAt.After(b.CreatedAt)
		}
	default:
		if !a.UpdatedAt.Equal(b.UpdatedAt) {
			return a.UpdatedAt.After(b.UpdatedAt)
		}
	}
	return a.ID < b.ID
}
func (r *Repository) List(ctx context.Context, f memory.ListFilter) (memory.ListPage, error) {
	rows, err := store.All[memory.Memory](ctx, r.Store, "memories")
	if err != nil {
		return memory.ListPage{}, err
	}
	minds := map[string]bool{}
	if f.MindID != nil {
		minds[*f.MindID] = true
	}
	if f.IncludeDescendants {
		nodes, e := store.All[mind.Mind](ctx, r.Store, "minds")
		if e != nil {
			return memory.ListPage{}, e
		}
		for changed := true; changed; {
			changed = false
			for _, n := range nodes {
				if n.ParentID != nil && minds[*n.ParentID] && !minds[n.ID] {
					minds[n.ID] = true
					changed = true
				}
			}
		}
	}
	page := memory.ListPage{}
	for _, m := range rows {
		if f.InboxOnly && m.MindID != nil || f.MindID != nil && (m.MindID == nil || !minds[*m.MindID]) || f.Status != "" && m.Status != f.Status || f.Pinned != nil && m.Pinned != *f.Pinned {
			continue
		}
		if f.DueFrom != nil && (m.DueAt == nil || m.DueAt.Before(*f.DueFrom)) || f.DueTo != nil && (m.DueAt == nil || !m.DueAt.Before(*f.DueTo)) || f.UpdatedFrom != nil && m.UpdatedAt.Before(*f.UpdatedFrom) || f.UpdatedTo != nil && !m.UpdatedAt.Before(*f.UpdatedTo) {
			continue
		}
		tags, e := r.ListTagIDs(ctx, m.ID)
		if e != nil {
			return page, e
		}
		match := true
		for _, id := range f.TagIDs {
			if !slices.Contains(tags, id) {
				match = false
			}
		}
		if !match {
			continue
		}
		if f.Query != "" && !strings.Contains(strings.ToLower(m.Title+" "+m.Note), strings.ToLower(f.Query)) {
			continue
		}
		for table, want := range map[string]*bool{"schedules": f.HasSchedule, "places": f.HasPlace} {
			if want != nil {
				_, ok, e := store.Read[json.RawMessage](ctx, r.Store, table, m.ID)
				if e != nil {
					return page, e
				}
				if ok != *want {
					match = false
				}
			}
		}
		if !match {
			continue
		}
		snippet := []rune(m.Note)
		if len(snippet) > 120 {
			snippet = snippet[:120]
		}
		page.Memories = append(page.Memories, memory.Summary{ID: m.ID, Title: m.Title, Status: m.Status, MindID: m.MindID, DueAt: m.DueAt, DueAllDay: m.DueAllDay, Pinned: m.Pinned, Priority: m.Priority, TagIDs: tags, Snippet: string(snippet), Rev: m.Rev, UpdatedAt: m.UpdatedAt, CreatedAt: m.CreatedAt, SortOrder: m.SortOrder})
	}
	sort.Slice(page.Memories, func(i, j int) bool { return less(page.Memories[i], page.Memories[j], f.Order) })
	page.Total = len(page.Memories)
	if f.Cursor != "" {
		b, e := base64.RawURLEncoding.DecodeString(f.Cursor)
		var last memory.Summary
		if e != nil || json.Unmarshal(b, &last) != nil {
			return page, fault.Invalid(fault.Violation{Field: "cursor", Reason: "invalid cursor"})
		}
		i := 0
		for i < len(page.Memories) && !less(last, page.Memories[i], f.Order) {
			i++
		}
		page.Memories = page.Memories[i:]
	}
	limit := f.Limit
	if limit == 0 {
		limit = 50
	}
	if limit < 1 || limit > 500 {
		return page, fault.Invalid(fault.Violation{Field: "limit", Reason: "out of range"})
	}
	if len(page.Memories) > limit {
		b, _ := json.Marshal(page.Memories[limit-1])
		page.NextCursor = base64.RawURLEncoding.EncodeToString(b)
		page.Memories = page.Memories[:limit]
	}
	return page, nil
}
