package attachments_test

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"github.com/polterware/polterware/sparky/backend/compositions/core"
	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
)

func TestCoreAttachmentAuditAndRecoveryBeforeImport(t *testing.T) {
	ctx := context.Background()
	data := t.TempDir()
	db := filepath.Join(data, "sparky.db")
	runtime, err := core.OpenRuntime(ctx, db)
	if err != nil {
		t.Fatal(err)
	}
	m, err := runtime.Services.Memories.Create(ctx, actor.UI(), memory.Create{Title: "Attachments"})
	if err != nil {
		t.Fatal(err)
	}
	source := filepath.Join(data, "source")
	if err = os.WriteFile(source, []byte("hello"), 0600); err != nil {
		t.Fatal(err)
	}
	service := runtime.Services.Attachments
	file, err := service.ImportFromPath(ctx, m.ID, source, "", m.Rev)
	if err != nil {
		t.Fatal(err)
	}
	duplicate, err := service.ImportFromPath(ctx, m.ID, source, "", m.Rev)
	if err != nil || duplicate.ID != file.ID {
		t.Fatalf("dedup: %+v %v", duplicate, err)
	}
	link, err := service.AddLink(ctx, m.ID, "https://example.com/", "Example", m.Rev)
	if err != nil {
		t.Fatal(err)
	}
	if _, err = service.Reorder(ctx, m.ID, []string{link.ID, file.ID}); err != nil {
		t.Fatal(err)
	}
	if err = service.Delete(ctx, link.ID); err != nil {
		t.Fatal(err)
	}
	page, err := runtime.Services.Activity.List(ctx, "", "", 100)
	if err != nil {
		t.Fatal(err)
	}
	counts := map[string]int{}
	for _, event := range page.Events {
		if event.Entity == "attachment" {
			counts[event.Op]++
			if event.Actor != "ui" {
				t.Fatalf("incorrect audit actor: %+v", event)
			}
		}
	}
	if counts["create"] != 2 || counts["update"] != 1 || counts["delete"] != 1 {
		t.Fatalf("audit (dedup must be silent): %v", counts)
	}
	if err = runtime.Close(); err != nil {
		t.Fatal(err)
	}
	// Simulate a crash: one committed row lost its file, one file lacks a row,
	// and the temporary copy never reached its final rename.
	dir := filepath.Join(data, "attachments", m.ID)
	if err = os.Remove(filepath.Join(data, "attachments", *file.RelPath)); err != nil {
		t.Fatal(err)
	}
	for _, name := range []string{".import-crashed", "orphan.bin"} {
		if err = os.WriteFile(filepath.Join(dir, name), []byte("orphan"), 0600); err != nil {
			t.Fatal(err)
		}
	}
	outside := filepath.Join(data, "outside")
	if err = os.Mkdir(outside, 0700); err != nil {
		t.Fatal(err)
	}
	victim := filepath.Join(outside, "private")
	if err = os.WriteFile(victim, []byte("private"), 0600); err != nil {
		t.Fatal(err)
	}
	if err = os.Symlink(outside, filepath.Join(data, "attachments", "symlink")); err != nil {
		t.Fatal(err)
	}
	runtime, err = core.OpenRuntime(ctx, db)
	if err != nil {
		t.Fatal(err)
	}
	defer runtime.Close()
	imported, err := runtime.Services.Attachments.ImportFromPath(ctx, m.ID, source, "", m.Rev)
	if err != nil {
		t.Fatal(err)
	}
	if imported.ID == file.ID {
		t.Fatal("recovery deduplicated against missing file")
	}
	entries, err := os.ReadDir(dir)
	if err != nil || len(entries) != 1 {
		t.Fatalf("recovery left orphans: %v %v", entries, err)
	}
	if content, err := os.ReadFile(victim); err != nil || string(content) != "private" {
		t.Fatalf("sweep followed symlink: %q %v", content, err)
	}
	page, err = runtime.Services.Activity.List(ctx, "", "system", 100)
	if err != nil {
		t.Fatal(err)
	}
	if len(page.Events) != 1 || page.Events[0].EntityID != file.ID || page.Events[0].Op != "delete" {
		t.Fatalf("missing recovery audit: %+v", page.Events)
	}
}
