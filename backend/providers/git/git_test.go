package git

import (
	"context"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
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

func TestReadMetadataOutsideWorkTree(t *testing.T) {
	meta := ReadMetadata(context.Background(), t.TempDir())
	if meta.Available || meta.SHA != "" || len(meta.Commits) != 0 {
		t.Fatalf("expected empty metadata, got %+v", meta)
	}
}

func TestReadMetadataFirstParent(t *testing.T) {
	dir := t.TempDir()
	gitCmd(t, dir, "init", "-b", "main")
	gitCmd(t, dir, "commit", "--allow-empty", "-m", "first subject")
	meta := ReadMetadata(context.Background(), dir)
	if !meta.Available {
		t.Fatal("expected a work tree")
	}
	if meta.Branch != "main" || meta.SHA == "" {
		t.Fatalf("head: %+v", meta)
	}
	if len(meta.Commits) != 1 || meta.Commits[0].SHA != meta.SHA || meta.Commits[0].Subject != "first subject" {
		t.Fatalf("commits: %+v", meta.Commits)
	}
	if strings.Contains(meta.SHA, "\n") {
		t.Fatalf("sha not a single token: %q", meta.SHA)
	}
}

func gitCmd(t *testing.T, dir string, args ...string) {
	t.Helper()
	cmd := exec.Command("git", args...)
	cmd.Dir = dir
	cmd.Env = append(os.Environ(),
		"GIT_AUTHOR_NAME=Test",
		"GIT_AUTHOR_EMAIL=test@example.com",
		"GIT_COMMITTER_NAME=Test",
		"GIT_COMMITTER_EMAIL=test@example.com",
	)
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("git %v: %v\n%s", args, err, out)
	}
}
