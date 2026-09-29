package agentsetup

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os/exec"
	"strings"
	"time"
)

// PluginState is what `claude plugin list` reports for the Polterware plugin.
type PluginState struct {
	Installed bool
	Enabled   bool
	Version   string
}

// Report is the setup doctor's view of this machine.
type Report struct {
	PolterwarePath string // absolute path of `polterware` on PATH, "" when missing
	ClaudePath     string // absolute path of `claude`, "" when missing
	Plugin         PluginState
	EmbeddedVer    string
	Marketplace    string
	Problem        string // why the plugin state could not be read, if it could not
}

// Current reports whether the installed plugin matches the binary.
func (r Report) Current() bool {
	return r.Plugin.Installed && r.Plugin.Version == r.EmbeddedVer
}

// Doctor inspects the machine. It never changes anything.
func Doctor(dbDir string) Report {
	report := Report{EmbeddedVer: EmbeddedVersion(), Marketplace: MarketplaceDir(dbDir)}
	report.PolterwarePath, _ = exec.LookPath("polterware")
	report.ClaudePath, _ = exec.LookPath("claude")
	if report.ClaudePath == "" {
		return report
	}
	state, err := pluginState()
	if err != nil {
		report.Problem = err.Error()
		return report
	}
	report.Plugin = state
	return report
}

func pluginState() (PluginState, error) {
	out, err := runClaude(20*time.Second, "plugin", "list", "--json")
	if err != nil {
		return PluginState{}, err
	}
	var plugins []struct {
		ID      string `json:"id"`
		Version string `json:"version"`
		Enabled bool   `json:"enabled"`
	}
	if err := json.Unmarshal([]byte(out), &plugins); err != nil {
		return PluginState{}, fmt.Errorf("claude plugin list: unreadable output")
	}
	for _, plugin := range plugins {
		if plugin.ID == PluginID {
			return PluginState{Installed: true, Enabled: plugin.Enabled, Version: plugin.Version}, nil
		}
	}
	return PluginState{}, nil
}

// InstallClaude writes the marketplace and installs, or updates, the plugin
// through the claude CLI.
func InstallClaude(dbDir string) error {
	if _, err := exec.LookPath("claude"); err != nil {
		return errors.New("claude is not installed or not on PATH")
	}
	dir := MarketplaceDir(dbDir)
	if err := WriteMarketplace(dir); err != nil {
		return err
	}
	known, err := marketplaceKnown()
	if err != nil {
		return err
	}
	if known {
		if _, err := runClaude(60*time.Second, "plugin", "marketplace", "update", MarketplaceName); err != nil {
			return err
		}
	} else if _, err := runClaude(60*time.Second, "plugin", "marketplace", "add", dir); err != nil {
		return err
	}
	state, err := pluginState()
	if err != nil {
		return err
	}
	switch {
	case !state.Installed:
		_, err = runClaude(60*time.Second, "plugin", "install", PluginID)
	case state.Version != EmbeddedVersion():
		_, err = runClaude(60*time.Second, "plugin", "update", PluginID)
	case !state.Enabled:
		_, err = runClaude(30*time.Second, "plugin", "enable", PluginID)
	}
	return err
}

// UninstallClaude removes the plugin and the local marketplace entry.
func UninstallClaude() error {
	if _, err := exec.LookPath("claude"); err != nil {
		return errors.New("claude is not installed or not on PATH")
	}
	state, err := pluginState()
	if err != nil {
		return err
	}
	if state.Installed {
		if _, err := runClaude(60*time.Second, "plugin", "uninstall", PluginID); err != nil {
			return err
		}
	}
	known, err := marketplaceKnown()
	if err != nil || !known {
		return err
	}
	_, err = runClaude(30*time.Second, "plugin", "marketplace", "remove", MarketplaceName)
	return err
}

// SetClaudeEnabled enables or disables the installed plugin.
func SetClaudeEnabled(enabled bool) error {
	verb := "disable"
	if enabled {
		verb = "enable"
	}
	_, err := runClaude(30*time.Second, "plugin", verb, PluginID)
	return err
}

func marketplaceKnown() (bool, error) {
	out, err := runClaude(20*time.Second, "plugin", "marketplace", "list", "--json")
	if err != nil {
		return false, err
	}
	var items []struct {
		Name string `json:"name"`
	}
	if err := json.Unmarshal([]byte(out), &items); err != nil {
		return false, fmt.Errorf("claude plugin marketplace list: unreadable output")
	}
	for _, item := range items {
		if item.Name == MarketplaceName {
			return true, nil
		}
	}
	return false, nil
}

func runClaude(timeout time.Duration, args ...string) (string, error) {
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()
	out, err := exec.CommandContext(ctx, "claude", args...).CombinedOutput()
	if ctx.Err() != nil {
		return "", fmt.Errorf("claude %s timed out", strings.Join(args[:min(2, len(args))], " "))
	}
	if err != nil {
		msg := strings.TrimSpace(string(out))
		if msg == "" {
			msg = err.Error()
		}
		if i := strings.IndexByte(msg, '\n'); i >= 0 {
			msg = msg[:i]
		}
		return "", fmt.Errorf("claude: %s", msg)
	}
	return string(out), nil
}
