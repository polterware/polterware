package git

import (
	"context"
	"os"
	"os/exec"
	"path/filepath"
	"testing"
)

func TestStatusFixture(t *testing.T) {
	dir := t.TempDir()
	_ = exec.Command("git", "init", dir).Run()
	_ = os.WriteFile(filepath.Join(dir, "a.txt"), []byte("x"), 0o644)
	out, err := Status(context.Background(), dir)
	if err != nil {
		t.Fatal(err)
	}
	if out == "" {
		// clean ok after add? porcelain may be non-empty
	}
	_ = out
}
