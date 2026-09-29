package agentsetup

import (
	"errors"
	"os"
	"path/filepath"
	"strings"
)

const (
	agentsLineBegin = "<!-- polterware:agent-setup:begin -->"
	agentsLineEnd   = "<!-- polterware:agent-setup:end -->"
	agentsLine      = "Polterware: run `polterware format` once for the Polter grammar, read with `polterware query cards`, write Work cards with `polterware apply <file|->`; start with `polterware status` and end with `polterware handoff`."
)

// SkillDir is where Codex and other agents read the polter skill.
func SkillDir() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(home, ".agents", "skills", "polter"), nil
}

// AgentsState describes the skill file and the AGENTS.md line for one project.
type AgentsState struct {
	SkillPath  string
	SkillReady bool
	LineReady  bool
}

// InspectAgents reads the state without changing it. root may be empty.
func InspectAgents(root string) AgentsState {
	var state AgentsState
	if dir, err := SkillDir(); err == nil {
		state.SkillPath = filepath.Join(dir, "SKILL.md")
		want, _ := SkillFile()
		have, err := os.ReadFile(state.SkillPath)
		state.SkillReady = err == nil && string(have) == string(want)
	}
	if root != "" {
		body, _ := os.ReadFile(filepath.Join(root, "AGENTS.md"))
		state.LineReady = strings.Contains(string(body), agentsLineBegin)
	}
	return state
}

// InstallAgents writes the skill file and, when root is set, the AGENTS.md line.
func InstallAgents(root string) error {
	dir, err := SkillDir()
	if err != nil {
		return err
	}
	skill, err := SkillFile()
	if err != nil {
		return err
	}
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return err
	}
	if err := os.WriteFile(filepath.Join(dir, "SKILL.md"), skill, 0o644); err != nil {
		return err
	}
	if root == "" {
		return nil
	}
	path := filepath.Join(root, "AGENTS.md")
	body, err := os.ReadFile(path)
	if err != nil && !errors.Is(err, os.ErrNotExist) {
		return err
	}
	text := string(body)
	if strings.Contains(text, agentsLineBegin) {
		return nil
	}
	block := agentsLineBegin + "\n" + agentsLine + "\n" + agentsLineEnd + "\n"
	if text != "" && !strings.HasSuffix(text, "\n") {
		text += "\n"
	}
	if text != "" {
		text += "\n"
	}
	return os.WriteFile(path, []byte(text+block), 0o644)
}

// UninstallAgents removes the skill file and the AGENTS.md line.
func UninstallAgents(root string) error {
	if dir, err := SkillDir(); err == nil {
		if err := os.RemoveAll(dir); err != nil {
			return err
		}
	}
	if root == "" {
		return nil
	}
	path := filepath.Join(root, "AGENTS.md")
	body, err := os.ReadFile(path)
	if err != nil {
		return nil
	}
	text := string(body)
	start := strings.Index(text, agentsLineBegin)
	end := strings.Index(text, agentsLineEnd)
	if start < 0 || end < start {
		return nil
	}
	end += len(agentsLineEnd)
	text = strings.TrimRight(text[:start], "\n") + "\n" + strings.TrimLeft(text[end:], "\n")
	if strings.TrimSpace(text) == "" {
		return os.Remove(path)
	}
	return os.WriteFile(path, []byte(text), 0o644)
}
