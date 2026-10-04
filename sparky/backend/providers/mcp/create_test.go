package mcp

import (
	"context"
	"encoding/json"
	"fmt"
	"testing"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/activity/fake"
	"github.com/polterware/polterware/sparky/backend/domains/activity/fake/testkit"
	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/attachments"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/place"
	"github.com/polterware/polterware/sparky/backend/domains/schedule"
)

type transactionalSchedule struct {
	Schedules
	store *fake.Store
}

func (s transactionalSchedule) Set(ctx context.Context, _ actor.Actor, in schedule.Schedule) (schedule.Schedule, error) {
	return in, s.store.Put(ctx, "test_schedules", in.MemoryID, in)
}
func (transactionalSchedule) Preview(context.Context, schedule.Schedule, int) ([]time.Time, string, error) {
	return []time.Time{time.Date(2026, 10, 5, 12, 0, 0, 0, time.UTC)}, "", nil
}

type transactionalPlace struct{ store *fake.Store }

func (p transactionalPlace) Set(ctx context.Context, _ actor.Actor, in place.Place) (place.Place, error) {
	return in, p.store.Put(ctx, "test_places", in.MemoryID, in)
}
func (p transactionalPlace) Clear(ctx context.Context, _ actor.Actor, id string) error {
	return p.store.Remove(ctx, "test_places", id)
}

type transactionalLinks struct {
	attachments.Service
	store *fake.Store
}

func (a transactionalLinks) AddLink(ctx context.Context, id, url, title string, _ int64) (attachments.Attachment, error) {
	if title == "fail" {
		return attachments.Attachment{}, fmt.Errorf("injected second-link failure")
	}
	item := attachments.Attachment{ID: id, MemoryID: id, Kind: attachments.KindLink, URL: &url}
	return item, a.store.Put(ctx, "test_links", id, item)
}

func TestCreateRollsBackMemorySchedulePlaceAndEarlierLinks(t *testing.T) {
	f := testkit.Setup()
	server, err := New(Deps{Memories: f.MS, Minds: f.NS, Within: f.Store.Within, Schedules: transactionalSchedule{store: f.Store}, Places: transactionalPlace{f.Store}, Attachments: transactionalLinks{store: f.Store}})
	if err != nil {
		t.Fatal(err)
	}
	sess := NewMemorySession(f.MS, "atomic-create")
	failed := createItem{Title: "Failed item", Schedule: json.RawMessage(`{"at":"2026-10-05T12:00:00Z"}`), Place: json.RawMessage(`{"lat":0,"lng":0}`), Links: []linkIn{{URL: "https://example.com", Title: "first"}, {URL: "https://example.com", Title: "fail"}}}
	if _, err := server.createOne(t.Context(), sess, failed); err == nil {
		t.Fatal("expected injected failure")
	}
	page, err := f.MS.List(t.Context(), memory.ListFilter{})
	if err != nil || page.Total != 0 {
		t.Fatalf("memory persisted: %#v / %v", page, err)
	}
	for _, table := range []string{"test_schedules", "test_places", "test_links"} {
		rows, err := fake.All[json.RawMessage](t.Context(), f.Store, table)
		if err != nil || len(rows) != 0 {
			t.Fatalf("%s persisted: %d / %v", table, len(rows), err)
		}
	}
	if len(sess.reads) != 0 {
		t.Fatal("rolled-back revisions were published")
	}
	// A sibling item commits independently; its final revision is registered.
	failed.Title = "Successful sibling"
	failed.Links = failed.Links[:1]
	row, err := server.createOne(t.Context(), sess, failed)
	if err != nil {
		t.Fatal(err)
	}
	id := row["id"].(string)
	if sess.expected(id, nil) != row["rev"].(int64) {
		t.Fatal("committed revision not registered")
	}
}
