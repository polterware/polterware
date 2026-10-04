package attachments

import (
	"context"
	"os"
	"path/filepath"
	"sync"
	"testing"
)

const testMemoryID = "01ARZ3NDEKTSV4RRFFQ69G5FAV"
const testAttachmentID = "01ARZ3NDEKTSV4RRFFQ69G5FAW"

func TestImportFileSafetyAndCrashShape(t *testing.T) {
	root := t.TempDir()
	s, err := New(nil, root)
	if err != nil {
		t.Fatal(err)
	}
	source := filepath.Join(root, "../input.txt")
	if err = os.WriteFile(source, []byte("hello"), 0600); err != nil {
		t.Fatal(err)
	}
	got, err := s.ImportFile(context.Background(), testMemoryID, testAttachmentID, source)
	if err != nil {
		t.Fatal(err)
	}
	if filepath.IsAbs(got.RelPath) || filepath.Base(got.RelPath) == "input.txt" {
		t.Fatalf("path includes source name: %q", got.RelPath)
	}
	if _, err = os.Stat(filepath.Join(root, "attachments", got.RelPath)); err != nil {
		t.Fatal(err)
	}
	if _, err = s.ImportFile(context.Background(), "../../escape", testAttachmentID, source); err == nil {
		t.Fatal("traversal ID accepted")
	}
	if _, err = s.ImportFile(context.Background(), testMemoryID, "../../escape", source); err == nil {
		t.Fatal("absolute/traversal ID accepted")
	}
	link := filepath.Join(root, "link")
	if err = os.Symlink(source, link); err != nil {
		t.Fatal(err)
	}
	if _, err = s.ImportFile(context.Background(), testMemoryID, testAttachmentID, link); err == nil {
		t.Fatal("symlink accepted")
	}
	large := filepath.Join(root, "large")
	f, err := os.Create(large)
	if err != nil {
		t.Fatal(err)
	}
	if err = f.Truncate(500<<20 + 1); err != nil {
		t.Fatal(err)
	}
	f.Close()
	if _, err = s.ImportFile(context.Background(), testMemoryID, testAttachmentID, large); err == nil {
		t.Fatal("oversized file accepted")
	}
}

func TestConcurrentImportsUseIndependentTemporaryFiles(t *testing.T) {
	root := t.TempDir()
	s, err := New(nil, root)
	if err != nil {
		t.Fatal(err)
	}
	src := filepath.Join(root, "src")
	if err = os.WriteFile(src, []byte("concurrent"), 0600); err != nil {
		t.Fatal(err)
	}
	ids := []string{"01ARZ3NDEKTSV4RRFFQ69G5FAW", "01ARZ3NDEKTSV4RRFFQ69G5FAX", "01ARZ3NDEKTSV4RRFFQ69G5FAY", "01ARZ3NDEKTSV4RRFFQ69G5FAZ"}
	var wg sync.WaitGroup
	errs := make(chan error, len(ids))
	for _, id := range ids {
		wg.Add(1)
		go func(id string) {
			defer wg.Done()
			_, e := s.ImportFile(context.Background(), testMemoryID, id, src)
			errs <- e
		}(id)
	}
	wg.Wait()
	close(errs)
	for e := range errs {
		if e != nil {
			t.Fatal(e)
		}
	}
}
