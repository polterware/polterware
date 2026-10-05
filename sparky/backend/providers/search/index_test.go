package search

import (
	"context"
	"errors"
	"fmt"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	searchdomain "github.com/polterware/polterware/sparky/backend/domains/search"
	"github.com/polterware/polterware/sparky/backend/domains/tag"
	"github.com/polterware/polterware/sparky/backend/platform/changes"
	"github.com/polterware/polterware/sparky/backend/providers/sqlite"
)

func openHarness(t *testing.T) (*sqlite.Store, *Index, searchdomain.Service) {
	t.Helper()
	store, err := sqlite.OpenWith(filepath.Join(t.TempDir(), "sparky.db"), changes.New(), nil)
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = store.Close() })
	idx := NewWithReader(store.Handle().Writer(), store.Handle().Q(context.Background(), false))
	store.SetIndex(idx)
	source := listDocuments{store: store}
	return store, idx, searchdomain.NewService(idx, source)
}

type listDocuments struct{ store *sqlite.Store }

func (l listDocuments) ListSearchDocuments(ctx context.Context) ([]searchdomain.Document, error) {
	rows, err := l.store.Handle().Q(ctx, false).QueryContext(ctx, `SELECT id FROM memories ORDER BY id`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var docs []searchdomain.Document
	for rows.Next() {
		var id string
		if err := rows.Scan(&id); err != nil {
			return nil, err
		}
		doc, err := l.store.Documents().Document(ctx, id)
		if err != nil {
			return nil, err
		}
		docs = append(docs, doc)
	}
	return docs, rows.Err()
}

func seedMemory(t *testing.T, store *sqlite.Store, id, title, note string, mindID *string, due, updated time.Time) {
	t.Helper()
	ctx := context.Background()
	now := time.Date(2026, 10, 3, 12, 0, 0, 0, time.UTC)
	if updated.IsZero() {
		updated = now
	}
	if err := store.Memories().Insert(ctx, memory.Memory{
		ID: id, MindID: mindID, Title: title, Note: note, Status: memory.StatusActive,
		DueAt: dueAtPtr(due), CreatedAt: now, UpdatedAt: updated, Rev: 1,
	}); err != nil {
		t.Fatal(err)
	}
}

func dueAtPtr(t time.Time) *time.Time {
	if t.IsZero() {
		return nil
	}
	v := t.UTC()
	return &v
}

func TestSearchPortugueseEnglishDiacriticsPrefixPhrase(t *testing.T) {
	store, idx, _ := openHarness(t)
	ctx := context.Background()
	seedMemory(t, store, "m1", "Café réunion", "nota sobre leite", nil, time.Time{}, time.Time{})
	seedMemory(t, store, "m2", "Team standup", "weekly sync in English", nil, time.Time{}, time.Time{})

	hits, err := idx.Search(ctx, "cafe reu", searchdomain.Filter{Limit: 10})
	if err != nil {
		t.Fatal(err)
	}
	if len(hits) != 1 || hits[0].MemoryID != "m1" {
		t.Fatalf("prefix/diacritics: %+v", hits)
	}
	if hits[0].Snippet == "" {
		t.Fatal("expected title snippet")
	}

	hits, err = idx.Search(ctx, `"weekly sync"`, searchdomain.Filter{Limit: 10})
	if err != nil {
		t.Fatal(err)
	}
	if len(hits) != 1 || hits[0].MemoryID != "m2" {
		t.Fatalf("phrase: %+v", hits)
	}

	hits, err = idx.Search(ctx, "one OR two", searchdomain.Filter{Limit: 10})
	if err != nil {
		t.Fatal(err)
	}
	if len(hits) != 0 {
		t.Fatalf("operators escaped: %+v", hits)
	}
}

func TestSearchFiltersMindTagStatusDates(t *testing.T) {
	store, idx, _ := openHarness(t)
	ctx := context.Background()
	now := time.Date(2026, 10, 3, 12, 0, 0, 0, time.UTC)
	parent := "root"
	if err := store.Minds().Insert(ctx, mind.Mind{ID: parent, Name: "Work", CreatedAt: now, UpdatedAt: now, Rev: 1}); err != nil {
		t.Fatal(err)
	}
	child := "child"
	if err := store.Minds().Insert(ctx, mind.Mind{ID: child, ParentID: &parent, Name: "Project", CreatedAt: now, UpdatedAt: now, Rev: 1}); err != nil {
		t.Fatal(err)
	}
	due := time.Date(2026, 10, 10, 9, 0, 0, 0, time.UTC)
	updated := time.Date(2026, 10, 5, 15, 0, 0, 0, time.UTC)
	seedMemory(t, store, "m1", "alpha task", "", &child, due, updated)
	seedMemory(t, store, "m2", "alpha inbox", "", nil, time.Time{}, now)
	if err := store.Tags().Insert(ctx, tag.Tag{ID: "t1", Name: "urgent", CreatedAt: now, UpdatedAt: now, Rev: 1}); err != nil {
		t.Fatal(err)
	}
	if err := store.Memories().SetTagIDs(ctx, "m1", []string{"t1"}); err != nil {
		t.Fatal(err)
	}
	if err := store.Memories().Update(ctx, 1, memory.Memory{
		ID: "m2", Title: "alpha inbox", Status: memory.StatusCompleted, CreatedAt: now, UpdatedAt: now, Rev: 2,
	}); err != nil {
		t.Fatal(err)
	}

	mindID := child
	hits, err := idx.Search(ctx, "alpha", searchdomain.Filter{MindID: &mindID, Limit: 10})
	if err != nil || len(hits) != 1 || hits[0].MemoryID != "m1" {
		t.Fatalf("mind filter: err=%v hits=%+v", err, hits)
	}

	hits, err = idx.Search(ctx, "alpha", searchdomain.Filter{MindID: &parent, IncludeDescendants: true, Limit: 10})
	if err != nil || len(hits) != 1 {
		t.Fatalf("descendants: %+v err=%v", hits, err)
	}

	hits, err = idx.Search(ctx, "alpha", searchdomain.Filter{TagIDs: []string{"t1"}, Limit: 10})
	if err != nil || len(hits) != 1 || hits[0].MemoryID != "m1" {
		t.Fatalf("tag filter: %+v err=%v", hits, err)
	}

	hits, err = idx.Search(ctx, "alpha", searchdomain.Filter{Status: memory.StatusCompleted, Limit: 10})
	if err != nil || len(hits) != 1 || hits[0].MemoryID != "m2" {
		t.Fatalf("status filter: %+v err=%v", hits, err)
	}

	dueFrom := due.Add(-time.Hour)
	dueTo := due.Add(time.Hour)
	hits, err = idx.Search(ctx, "alpha", searchdomain.Filter{DueFrom: &dueFrom, DueTo: &dueTo, Limit: 10})
	if err != nil || len(hits) != 1 || hits[0].MemoryID != "m1" {
		t.Fatalf("due filter: %+v err=%v", hits, err)
	}

	updFrom := updated.Add(-time.Hour)
	updTo := updated.Add(time.Hour)
	hits, err = idx.Search(ctx, "alpha", searchdomain.Filter{UpdatedFrom: &updFrom, UpdatedTo: &updTo, Limit: 10})
	if err != nil || len(hits) != 1 || hits[0].MemoryID != "m1" {
		t.Fatalf("updated filter: %+v err=%v", hits, err)
	}
}

func TestIndexRollbackBulkDeleteMindCascadeReindex(t *testing.T) {
	store, idx, svc := openHarness(t)
	ctx := context.Background()
	now := time.Date(2026, 10, 3, 12, 0, 0, 0, time.UTC)
	root := "root"
	if err := store.Minds().Insert(ctx, mind.Mind{ID: root, Name: "Root", CreatedAt: now, UpdatedAt: now, Rev: 1}); err != nil {
		t.Fatal(err)
	}
	child := "child"
	if err := store.Minds().Insert(ctx, mind.Mind{ID: child, ParentID: &root, Name: "Child", CreatedAt: now, UpdatedAt: now, Rev: 1}); err != nil {
		t.Fatal(err)
	}
	seedMemory(t, store, "m1", "rollback probe", "", nil, time.Time{}, time.Time{})
	seedMemory(t, store, "m2", "bulk one", "", &child, time.Time{}, time.Time{})
	seedMemory(t, store, "m3", "bulk two", "", &child, time.Time{}, time.Time{})
	seedMemory(t, store, "m4", "cascade keep", "", &child, time.Time{}, time.Time{})

	before, err := idx.Count(ctx)
	if err != nil {
		t.Fatal(err)
	}
	err = store.Within(ctx, func(ctx context.Context) error {
		if err := store.Memories().Insert(ctx, memory.Memory{
			ID: "ghost", Title: "ghost memory", Status: memory.StatusActive,
			CreatedAt: now, UpdatedAt: now, Rev: 1,
		}); err != nil {
			return err
		}
		return errors.New("force rollback")
	})
	if err == nil {
		t.Fatal("expected rollback error")
	}
	after, err := idx.Count(ctx)
	if err != nil {
		t.Fatal(err)
	}
	if after != before {
		t.Fatalf("rollback changed count: before=%d after=%d", before, after)
	}
	hits, err := idx.Search(ctx, "ghost", searchdomain.Filter{Limit: 10})
	if err != nil {
		t.Fatal(err)
	}
	if len(hits) != 0 {
		t.Fatalf("rollback left indexed row: %+v", hits)
	}

	for _, id := range []string{"m2", "m3"} {
		if err := store.Memories().Delete(ctx, 0, id); err != nil {
			t.Fatal(err)
		}
	}
	hits, err = idx.Search(ctx, "bulk", searchdomain.Filter{Limit: 10})
	if err != nil {
		t.Fatal(err)
	}
	if len(hits) != 0 {
		t.Fatalf("bulk delete left hits: %+v", hits)
	}

	if err := store.Minds().Update(ctx, 1, mind.Mind{ID: child, ParentID: &root, Name: "Renamed", CreatedAt: now, UpdatedAt: now, Rev: 2}); err != nil {
		t.Fatal(err)
	}
	hits, err = idx.Search(ctx, "Renamed", searchdomain.Filter{Limit: 10})
	if err != nil || len(hits) == 0 {
		t.Fatalf("mind rename reindex: %+v err=%v", hits, err)
	}

	if err := store.Minds().Delete(ctx, 0, root); err != nil {
		t.Fatal(err)
	}
	hits, err = idx.Search(ctx, "cascade", searchdomain.Filter{Limit: 10})
	if err != nil || len(hits) != 1 || hits[0].MemoryID != "m4" {
		t.Fatalf("mind cascade memory still searchable: %+v err=%v", hits, err)
	}

	if _, err := svc.Reindex(ctx); err != nil {
		t.Fatal(err)
	}
	snapshot, err := snapshotIndex(ctx, idx)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := svc.Reindex(ctx); err != nil {
		t.Fatal(err)
	}
	rebuilt, err := snapshotIndex(ctx, idx)
	if err != nil {
		t.Fatal(err)
	}
	if snapshot != rebuilt {
		t.Fatalf("reindex mismatch:\nbefore=%s\nafter=%s", snapshot, rebuilt)
	}
}

func snapshotIndex(ctx context.Context, idx *Index) (string, error) {
	rows, err := idx.readConn(ctx).QueryContext(ctx, `
		SELECT memory_id, title, note, checklist, tags, mind
		FROM memory_fts ORDER BY memory_id`)
	if err != nil {
		return "", err
	}
	defer rows.Close()
	var parts []string
	for rows.Next() {
		var id, title, note, checklist, tags, mindPath string
		if err := rows.Scan(&id, &title, &note, &checklist, &tags, &mindPath); err != nil {
			return "", err
		}
		parts = append(parts, fmt.Sprintf("%s|%s|%s|%s|%s|%s", id, title, note, checklist, tags, mindPath))
	}
	return strings.Join(parts, "\n"), rows.Err()
}
