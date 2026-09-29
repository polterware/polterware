package polter

import (
	"os"
	"strings"
	"testing"
)

// The Claude Code plugin skill embeds Spec verbatim; keep them in sync.
func TestPluginSkillEmbedsSpec(t *testing.T) {
	body, err := os.ReadFile("../../providers/agentsetup/plugin/skills/polter/SKILL.md")
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(string(body), Spec) {
		t.Fatal("backend/providers/agentsetup/plugin/skills/polter/SKILL.md does not contain polter.Spec verbatim; copy the Spec text into its markdown fence")
	}
}
