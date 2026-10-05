package mcp_test

import (
	"context"
	"fmt"
	"strings"
	"testing"
	"time"

	sdk "github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/attachments"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/place"
	"github.com/polterware/polterware/sparky/backend/domains/schedule"
	"github.com/polterware/polterware/sparky/backend/domains/search"
	provider "github.com/polterware/polterware/sparky/backend/providers/mcp"
)

func resultRow(t *testing.T, h *harness, tool string, item map[string]any) map[string]any {
	t.Helper()
	return structured(t, h.call(t, tool, map[string]any{"items": []map[string]any{item}}))["results"].([]any)[0].(map[string]any)
}
func createID(t *testing.T, h *harness, item map[string]any) string {
	t.Helper()
	row := resultRow(t, h, "create_memories", item)
	if row["ok"] != true {
		t.Fatalf("create: %#v", row)
	}
	return row["id"].(string)
}

func TestSearchAndResourcesTrackReturnedRevision(t *testing.T) {
	for _, read := range []string{"search", "fallback search", "memory resource"} {
		t.Run(read, func(t *testing.T) {
			h := setupMCP(t, func(d *provider.Deps) {
				if read == "fallback search" {
					d.Search = nil
				}
			})
			// Create through UI so the MCP session has not observed the memory.
			m, err := h.fix.MS.Create(t.Context(), actor.UI(), memory.Create{Title: "Snapshot"})
			if err != nil {
				t.Fatal(err)
			}
			if read == "memory resource" {
				got, err := h.sess.ReadResource(t.Context(), &sdk.ReadResourceParams{URI: "sparky://memory/" + m.ID})
				if err != nil || got.Contents[0].Text != "Snapshot" {
					t.Fatalf("resource: %v / %v", got, err)
				}
			} else {
				got := structured(t, h.call(t, "search_memories", map[string]any{"query": "Snapshot"}))["hits"].([]any)[0].(map[string]any)
				if got["rev"] != float64(1) {
					t.Fatalf("snapshot: %#v", got)
				}
			}
			title := "UI changed"
			if _, err := h.fix.MS.Update(t.Context(), actor.UI(), m.ID, 1, memory.Patch{Title: &title}); err != nil {
				t.Fatal(err)
			}
			row := resultRow(t, h, "update_memories", map[string]any{"id": m.ID, "title": "Agent overwrite"})
			if row["conflict"] != true {
				t.Fatalf("missing conflict: %#v", row)
			}
		})
	}
}

func TestMindResourceTracksReturnedRevision(t *testing.T) {
	h := setupMCP(t)
	m, err := h.fix.NS.Create(t.Context(), actor.UI(), mind.Create{Name: "Work"})
	if err != nil {
		t.Fatal(err)
	}
	if _, err := h.sess.ReadResource(t.Context(), &sdk.ReadResourceParams{URI: "sparky://minds"}); err != nil {
		t.Fatal(err)
	}
	name := "UI renamed"
	if _, err := h.fix.NS.Update(t.Context(), actor.UI(), m.ID, 1, mind.Patch{Name: &name}); err != nil {
		t.Fatal(err)
	}
	res := h.call(t, "update_mind", map[string]any{"id": m.ID, "newName": "Agent"})
	if !res.IsError || structured(t, res)["conflict"] != true {
		t.Fatalf("mind resource did not protect revision: %s", textOf(res))
	}
}

func TestEmptyArraysReplaceAndTagsReturnFinalRevision(t *testing.T) {
	h := setupMCP(t)
	if _, err := h.fix.TS.Create(t.Context(), actor.UI(), "work", nil); err != nil {
		t.Fatal(err)
	}
	id := createID(t, h, map[string]any{"title": "Task", "tags": []string{"work"}, "checklist": []string{"A"}})
	row := resultRow(t, h, "update_memories", map[string]any{"id": id, "note": "Keep relations"})
	if row["ok"] != true {
		t.Fatal(row)
	}
	got, err := h.fix.MS.Get(t.Context(), id, memory.Include{Checklist: true, Tags: true})
	if err != nil || len(got.Checklist) != 1 || len(got.TagIDs) != 1 {
		t.Fatalf("omitted arrays changed relations: %#v / %v", got, err)
	}
	row = resultRow(t, h, "update_memories", map[string]any{"id": id, "checklist": []string{}, "tags": []string{}})
	got, err = h.fix.MS.Get(t.Context(), id, memory.Include{Checklist: true, Tags: true})
	if err != nil || len(got.Checklist) != 0 || len(got.TagIDs) != 0 || row["rev"] != float64(got.Memory.Rev) {
		t.Fatalf("empty arrays/final rev: %#v / %#v / %v", row, got, err)
	}
	row = resultRow(t, h, "update_memories", map[string]any{"id": id, "title": "Follow-up"})
	if row["ok"] != true {
		t.Fatalf("session retained pre-tag rev: %#v", row)
	}
}

func TestTagsNeverObservePostCommitUIRevision(t *testing.T) {
	var afterCommit func()
	h := setupMCP(t, func(d *provider.Deps) {
		within := d.Within
		d.Within = func(ctx context.Context, fn func(context.Context) error) error {
			err := within(ctx, fn)
			if err == nil && afterCommit != nil {
				afterCommit()
			}
			return err
		}
	})
	id := createID(t, h, map[string]any{"title": "Task"})
	afterCommit = func() {
		title := "Later UI edit"
		if _, err := h.fix.MS.Update(t.Context(), actor.UI(), id, 3, memory.Patch{Title: &title}); err != nil {
			t.Fatal(err)
		}
	}
	row := resultRow(t, h, "update_memories", map[string]any{"id": id, "tags": []string{}})
	afterCommit = nil
	if row["rev"] != float64(3) {
		t.Fatalf("wrong tag revision: %#v", row)
	}
	row = resultRow(t, h, "update_memories", map[string]any{"id": id, "title": "Agent"})
	if row["conflict"] != true {
		t.Fatalf("later UI edit not protected: %#v", row)
	}
}

func TestExplicitNonpositiveRevisionsCannotBypassSession(t *testing.T) {
	h := setupMCP(t)
	id := createID(t, h, map[string]any{"title": "Task", "checklist": []string{"A"}})
	title := "UI edit"
	if _, err := h.fix.MS.Update(t.Context(), actor.UI(), id, 1, memory.Patch{Title: &title}); err != nil {
		t.Fatal(err)
	}
	for _, rev := range []int64{0, -1} {
		row := resultRow(t, h, "update_memories", map[string]any{"id": id, "title": "Overwrite", "expectedRev": rev})
		if row["ok"] != false {
			t.Fatalf("revision %d bypassed session: %#v", rev, row)
		}
		for _, tc := range []struct {
			tool string
			args map[string]any
		}{
			{"set_memory_status", map[string]any{"id": id, "status": "completed", "expectedRev": rev}},
			{"toggle_check_item", map[string]any{"memoryId": id, "itemTitle": "A", "expectedRev": rev}},
			{"delete_memories", map[string]any{"ids": []string{id}, "confirm": true, "expectedRev": map[string]int64{id: rev}}},
			{"add_attachments", map[string]any{"memoryId": id, "links": []map[string]string{{"url": "https://example.com"}}, "expectedRev": rev}},
		} {
			if res := h.call(t, tc.tool, tc.args); !res.IsError {
				t.Fatalf("%s accepted rev %d", tc.tool, rev)
			}
		}
	}
	got, err := h.fix.MS.Get(t.Context(), id, memory.Include{})
	if err != nil || got.Memory.Rev != 2 || got.Memory.Title != title {
		t.Fatalf("invalid revision wrote: %#v / %v", got, err)
	}
}

type failSchedule struct{ scheduleStub }

func (failSchedule) Set(context.Context, actor.Actor, schedule.Schedule) (schedule.Schedule, error) {
	return schedule.Schedule{}, fmt.Errorf("injected schedule failure")
}

type failPlace struct{}

func (failPlace) Set(context.Context, actor.Actor, place.Place) (place.Place, error) {
	return place.Place{}, fmt.Errorf("injected place failure")
}
func (failPlace) Clear(context.Context, actor.Actor, string) error { return nil }

type failLink struct{ attachments.Service }

func (failLink) AddLink(context.Context, string, string, string, int64) (attachments.Attachment, error) {
	return attachments.Attachment{}, fmt.Errorf("injected link failure")
}

func TestCreateValidatesBeforePersistenceAndRollsBackEachItem(t *testing.T) {
	for _, stage := range []string{"invalid schedule", "invalid place", "invalid link", "schedule failure", "place failure", "link failure"} {
		t.Run(stage, func(t *testing.T) {
			h := setupMCP(t, func(d *provider.Deps) {
				switch stage {
				case "schedule failure":
					d.Schedules = failSchedule{}
				case "place failure":
					d.Places = failPlace{}
				case "link failure":
					d.Attachments = failLink{}
				}
			})
			item := map[string]any{"title": "Failed item"}
			switch stage {
			case "invalid schedule":
				item["schedule"] = map[string]any{"at": "invalid"}
			case "invalid place":
				item["place"] = map[string]any{"lat": 100, "lng": 0}
			case "invalid link":
				item["links"] = []map[string]any{{"url": "file:///secret"}}
			case "schedule failure":
				item["schedule"] = map[string]any{"at": "2026-10-05T12:00:00Z"}
			case "place failure":
				item["place"] = map[string]any{"lat": 0, "lng": 0}
			case "link failure":
				item["links"] = []map[string]any{{"url": "https://example.com"}}
			}
			// Repeat failures to prove retries do not create hidden duplicates.
			for i := 0; i < 2; i++ {
				if row := resultRow(t, h, "create_memories", item); row["ok"] != false {
					t.Fatalf("expected failure: %#v", row)
				}
			}
			page, err := h.fix.MS.List(t.Context(), memory.ListFilter{})
			if err != nil || page.Total != 0 {
				t.Fatalf("failed item persisted: %#v / %v", page, err)
			}
			events, err := h.fix.Audit.Repository.List(t.Context(), "", "", 100)
			if err != nil || len(events.Events) != 0 {
				t.Fatalf("rollback left audit: %#v / %v", events, err)
			}
			good := createID(t, h, map[string]any{"title": "Independent item"})
			if good == "" {
				t.Fatal("good item did not commit")
			}
		})
	}
}

type todaySchedule struct {
	scheduleStub
	id *string
}

func (s todaySchedule) ListOccurrences(context.Context, time.Time, time.Time, *string, bool, int) ([]schedule.Occurrence, error) {
	return []schedule.Occurrence{{MemoryID: *s.id, Title: "Old occurrence title"}}, nil
}
func TestTodayResourceTracksReturnedMemorySnapshot(t *testing.T) {
	var id string
	h := setupMCP(t, func(d *provider.Deps) { d.Schedules = todaySchedule{id: &id} })
	m, err := h.fix.MS.Create(t.Context(), actor.UI(), memory.Create{Title: "Today task"})
	if err != nil {
		t.Fatal(err)
	}
	id = m.ID
	res, err := h.sess.ReadResource(t.Context(), &sdk.ReadResourceParams{URI: "sparky://today"})
	if err != nil || !strings.Contains(res.Contents[0].Text, "Today task") {
		t.Fatalf("today snapshot: %v / %v", res, err)
	}
	title := "UI changed"
	if _, err := h.fix.MS.Update(t.Context(), actor.UI(), id, 1, memory.Patch{Title: &title}); err != nil {
		t.Fatal(err)
	}
	row := resultRow(t, h, "update_memories", map[string]any{"id": id, "title": "Overwrite"})
	if row["conflict"] != true {
		t.Fatalf("today resource did not protect revision: %#v", row)
	}
}

type searchThenWrite struct {
	search.Service
	memories memory.Service
}

func (s searchThenWrite) Search(ctx context.Context, q string, f search.Filter) (search.Result, error) {
	result, err := s.Service.Search(ctx, q, f)
	if err != nil {
		return result, err
	}
	title, note := "New snapshot title", "New snapshot note"
	_, err = s.memories.Update(ctx, actor.UI(), result.Hits[0].MemoryID, 1, memory.Patch{Title: &title, Note: &note})
	return result, err
}
func TestSearchRegistersRevisionOfHydratedSnapshot(t *testing.T) {
	h := setupMCP(t, func(d *provider.Deps) { d.Within = nil; d.Search = searchThenWrite{d.Search, d.Memories} })
	if _, err := h.fix.MS.Create(t.Context(), actor.UI(), memory.Create{Title: "Original"}); err != nil {
		t.Fatal(err)
	}
	hits := structured(t, h.call(t, "search_memories", map[string]any{"query": "Original"}))["hits"].([]any)
	row := hits[0].(map[string]any)
	if row["title"] != "New snapshot title" || row["snippet"] != "New snapshot note" || row["rev"] != float64(2) {
		t.Fatalf("mixed search snapshots: %#v", row)
	}
}

func TestToggleToolReturnsAndRegistersRevision(t *testing.T) {
	h := setupMCP(t)
	id := createID(t, h, map[string]any{"title": "Task", "checklist": []string{"A"}})
	res := h.call(t, "toggle_check_item", map[string]any{"memoryId": id, "itemTitle": "A"})
	if res.IsError || structured(t, res)["rev"] != float64(2) {
		t.Fatalf("toggle revision: %s / %#v", textOf(res), structured(t, res))
	}
	row := resultRow(t, h, "update_memories", map[string]any{"id": id, "note": "Follow-up"})
	if row["ok"] != true {
		t.Fatal(row)
	}
}

func TestToggleTitleLookupDoesNotAdvancePreviouslyReadRevision(t *testing.T) {
	h := setupMCP(t)
	id := createID(t, h, map[string]any{"title": "Task", "checklist": []string{"A"}})
	title := "UI changed"
	if _, err := h.fix.MS.Update(t.Context(), actor.UI(), id, 1, memory.Patch{Title: &title}); err != nil {
		t.Fatal(err)
	}
	res := h.call(t, "toggle_check_item", map[string]any{"memoryId": id, "itemTitle": "A"})
	if !res.IsError || structured(t, res)["conflict"] != true {
		t.Fatalf("internal title lookup bypassed prior read: %s", textOf(res))
	}
}

type revisionSchedule struct {
	provider.Schedules
	memories memory.Service
}

func (s revisionSchedule) Set(ctx context.Context, who actor.Actor, in schedule.Schedule) (schedule.Schedule, error) {
	out, err := s.Schedules.Set(ctx, who, in)
	if err == nil {
		_, err = s.memories.Update(ctx, who, in.MemoryID, 0, memory.Patch{})
	}
	return out, err
}

type revisionPlace struct {
	provider.Places
	memories memory.Service
}

func (p revisionPlace) Set(ctx context.Context, who actor.Actor, in place.Place) (place.Place, error) {
	out, err := p.Places.Set(ctx, who, in)
	if err == nil {
		_, err = p.memories.Update(ctx, who, in.MemoryID, 0, memory.Patch{})
	}
	return out, err
}

func TestCompoundWritesNeverObservePostCommitUIRevision(t *testing.T) {
	for _, tc := range []struct {
		name  string
		patch map[string]any
		rev   int64
	}{
		{"schedule", map[string]any{"schedule": map[string]any{"at": "2026-10-05T12:00:00Z"}}, 3},
		{"place", map[string]any{"place": map[string]any{"lat": 0, "lng": 0}}, 3},
		{"checklist", map[string]any{"checklist": []string{"Check"}}, 3},
		{"links", map[string]any{"links": []any{map[string]any{"url": "https://example.com"}}}, 2},
	} {
		t.Run(tc.name, func(t *testing.T) {
			var afterCommit func()
			h := setupMCP(t, func(d *provider.Deps) {
				d.Schedules = revisionSchedule{d.Schedules, d.Memories}
				d.Places = revisionPlace{d.Places, d.Memories}
				within := d.Within
				d.Within = func(ctx context.Context, fn func(context.Context) error) error {
					err := within(ctx, fn)
					if err == nil && afterCommit != nil {
						afterCommit()
					}
					return err
				}
			})
			id := createID(t, h, map[string]any{"title": "Task"})
			afterCommit = func() {
				title := "UI edit after commit"
				if _, err := h.fix.MS.Update(t.Context(), actor.UI(), id, tc.rev, memory.Patch{Title: &title}); err != nil {
					t.Fatal(err)
				}
			}
			tc.patch["id"] = id
			row := resultRow(t, h, "update_memories", tc.patch)
			afterCommit = nil
			if row["ok"] != true || row["rev"] != float64(tc.rev) {
				t.Fatalf("observed later revision: %v", row)
			}
			if row := resultRow(t, h, "update_memories", map[string]any{"id": id, "title": "Overwrite"}); row["conflict"] != true {
				t.Fatal(row)
			}
		})
	}
}

type attachmentWriteThenUI struct {
	attachments.Service
	memories memory.Service
}

func (s attachmentWriteThenUI) AddLink(ctx context.Context, id, url, title string, rev int64) (attachments.Attachment, error) {
	out, err := s.Service.AddLink(ctx, id, url, title, rev)
	if err == nil {
		title := "Later UI edit"
		_, err = s.memories.Update(ctx, actor.UI(), id, rev, memory.Patch{Title: &title})
	}
	return out, err
}
func TestFirstAttachmentWriteNeverObservesPostCommitUIRevision(t *testing.T) {
	h := setupMCP(t, func(d *provider.Deps) { d.Attachments = attachmentWriteThenUI{d.Attachments, d.Memories} })
	m, err := h.fix.MS.Create(t.Context(), actor.UI(), memory.Create{Title: "Unread"})
	if err != nil {
		t.Fatal(err)
	}
	row := structured(t, h.call(t, "add_attachments", map[string]any{"memoryId": m.ID, "links": []any{map[string]any{"url": "https://example.com"}}}))
	if row["rev"] != float64(1) {
		t.Fatal(row)
	}
	if row := resultRow(t, h, "update_memories", map[string]any{"id": m.ID, "title": "Overwrite"}); row["conflict"] != true {
		t.Fatal(row)
	}
}
