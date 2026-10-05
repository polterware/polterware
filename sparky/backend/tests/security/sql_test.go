package security

import (
	"path/filepath"
	"testing"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/tag"
	"github.com/polterware/polterware/sparky/backend/providers/sqlite"
)

func TestHostileSQLValuesCannotChangeQueryStructure(t *testing.T) {
	store, err := sqlite.Open(filepath.Join(t.TempDir(), "sparky.db"))
	if err != nil {
		t.Fatal(err)
	}
	defer store.Close()
	now := time.Now().UTC()
	payload := "x'); DROP TABLE memories; --"
	ctx := t.Context()
	if err := store.Minds().Insert(ctx, mind.Mind{ID: "mind", Name: payload, CreatedAt: now, UpdatedAt: now, Rev: 1}); err != nil {
		t.Fatal(err)
	}
	if err := store.Tags().Insert(ctx, tag.Tag{ID: "tag", Name: payload, CreatedAt: now, UpdatedAt: now, Rev: 1}); err != nil {
		t.Fatal(err)
	}
	if err := store.Memories().Insert(ctx, memory.Memory{ID: "memory", MindID: ptr("mind"), Title: payload, Note: payload, Status: memory.StatusActive, CreatedAt: now, UpdatedAt: now, Rev: 1}); err != nil {
		t.Fatal(err)
	}
	got, err := store.Memories().Get(ctx, "memory")
	if err != nil || got.Title != payload || got.Note != payload {
		t.Fatalf("hostile text was not literal: %+v %v", got, err)
	}
	for _, filter := range []memory.ListFilter{{MindID: ptr("' OR 1=1 --")}, {Status: "' OR 1=1 --"}, {TagIDs: []string{"') OR 1=1 --"}}} {
		page, err := store.Memories().List(ctx, filter)
		if err != nil || len(page.Memories) != 0 {
			t.Fatalf("filter escaped parameters: %+v %v", page, err)
		}
	}
	for _, id := range []string{"' OR 1=1 --", "memory'; DELETE FROM memories; --"} {
		if _, err := store.Memories().Get(ctx, id); err == nil {
			t.Fatalf("hostile lookup returned a row: %s", id)
		}
	}
	if _, err := store.Tags().GetByName(ctx, payload); err != nil {
		t.Fatal(err)
	}
	if _, err := store.Activity().List(ctx, "", "' OR 1=1 --", 10); err != nil {
		t.Fatal(err)
	}
	// FTS grammar may reject hostile syntax; it must never change SQL structure.
	_, _ = store.Memories().List(ctx, memory.ListFilter{Query: "x' OR 1=1 --"})
	if _, err := store.Memories().Get(ctx, "memory"); err != nil {
		t.Fatalf("table was altered: %v", err)
	}
}
func ptr(value string) *string { return &value }
