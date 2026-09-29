package agentsetup

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestWriteMarketplaceMatchesEmbeddedPlugin(t *testing.T) {
	dir := t.TempDir()
	if err := WriteMarketplace(dir); err != nil {
		t.Fatal(err)
	}
	for _, rel := range []string{
		".claude-plugin/marketplace.json",
		"plugins/polterware/.claude-plugin/plugin.json",
		"plugins/polterware/hooks/hooks.json",
		"plugins/polterware/skills/polter/SKILL.md",
	} {
		if _, err := os.Stat(filepath.Join(dir, rel)); err != nil {
			t.Fatalf("%s: %v", rel, err)
		}
	}
	if EmbeddedVersion() == "" {
		t.Fatal("embedded plugin has no version")
	}
	// A stale file from an older plugin must not survive a rewrite.
	stale := filepath.Join(dir, "plugins", "polterware", "old.txt")
	if err := os.WriteFile(stale, []byte("x"), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := WriteMarketplace(dir); err != nil {
		t.Fatal(err)
	}
	if _, err := os.Stat(stale); !os.IsNotExist(err) {
		t.Fatalf("stale file kept: %v", err)
	}
}

func TestHooksJSONCallsOnlyKnownSwitches(t *testing.T) {
	body, err := pluginFS.ReadFile(pluginRoot + "/hooks/hooks.json")
	if err != nil {
		t.Fatal(err)
	}
	for _, sub := range []string{"session-start", "session-end", "status", "stop", "lint"} {
		if !strings.Contains(string(body), "polterware hook "+sub) {
			t.Fatalf("hooks.json does not call %s", sub)
		}
		if !ValidHook(SwitchFor(sub)) {
			t.Fatalf("%s has no switch", sub)
		}
	}
}

func TestAgentsLineRoundTrip(t *testing.T) {
	home := t.TempDir()
	t.Setenv("HOME", home)
	root := t.TempDir()
	path := filepath.Join(root, "AGENTS.md")
	if err := os.WriteFile(path, []byte("# Rules\n\nBe kind.\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := InstallAgents(root); err != nil {
		t.Fatal(err)
	}
	if err := InstallAgents(root); err != nil {
		t.Fatal(err)
	}
	body, _ := os.ReadFile(path)
	if strings.Count(string(body), agentsLineBegin) != 1 {
		t.Fatalf("line not idempotent:\n%s", body)
	}
	state := InspectAgents(root)
	if !state.SkillReady || !state.LineReady {
		t.Fatalf("state = %+v", state)
	}
	if err := UninstallAgents(root); err != nil {
		t.Fatal(err)
	}
	body, _ = os.ReadFile(path)
	if strings.TrimSpace(string(body)) != "# Rules\n\nBe kind." {
		t.Fatalf("user text changed:\n%q", body)
	}
	if InspectAgents(root).SkillReady {
		t.Fatal("skill file kept")
	}
}
