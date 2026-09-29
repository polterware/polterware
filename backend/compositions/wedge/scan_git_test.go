package wedge_test

import (
	"context"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	"github.com/polterware/polterware/backend/compositions/wedge"
	"github.com/polterware/polterware/backend/domains/ppm"
	"github.com/polterware/polterware/backend/domains/projects"
	gitpkg "github.com/polterware/polterware/backend/providers/git"
	"github.com/polterware/polterware/backend/providers/importer"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func TestScanSkipsGitOutsideWorkTree(t *testing.T) {
	root := t.TempDir()
	if err := os.WriteFile(filepath.Join(root, "go.mod"), []byte("module github.com/acme/demo\n\ngo 1.23\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	store, project := openScanProject(t, root)
	defer store.Close()

	scan, err := wedge.ScanProject(store, project.ID)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(scan.Message, "Git metadata skipped.") {
		t.Fatalf("message: %s", scan.Message)
	}
	evidence, err := store.ListPpmEvidence(project.ID)
	if err != nil {
		t.Fatal(err)
	}
	for _, item := range evidence {
		if item.Kind == ppm.EvidenceKindCommit {
			t.Fatalf("commit evidence outside a work tree: %+v", item)
		}
	}
}

func TestScanStampsHeadOnObservedFacts(t *testing.T) {
	root := t.TempDir()
	if err := os.WriteFile(filepath.Join(root, "go.mod"), []byte("module github.com/acme/demo\n\ngo 1.23\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	gitScanCmd(t, root, "init", "-b", "main")
	gitScanCmd(t, root, "add", "go.mod")
	gitScanCmd(t, root, "commit", "-m", "first subject")

	store, project := openScanProject(t, root)
	defer store.Close()

	first, err := wedge.ScanProject(store, project.ID)
	if err != nil {
		t.Fatal(err)
	}
	if first.FactCount == 0 || !strings.Contains(first.Message, "Git branch main.") {
		t.Fatalf("first scan: %+v", first)
	}
	meta := gitpkg.ReadMetadata(context.Background(), root)
	headID := importer.StableGitHeadEvidenceID(project.ID)
	assertHeadStamp(t, store, project.ID, headID, meta.SHA, "main", "first subject")

	gitScanCmd(t, root, "commit", "--allow-empty", "-m", "second subject")
	second, err := wedge.ScanProject(store, project.ID)
	if err != nil {
		t.Fatal(err)
	}
	if second.FactCount != first.FactCount {
		t.Fatalf("fact count changed: %d -> %d", first.FactCount, second.FactCount)
	}
	if second.EvidenceCount != first.EvidenceCount+1 {
		t.Fatalf("expected one new commit row: first=%d second=%d", first.EvidenceCount, second.EvidenceCount)
	}
	next := gitpkg.ReadMetadata(context.Background(), root)
	if next.SHA == meta.SHA {
		t.Fatal("second commit did not move HEAD")
	}
	assertHeadStamp(t, store, project.ID, headID, next.SHA, "main", "second subject")

	old, err := store.GetPpmEvidence(importer.StableGitCommitEvidenceID(project.ID, meta.SHA))
	if err != nil {
		t.Fatal(err)
	}
	if old.Path != meta.SHA || old.Summary != "first subject" || old.Source != "git:commit" {
		t.Fatalf("previous commit was not kept: %+v", old)
	}
}

func TestScanRecordsNamedRepository(t *testing.T) {
	root := t.TempDir()
	if err := os.WriteFile(filepath.Join(root, "go.mod"), []byte("module github.com/acme/demo\n\ngo 1.23\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	gitScanCmd(t, root, "init", "-b", "main")
	gitScanCmd(t, root, "add", "go.mod")
	gitScanCmd(t, root, "commit", "-m", "root subject")

	extra := t.TempDir()
	if err := os.WriteFile(filepath.Join(extra, "README.md"), []byte("# api\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	gitScanCmd(t, extra, "init", "-b", "feature")
	gitScanCmd(t, extra, "add", "README.md")
	gitScanCmd(t, extra, "commit", "-m", "api subject")

	store, project := openScanProject(t, root)
	defer store.Close()
	if _, err := wedge.AddProjectRepo(store, project.ID, "api", extra); err != nil {
		t.Fatal(err)
	}

	scan, err := wedge.ScanProject(store, project.ID)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(scan.Message, "Git branch main.") || !strings.Contains(scan.Message, "Git api branch feature.") {
		t.Fatalf("message: %s", scan.Message)
	}
	rootMeta := gitpkg.ReadMetadata(context.Background(), root)
	extraMeta := gitpkg.ReadMetadata(context.Background(), extra)
	rootHead := importer.StableGitHeadEvidenceID(project.ID)
	apiHead := importer.StableGitRepoHeadEvidenceID(project.ID, "api")
	assertHeadStamp(t, store, project.ID, rootHead, rootMeta.SHA, "main", "root subject")

	head, err := store.GetPpmEvidence(apiHead)
	if err != nil {
		t.Fatal(err)
	}
	if head.Source != "git:api:HEAD" || head.Path != extraMeta.SHA || head.Summary != "feature" {
		t.Fatalf("api head: %+v", head)
	}
	commit, err := store.GetPpmEvidence(importer.StableGitRepoCommitEvidenceID(project.ID, "api", extraMeta.SHA))
	if err != nil {
		t.Fatal(err)
	}
	if commit.Source != "git:api:commit" || commit.Summary != "api subject" {
		t.Fatalf("api commit: %+v", commit)
	}
	facts, err := store.ListPpmFacts(project.ID, "")
	if err != nil {
		t.Fatal(err)
	}
	for _, fact := range facts {
		linked, err := store.ListPpmEvidenceForFact(fact.ID)
		if err != nil {
			t.Fatal(err)
		}
		for _, item := range linked {
			if item.ID == apiHead {
				t.Fatalf("root fact %s was stamped with the api checkout", fact.ID)
			}
		}
	}
}

func assertHeadStamp(t *testing.T, store *storepkg.Store, projectID, headID, sha, branch, subject string) {
	t.Helper()
	head, err := store.GetPpmEvidence(headID)
	if err != nil {
		t.Fatal(err)
	}
	if head.Kind != ppm.EvidenceKindCommit || head.Source != "git:HEAD" || head.Path != sha || head.Summary != branch {
		t.Fatalf("head evidence: %+v", head)
	}
	commit, err := store.GetPpmEvidence(importer.StableGitCommitEvidenceID(projectID, sha))
	if err != nil {
		t.Fatal(err)
	}
	if commit.Kind != ppm.EvidenceKindCommit || commit.Source != "git:commit" || commit.Summary != subject || commit.Path != sha {
		t.Fatalf("commit evidence: %+v", commit)
	}
	facts, err := store.ListPpmFacts(projectID, "")
	if err != nil {
		t.Fatal(err)
	}
	if len(facts) == 0 {
		t.Fatal("expected observed facts")
	}
	for _, fact := range facts {
		linked, err := store.ListPpmEvidenceForFact(fact.ID)
		if err != nil {
			t.Fatal(err)
		}
		found := false
		for _, item := range linked {
			if item.ID == headID {
				found = true
				break
			}
		}
		if !found {
			t.Fatalf("fact %s is not stamped with HEAD", fact.ID)
		}
	}
}

func openScanProject(t *testing.T, root string) (*storepkg.Store, projects.Project) {
	t.Helper()
	store, err := storepkg.New(filepath.Join(t.TempDir(), "scan.db"))
	if err != nil {
		t.Fatal(err)
	}
	project, err := wedge.InitProject(store, wedge.InitInput{Name: "Demo", RootPath: root})
	if err != nil {
		store.Close()
		t.Fatal(err)
	}
	return store, project
}

func gitScanCmd(t *testing.T, dir string, args ...string) {
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
