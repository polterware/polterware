package mcp

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"time"

	mcpsdk "github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/schedule"
)

func (s *Server) createMemories(ctx context.Context, req *mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	var in struct {
		Items []createItem `json:"items"`
	}
	if err := decodeArgs(req, &in); err != nil {
		return fail(err), nil
	}
	if len(in.Items) == 0 {
		return fail(fmt.Errorf("create_memories needs at least 1 item")), nil
	}
	if len(in.Items) > MaxBatch {
		return fail(fmt.Errorf("create_memories accepts at most %d items", MaxBatch)), nil
	}
	sess := s.session(req)
	results := make([]map[string]any, 0, len(in.Items))
	okCount := 0
	for _, item := range in.Items {
		row, err := s.createOne(ctx, sess, item)
		if err != nil {
			results = append(results, itemError("", err))
			continue
		}
		okCount++
		results = append(results, row)
	}
	return ok(fmt.Sprintf("created %d", okCount), map[string]any{"results": results, "actor": sess.Actor().Raw}), nil
}

type createItem struct {
	Title     string            `json:"title"`
	Note      string            `json:"note"`
	Mind      string            `json:"mind"`
	Tags      []string          `json:"tags"`
	Due       string            `json:"due"`
	Pinned    bool              `json:"pinned"`
	Checklist []json.RawMessage `json:"checklist"`
	Schedule  json.RawMessage   `json:"schedule"`
	Place     json.RawMessage   `json:"place"`
	Links     []linkIn          `json:"links"`
}

type linkIn struct {
	URL   string `json:"url"`
	Title string `json:"title"`
}

func (s *Server) createOne(ctx context.Context, sess *MemorySession, item createItem) (map[string]any, error) {
	in := memory.Create{Title: item.Title, Note: item.Note, Pinned: item.Pinned}
	if item.Mind != "" {
		id, err := optionalMind(ctx, s.deps.Minds, item.Mind)
		if err != nil {
			return nil, err
		}
		in.MindID = id
	}
	if len(item.Tags) > 0 {
		if s.deps.Tags == nil {
			return nil, fault.Unavailable("tags backend is not wired")
		}
		ids, err := resolveTags(ctx, s.deps.Tags, item.Tags)
		if err != nil {
			return nil, err
		}
		in.TagIDs = ids
	}
	due, allDay, err := parseDue(item.Due)
	if err != nil {
		return nil, err
	}
	in.DueAt, in.DueAllDay = due, allDay
	if len(item.Checklist) > 0 {
		in.Checklist, err = parseChecklist(item.Checklist)
		if err != nil {
			return nil, err
		}
	}
	created, err := sess.Create(ctx, in)
	if err != nil {
		return nil, err
	}
	extra := map[string]any{"rev": created.Rev, "title": created.Title}
	if len(item.Schedule) > 0 && string(item.Schedule) != "null" {
		next, err := s.setSchedule(ctx, sess, created.ID, item.Schedule)
		if err != nil {
			return nil, err
		}
		if len(next) > 0 {
			extra["nextOccurrences"] = next
		}
	}
	if len(item.Place) > 0 && string(item.Place) != "null" {
		if err := s.setPlace(ctx, created.ID, item.Place); err != nil {
			return nil, err
		}
	}
	if err := s.addLinks(ctx, created.ID, item.Links); err != nil {
		return nil, err
	}
	return itemOK(created.ID, extra), nil
}

func (s *Server) updateMemories(ctx context.Context, req *mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	var in struct {
		Items []updateItem `json:"items"`
	}
	if err := decodeArgs(req, &in); err != nil {
		return fail(err), nil
	}
	if len(in.Items) == 0 {
		return fail(fmt.Errorf("update_memories needs at least 1 item")), nil
	}
	if len(in.Items) > MaxBatch {
		return fail(fmt.Errorf("update_memories accepts at most %d items", MaxBatch)), nil
	}
	sess := s.session(req)
	results := make([]map[string]any, 0, len(in.Items))
	okCount := 0
	for _, item := range in.Items {
		row, err := s.updateOne(ctx, sess, item)
		if err != nil {
			results = append(results, itemError(item.ID, err))
			continue
		}
		okCount++
		results = append(results, row)
	}
	return ok(fmt.Sprintf("updated %d", okCount), map[string]any{"results": results}), nil
}

type updateItem struct {
	ID          string            `json:"id"`
	ExpectedRev *int64            `json:"expectedRev"`
	Title       nullable[string]  `json:"title"`
	Note        nullable[string]  `json:"note"`
	Mind        nullable[string]  `json:"mind"`
	Tags        []string          `json:"tags"`
	AddTags     []string          `json:"addTags"`
	RemoveTags  []string          `json:"removeTags"`
	Due         nullable[string]  `json:"due"`
	Pinned      nullable[bool]    `json:"pinned"`
	Checklist   []json.RawMessage `json:"checklist"`
	Schedule    json.RawMessage   `json:"schedule"`
	Place       json.RawMessage   `json:"place"`
	Links       []linkIn          `json:"links"`
}

func (s *Server) updateOne(ctx context.Context, sess *MemorySession, item updateItem) (map[string]any, error) {
	if item.ID == "" {
		return nil, fmt.Errorf("id is required")
	}
	patch := memory.Patch{}
	if item.Title.Set && !item.Title.Null {
		patch.Title = &item.Title.Value
	}
	if item.Note.Set {
		if item.Note.Null {
			empty := ""
			patch.Note = &empty
		} else {
			patch.Note = &item.Note.Value
		}
	}
	if item.Mind.Set {
		if item.Mind.Null || item.Mind.Value == "" {
			patch.ClearMindID = true
		} else {
			id, err := optionalMind(ctx, s.deps.Minds, item.Mind.Value)
			if err != nil {
				return nil, err
			}
			patch.MindID = id
		}
	}
	if item.Due.Set {
		if item.Due.Null || item.Due.Value == "" {
			patch.ClearDueAt = true
		} else {
			due, allDay, err := parseDue(item.Due.Value)
			if err != nil {
				return nil, err
			}
			patch.DueAt, patch.DueAllDay = due, &allDay
		}
	}
	if item.Pinned.Set && !item.Pinned.Null {
		patch.Pinned = &item.Pinned.Value
	}
	updated, err := sess.Update(ctx, item.ID, item.ExpectedRev, patch)
	if err != nil {
		return nil, err
	}
	if len(item.Checklist) > 0 {
		items, err := parseChecklist(item.Checklist)
		if err != nil {
			return nil, err
		}
		if _, err := s.deps.Memories.ReplaceChecklist(ctx, sess.Actor(), item.ID, updated.Rev, items); err != nil {
			return nil, err
		}
		updated.Rev++
		sess.observe(updated.ID, updated.Rev)
	}
	if err := s.applyTagChanges(ctx, sess, item, updated.Rev); err != nil {
		return nil, err
	}
	extra := map[string]any{"rev": updated.Rev, "title": updated.Title}
	if len(item.Schedule) > 0 {
		if string(item.Schedule) == "null" {
			if s.deps.Schedules == nil {
				return nil, fault.Unavailable("schedule backend is not wired")
			}
			if err := s.deps.Schedules.Clear(ctx, sess.Actor(), item.ID); err != nil && !fault.IsCode(err, fault.CodeNotFound) {
				return nil, err
			}
		} else {
			next, err := s.setSchedule(ctx, sess, item.ID, item.Schedule)
			if err != nil {
				return nil, err
			}
			if len(next) > 0 {
				extra["nextOccurrences"] = next
			}
		}
	}
	if len(item.Place) > 0 {
		if string(item.Place) == "null" {
			if s.deps.Places != nil {
				_ = s.deps.Places.Clear(ctx, item.ID)
			}
		} else if err := s.setPlace(ctx, item.ID, item.Place); err != nil {
			return nil, err
		}
	}
	if err := s.addLinks(ctx, item.ID, item.Links); err != nil {
		return nil, err
	}
	return itemOK(item.ID, extra), nil
}

func (s *Server) applyTagChanges(ctx context.Context, sess *MemorySession, item updateItem, rev int64) error {
	if s.deps.Tags == nil || (len(item.Tags) == 0 && len(item.AddTags) == 0 && len(item.RemoveTags) == 0) {
		return nil
	}
	current, err := s.deps.Memories.Get(ctx, item.ID, memory.Include{Tags: true})
	if err != nil {
		return err
	}
	ids := append([]string{}, current.TagIDs...)
	if len(item.Tags) > 0 {
		ids, err = resolveTags(ctx, s.deps.Tags, item.Tags)
		if err != nil {
			return err
		}
	}
	if len(item.AddTags) > 0 {
		add, err := resolveTags(ctx, s.deps.Tags, item.AddTags)
		if err != nil {
			return err
		}
		ids = union(ids, add)
	}
	if len(item.RemoveTags) > 0 {
		remove, err := resolveTags(ctx, s.deps.Tags, item.RemoveTags)
		if err != nil {
			return err
		}
		ids = subtract(ids, remove)
	}
	return s.deps.Memories.SetTags(ctx, sess.Actor(), item.ID, rev, ids)
}

func (s *Server) setMemoryStatus(ctx context.Context, req *mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	var in struct {
		ID         string `json:"id"`
		Status     string `json:"status"`
		Occurrence string `json:"occurrence"`
	}
	if err := decodeArgs(req, &in); err != nil {
		return fail(err), nil
	}
	if in.ID == "" || in.Status == "" {
		return fail(fmt.Errorf("id and status are required")), nil
	}
	var occurrence *time.Time
	if in.Occurrence != "" {
		at, err := parseInstant(in.Occurrence, "occurrence")
		if err != nil {
			return fail(err), nil
		}
		occurrence = at
	}
	updated, err := s.session(req).SetStatus(ctx, in.ID, nil, in.Status, occurrence)
	if err != nil {
		return fail(err), nil
	}
	return ok(updated.Title+" is "+updated.Status, map[string]any{"id": updated.ID, "status": updated.Status, "rev": updated.Rev}), nil
}

func (s *Server) toggleCheckItem(ctx context.Context, req *mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	var in struct {
		MemoryID  string `json:"memoryId"`
		ItemID    string `json:"itemId"`
		ItemTitle string `json:"itemTitle"`
	}
	if err := decodeArgs(req, &in); err != nil {
		return fail(err), nil
	}
	if in.MemoryID == "" {
		return fail(fmt.Errorf("memoryId is required")), nil
	}
	sess := s.session(req)
	itemID := in.ItemID
	if itemID == "" {
		if in.ItemTitle == "" {
			return fail(fmt.Errorf("itemId or itemTitle is required")), nil
		}
		got, err := sess.Get(ctx, in.MemoryID, memory.Include{Checklist: true})
		if err != nil {
			return fail(err), nil
		}
		for _, item := range got.Checklist {
			if strings.EqualFold(item.Title, in.ItemTitle) {
				itemID = item.ID
				break
			}
		}
		if itemID == "" {
			return fail(fmt.Errorf("checklist item %q not found", in.ItemTitle)), nil
		}
	}
	// Request 3: ToggleCheckItem has no expectedRev. Do not pre-read to fake atomicity.
	item, err := s.deps.Memories.ToggleCheckItem(ctx, sess.Actor(), in.MemoryID, itemID, nil)
	if err != nil {
		return fail(err), nil
	}
	return ok(item.Title, map[string]any{"id": item.ID, "title": item.Title, "done": item.Done, "memoryId": in.MemoryID}), nil
}

func (s *Server) deleteMemories(ctx context.Context, req *mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	var in struct {
		IDs     []string `json:"ids"`
		Confirm bool     `json:"confirm"`
	}
	if err := decodeArgs(req, &in); err != nil {
		return fail(err), nil
	}
	if len(in.IDs) == 0 {
		return fail(fmt.Errorf("ids is required")), nil
	}
	if len(in.IDs) > MaxBatch {
		return fail(fmt.Errorf("delete_memories accepts at most %d ids", MaxBatch)), nil
	}
	titles := make([]string, 0, len(in.IDs))
	for _, id := range in.IDs {
		got, err := s.deps.Memories.Get(ctx, id, memory.Include{})
		if err != nil {
			titles = append(titles, id)
			continue
		}
		titles = append(titles, got.Memory.Title)
	}
	if !in.Confirm {
		return fail(fmt.Errorf("refusing to delete. Set confirm true. Would delete: %s", strings.Join(titles, ", "))), nil
	}
	sess := s.session(req)
	results := make([]map[string]any, 0, len(in.IDs))
	okCount := 0
	for _, id := range in.IDs {
		// Request 3: Delete has no expectedRev. Call the domain port as-is.
		if err := s.deps.Memories.Delete(ctx, sess.Actor(), id); err != nil {
			results = append(results, itemError(id, err))
			continue
		}
		okCount++
		results = append(results, itemOK(id, nil))
	}
	return ok(fmt.Sprintf("deleted %d", okCount), map[string]any{"results": results}), nil
}

func (s *Server) createMind(ctx context.Context, req *mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	var in struct {
		Name      string `json:"name"`
		Parent    string `json:"parent"`
		Color     string `json:"color"`
		Icon      string `json:"icon"`
		SortOrder int    `json:"sortOrder"`
	}
	if err := decodeArgs(req, &in); err != nil {
		return fail(err), nil
	}
	create := mind.Create{Name: in.Name, SortOrder: in.SortOrder}
	if in.Parent != "" {
		id, err := optionalMind(ctx, s.deps.Minds, in.Parent)
		if err != nil {
			return fail(err), nil
		}
		create.ParentID = id
	}
	if in.Color != "" {
		create.Color = &in.Color
	}
	if in.Icon != "" {
		create.Icon = &in.Icon
	}
	created, err := s.deps.Minds.Create(ctx, s.session(req).Actor(), create)
	if err != nil {
		return fail(err), nil
	}
	s.session(req).observeMind(created.ID, created.Rev)
	return ok(created.Name, mindJSON(created)), nil
}

func (s *Server) updateMind(ctx context.Context, req *mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	var in struct {
		ID          string           `json:"id"`
		Name        string           `json:"name"`
		NewName     string           `json:"newName"`
		Parent      nullable[string] `json:"parent"`
		Color       nullable[string] `json:"color"`
		Icon        nullable[string] `json:"icon"`
		SortOrder   *int             `json:"sortOrder"`
		ExpectedRev *int64           `json:"expectedRev"`
	}
	if err := decodeArgs(req, &in); err != nil {
		return fail(err), nil
	}
	ref := in.ID
	if ref == "" {
		ref = in.Name
	}
	if ref == "" {
		return fail(fmt.Errorf("id or name is required")), nil
	}
	got, err := resolveMind(ctx, s.deps.Minds, ref)
	if err != nil {
		return fail(err), nil
	}
	patch := mind.Patch{SortOrder: in.SortOrder}
	if in.NewName != "" {
		patch.Name = &in.NewName
	} else if in.Name != "" && in.ID != "" {
		patch.Name = &in.Name
	}
	if in.Parent.Set {
		if in.Parent.Null || in.Parent.Value == "" {
			patch.ClearParent = true
		} else {
			id, err := optionalMind(ctx, s.deps.Minds, in.Parent.Value)
			if err != nil {
				return fail(err), nil
			}
			patch.ParentID = id
		}
	}
	if in.Color.Set {
		if in.Color.Null {
			patch.ClearColor = true
		} else {
			patch.Color = &in.Color.Value
		}
	}
	if in.Icon.Set {
		if in.Icon.Null {
			patch.ClearIcon = true
		} else {
			patch.Icon = &in.Icon.Value
		}
	}
	sess := s.session(req)
	updated, err := s.deps.Minds.Update(ctx, sess.Actor(), got.ID, sess.expectedMind(got.ID, in.ExpectedRev), patch)
	if err != nil {
		return fail(err), nil
	}
	sess.observeMind(updated.ID, updated.Rev)
	return ok(updated.Name, mindJSON(updated)), nil
}

func (s *Server) deleteMind(ctx context.Context, req *mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	var in struct {
		ID      string `json:"id"`
		Name    string `json:"name"`
		Confirm bool   `json:"confirm"`
	}
	if err := decodeArgs(req, &in); err != nil {
		return fail(err), nil
	}
	ref := in.ID
	if ref == "" {
		ref = in.Name
	}
	if ref == "" {
		return fail(fmt.Errorf("id or name is required")), nil
	}
	got, err := resolveMind(ctx, s.deps.Minds, ref)
	if err != nil {
		return fail(err), nil
	}
	if !in.Confirm {
		return fail(fmt.Errorf("refusing to delete mind %q. Children are deleted and memories move to Inbox. Set confirm true", got.Name)), nil
	}
	// Request 3: mind.Delete has no expectedRev.
	result, err := s.deps.Minds.Delete(ctx, s.session(req).Actor(), got.ID)
	if err != nil {
		return fail(err), nil
	}
	return ok("deleted "+got.Name, map[string]any{"deletedMindIds": result.DeletedMindIDs, "movedMemoryCount": result.MovedMemoryCount}), nil
}

func (s *Server) addAttachments(ctx context.Context, req *mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	var in struct {
		MemoryID string   `json:"memoryId"`
		Links    []linkIn `json:"links"`
		Files    []struct {
			Path string `json:"path"`
		} `json:"files"`
	}
	if err := decodeArgs(req, &in); err != nil {
		return fail(err), nil
	}
	if in.MemoryID == "" {
		return fail(fmt.Errorf("memoryId is required")), nil
	}
	if _, err := s.deps.Memories.Get(ctx, in.MemoryID, memory.Include{}); err != nil {
		return fail(err), nil
	}
	if len(in.Files) > 0 {
		allowed, err := s.fileAttachAllowed(ctx)
		if err != nil {
			return fail(err), nil
		}
		if !allowed {
			return fail(fmt.Errorf("file attachments are off. Enable them in Settings > Agents > Allow file attach")), nil
		}
		for _, file := range in.Files {
			if err := checkAttachPath(file.Path); err != nil {
				return fail(err), nil
			}
		}
	}
	if s.deps.Attachments == nil {
		return fail(fault.Unavailable("attachments backend is not wired")), nil
	}
	var out []map[string]any
	for _, link := range in.Links {
		a, err := s.deps.Attachments.AddLink(ctx, in.MemoryID, link.URL, link.Title)
		if err != nil {
			return fail(err), nil
		}
		row := map[string]any{"id": a.ID, "kind": a.Kind}
		if a.URL != nil {
			row["url"] = *a.URL
		}
		out = append(out, row)
	}
	for _, file := range in.Files {
		a, err := s.deps.Attachments.ImportFromPath(ctx, in.MemoryID, file.Path, "")
		if err != nil {
			return fail(err), nil
		}
		out = append(out, map[string]any{"id": a.ID, "kind": a.Kind})
	}
	return ok(fmt.Sprintf("added %d", len(out)), map[string]any{"attachments": out}), nil
}

func (s *Server) setSchedule(ctx context.Context, sess *MemorySession, memoryID string, raw json.RawMessage) ([]string, error) {
	if s.deps.Schedules == nil {
		return nil, fault.Unavailable("schedule backend is not wired")
	}
	row, rule, err := parseSchedule(raw, memoryID)
	if err != nil {
		return nil, err
	}
	if _, err := s.deps.Schedules.Set(ctx, sess.Actor(), row); err != nil {
		return nil, err
	}
	times, _, err := s.deps.Schedules.Preview(ctx, row, 3)
	if err != nil {
		times, _, err = schedule.Preview(rule, 3)
		if err != nil {
			return nil, err
		}
	}
	return nextOccurrences(times), nil
}

func (s *Server) setPlace(ctx context.Context, memoryID string, raw json.RawMessage) error {
	if s.deps.Places == nil {
		return fault.Unavailable("place backend is not wired")
	}
	p, err := parsePlace(raw, memoryID)
	if err != nil {
		return err
	}
	_, err = s.deps.Places.Set(ctx, p)
	return err
}

func (s *Server) addLinks(ctx context.Context, memoryID string, links []linkIn) error {
	if len(links) == 0 {
		return nil
	}
	if s.deps.Attachments == nil {
		return fault.Unavailable("attachments backend is not wired")
	}
	for _, link := range links {
		if _, err := s.deps.Attachments.AddLink(ctx, memoryID, link.URL, link.Title); err != nil {
			return err
		}
	}
	return nil
}

func union(a, b []string) []string {
	seen := map[string]bool{}
	out := []string{}
	for _, id := range append(append([]string{}, a...), b...) {
		if !seen[id] {
			seen[id] = true
			out = append(out, id)
		}
	}
	return out
}

func subtract(ids, remove []string) []string {
	drop := map[string]bool{}
	for _, id := range remove {
		drop[id] = true
	}
	out := []string{}
	for _, id := range ids {
		if !drop[id] {
			out = append(out, id)
		}
	}
	return out
}
