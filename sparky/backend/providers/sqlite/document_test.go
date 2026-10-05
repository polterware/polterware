package sqlite

import (
	"context"
	"path/filepath"
	"testing"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/tag"
	"github.com/polterware/polterware/sparky/backend/platform/changes"
)

func TestDocumentSourceAndIndex(t *testing.T) {
	idx := &recIndex{}
	s, err := OpenWith(filepath.Join(t.TempDir(), "sparky.db"), changes.New(), idx)
	if err != nil {
		t.Fatal(err)
	}
	defer s.Close()
	ctx := context.Background()
	now := time.Date(2026, 10, 3, 21, 30, 0, 0, time.UTC)
	parent := "parent"
	if err := s.Minds().Insert(ctx, mind.Mind{ID: parent, Name: "Work", CreatedAt: now, UpdatedAt: now, Rev: 1}); err != nil {
		t.Fatal(err)
	}
	if err := s.Minds().Insert(ctx, mind.Mind{ID: "child", ParentID: &parent, Name: "Project", CreatedAt: now, UpdatedAt: now, Rev: 1}); err != nil {
		t.Fatal(err)
	}
	child := "child"
	if err := s.Memories().Insert(ctx, memory.Memory{
		ID: "m1", MindID: &child, Title: "café", Note: "# Hello\n**note**", Status: memory.StatusActive, CreatedAt: now, UpdatedAt: now, Rev: 1,
	}); err != nil {
		t.Fatal(err)
	}
	if err := s.Tags().Insert(ctx, tag.Tag{ID: "t1", Name: "urgent", CreatedAt: now, UpdatedAt: now, Rev: 1}); err != nil {
		t.Fatal(err)
	}
	if err := s.Memories().SetTagIDs(ctx, "m1", []string{"t1"}); err != nil {
		t.Fatal(err)
	}
	if err := s.Memories().ReplaceCheckItems(ctx, "m1", []memory.CheckItem{{
		ID: "c1", MemoryID: "m1", Title: "buy milk", Detail: "2L", CreatedAt: now, UpdatedAt: now,
	}}); err != nil {
		t.Fatal(err)
	}
	doc, err := s.Documents().Document(ctx, "m1")
	if err != nil {
		t.Fatal(err)
	}
	if doc.Title != "café" || doc.Mind != "Work / Project" || doc.Tags != "urgent" {
		t.Fatalf("doc: %+v", doc)
	}
	if doc.Checklist != "buy milk\n2L" {
		t.Fatalf("checklist %q", doc.Checklist)
	}
	if doc.Note != "Hello\nnote" {
		t.Fatalf("note %q", doc.Note)
	}
	last := idx.last()
	if last.MemoryID != "m1" || last.Mind != "Work / Project" {
		t.Fatalf("index last %+v", last)
	}
	if err := s.Memories().Delete(ctx, 0, "m1"); err != nil {
		t.Fatal(err)
	}
	if len(idx.deleted) == 0 || idx.deleted[len(idx.deleted)-1] != "m1" {
		t.Fatalf("delete index: %v", idx.deleted)
	}
}
