package attachments

import (
	"context"
	"errors"
	"os"
	"path/filepath"
	"sync"
	"testing"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
)

type testStore struct {
	mu     sync.Mutex
	files  map[string]ImportedFile
	rows   map[string]Attachment
	source string
}

func newTestStore(src string) *testStore {
	return &testStore{files: map[string]ImportedFile{}, rows: map[string]Attachment{}, source: src}
}
func (s *testStore) List(_ context.Context, m string) ([]Attachment, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	var out []Attachment
	for _, a := range s.rows {
		if a.MemoryID == m {
			out = append(out, a)
		}
	}
	return out, nil
}
func (s *testStore) Get(_ context.Context, id string) (Attachment, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	a, ok := s.rows[id]
	if !ok {
		return a, errors.New("missing")
	}
	return a, nil
}
func (s *testStore) Insert(_ context.Context, a Attachment) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.rows[a.ID] = a
	return nil
}
func (s *testStore) Delete(_ context.Context, id string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	delete(s.rows, id)
	return nil
}
func (s *testStore) Reorder(context.Context, string, []string) error { return nil }
func (s *testStore) ImportFile(_ context.Context, m, id, src string) (ImportedFile, error) {
	if src != s.source {
		return ImportedFile{}, errors.New("unexpected source")
	}
	f := ImportedFile{RelPath: filepath.Join(m, id+".dat"), SizeBytes: 5, SHA256: "same-hash", Mime: "application/octet-stream"}
	s.mu.Lock()
	s.files[f.RelPath] = f
	s.mu.Unlock()
	return f, nil
}
func (s *testStore) AbsolutePath(p string) (string, error) {
	return filepath.Join("/data/attachments", p), nil
}
func (s *testStore) RemoveFile(_ context.Context, p string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	delete(s.files, p)
	return nil
}

type testIDs struct{ n int }

func (g *testIDs) New() string { g.n++; return string(rune('a' + g.n)) }

type testClock struct{}

func (testClock) Now() time.Time { return time.Date(2026, 10, 4, 0, 0, 0, 0, time.UTC) }

func TestImportDeduplicatesHashAndNeverUsesDisplayNameForPath(t *testing.T) {
	src := filepath.Join(t.TempDir(), "source")
	if err := os.WriteFile(src, []byte("hello"), 0600); err != nil {
		t.Fatal(err)
	}
	store := newTestStore(src)
	svc := NewService(store, &testIDs{}, testClock{})
	name := "../../outside.txt"
	first, err := svc.ImportFromPath(context.Background(), "01ARZ3NDEKTSV4RRFFQ69G5FAV", src, name, 0)
	if err != nil {
		t.Fatal(err)
	}
	second, err := svc.ImportFromPath(context.Background(), first.MemoryID, src, "duplicate", 0)
	if err != nil {
		t.Fatal(err)
	}
	if first.ID != second.ID {
		t.Fatalf("duplicate hash created %q; want existing %q", second.ID, first.ID)
	}
	if filepath.IsAbs(*first.RelPath) || filepath.Base(*first.RelPath) == name {
		t.Fatalf("display name influenced stored path: %q", *first.RelPath)
	}
	if len(store.rows) != 1 {
		t.Fatalf("got %d rows, want 1", len(store.rows))
	}
}

func TestConcurrentDuplicateImportsDeduplicate(t *testing.T) {
	src := filepath.Join(t.TempDir(), "source")
	if err := os.WriteFile(src, []byte("hello"), 0600); err != nil {
		t.Fatal(err)
	}
	store := newTestStore(src)
	svc := NewService(store, &testIDs{}, testClock{})
	const mem = "01ARZ3NDEKTSV4RRFFQ69G5FAV"
	var wg sync.WaitGroup
	for i := 0; i < 8; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			if _, e := svc.ImportFromPath(context.Background(), mem, src, "", 0); e != nil {
				t.Error(e)
			}
		}()
	}
	wg.Wait()
	if len(store.rows) != 1 {
		t.Fatalf("got %d rows, want one", len(store.rows))
	}
}

type revMem struct {
	memory.Repository
	current memory.Memory
}

func (r *revMem) Get(context.Context, string) (memory.Memory, error) { return r.current, nil }

type joinTx struct{}

func (joinTx) Within(ctx context.Context, fn func(context.Context) error) error { return fn(ctx) }

func TestAddLinkExpectedRev(t *testing.T) {
	store := newTestStore("")
	svc := NewService(store, &testIDs{}, testClock{})
	svc.WithMemoryRev(&revMem{current: memory.Memory{ID: "m", Rev: 2}}, joinTx{})
	if _, err := svc.AddLink(context.Background(), "m", "https://example.com/a", "", 1); !fault.IsCode(err, fault.CodeConflict) {
		t.Fatalf("stale: %v", err)
	}
	if _, err := svc.AddLink(context.Background(), "m", "https://example.com/a", "", 0); err != nil {
		t.Fatal(err)
	}
	if _, err := svc.AddLink(context.Background(), "m", "https://example.com/b", "", 2); err != nil {
		t.Fatal(err)
	}
}
