package agentsetup

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func codexHome(t *testing.T) string {
	t.Helper()
	t.Setenv("HOME", t.TempDir())
	return t.TempDir()
}

func TestInstallAgentsWritesSkillAndAgentsLine(t *testing.T) {
	root := codexHome(t)
	if err := InstallAgents(root); err != nil {
		t.Fatal(err)
	}
	state := InspectAgents(root)
	if !state.SkillReady || !state.LineReady {
		t.Fatalf("state = %+v", state)
	}
	body, err := os.ReadFile(filepath.Join(root, "AGENTS.md"))
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(string(body), "polterware status") {
		t.Fatalf("AGENTS.md missing the Polterware line:\n%s", body)
	}
}

func TestInstallAgentsWithoutRootOnlyWritesSkill(t *testing.T) {
	codexHome(t)
	if err := InstallAgents(""); err != nil {
		t.Fatal(err)
	}
	state := InspectAgents("")
	if !state.SkillReady || state.LineReady {
		t.Fatalf("state = %+v", state)
	}
}

func TestInstallAgentsIsIdempotent(t *testing.T) {
	root := codexHome(t)
	for range 3 {
		if err := InstallAgents(root); err != nil {
			t.Fatal(err)
		}
	}
	body, _ := os.ReadFile(filepath.Join(root, "AGENTS.md"))
	if got := strings.Count(string(body), agentsLineBegin); got != 1 {
		t.Fatalf("block written %d times:\n%s", got, body)
	}
}

func TestInstallAgentsKeepsExistingContent(t *testing.T) {
	root := codexHome(t)
	path := filepath.Join(root, "AGENTS.md")
	const original = "# Rules\n\nBe kind.\n"
	if err := os.WriteFile(path, []byte(original), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := InstallAgents(root); err != nil {
		t.Fatal(err)
	}
	body, _ := os.ReadFile(path)
	if !strings.HasPrefix(string(body), original) {
		t.Fatalf("existing content was changed:\n%s", body)
	}
}

func TestInstallAgentsAddsNewlineBeforeBlock(t *testing.T) {
	root := codexHome(t)
	path := filepath.Join(root, "AGENTS.md")
	if err := os.WriteFile(path, []byte("no trailing newline"), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := InstallAgents(root); err != nil {
		t.Fatal(err)
	}
	body, _ := os.ReadFile(path)
	if !strings.Contains(string(body), "no trailing newline\n\n"+agentsLineBegin) {
		t.Fatalf("block must start on its own paragraph:\n%q", body)
	}
}

func TestUninstallAgentsKeepsOtherContent(t *testing.T) {
	root := codexHome(t)
	path := filepath.Join(root, "AGENTS.md")
	const original = "# Rules\n\nBe kind.\n"
	if err := os.WriteFile(path, []byte(original), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := InstallAgents(root); err != nil {
		t.Fatal(err)
	}
	if err := UninstallAgents(root); err != nil {
		t.Fatal(err)
	}
	body, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(string(body), "polterware") || !strings.Contains(string(body), "Be kind.") {
		t.Fatalf("uninstall left the wrong content:\n%s", body)
	}
	if state := InspectAgents(root); state.SkillReady || state.LineReady {
		t.Fatalf("state after uninstall = %+v", state)
	}
}

func TestUninstallAgentsDeletesFileThatOnlyHeldTheBlock(t *testing.T) {
	root := codexHome(t)
	if err := InstallAgents(root); err != nil {
		t.Fatal(err)
	}
	if err := UninstallAgents(root); err != nil {
		t.Fatal(err)
	}
	if _, err := os.Stat(filepath.Join(root, "AGENTS.md")); !os.IsNotExist(err) {
		t.Fatalf("AGENTS.md should be removed, stat err = %v", err)
	}
}

func TestUninstallAgentsWithoutInstallIsANoop(t *testing.T) {
	root := codexHome(t)
	path := filepath.Join(root, "AGENTS.md")
	if err := os.WriteFile(path, []byte("keep me\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := UninstallAgents(root); err != nil {
		t.Fatal(err)
	}
	if body, _ := os.ReadFile(path); string(body) != "keep me\n" {
		t.Fatalf("file changed: %q", body)
	}
	if err := UninstallAgents(filepath.Join(root, "missing")); err != nil {
		t.Fatalf("missing root must not fail: %v", err)
	}
}

func TestInspectAgentsDetectsStaleSkill(t *testing.T) {
	codexHome(t)
	if err := InstallAgents(""); err != nil {
		t.Fatal(err)
	}
	dir, err := SkillDir()
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, "SKILL.md"), []byte("stale"), 0o644); err != nil {
		t.Fatal(err)
	}
	if InspectAgents("").SkillReady {
		t.Fatal("an edited skill file must not count as ready")
	}
}
