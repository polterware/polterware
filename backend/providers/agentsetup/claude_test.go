package agentsetup

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

// fakeClaude installs a `claude` script on PATH that logs its arguments and
// answers the plugin and marketplace list queries from JSON files.
type fakeClaude struct {
	dir string
}

func newFakeClaude(t *testing.T, plugins, marketplaces string) *fakeClaude {
	t.Helper()
	dir := t.TempDir()
	script := `#!/bin/sh
echo "$@" >> "` + dir + `/calls.log"
case "$1 $2 $3" in
  "plugin list --json") cat "` + dir + `/plugins.json" ;;
  "plugin marketplace list") cat "` + dir + `/marketplaces.json" ;;
esac
exit 0
`
	if err := os.WriteFile(filepath.Join(dir, "claude"), []byte(script), 0o755); err != nil {
		t.Fatal(err)
	}
	f := &fakeClaude{dir: dir}
	f.set(t, plugins, marketplaces)
	t.Setenv("PATH", dir+string(os.PathListSeparator)+"/usr/bin:/bin")
	return f
}

func (f *fakeClaude) set(t *testing.T, plugins, marketplaces string) {
	t.Helper()
	for name, body := range map[string]string{"plugins.json": plugins, "marketplaces.json": marketplaces} {
		if err := os.WriteFile(filepath.Join(f.dir, name), []byte(body), 0o644); err != nil {
			t.Fatal(err)
		}
	}
}

func (f *fakeClaude) calls(t *testing.T) []string {
	t.Helper()
	body, err := os.ReadFile(filepath.Join(f.dir, "calls.log"))
	if os.IsNotExist(err) {
		return nil
	}
	if err != nil {
		t.Fatal(err)
	}
	return strings.Split(strings.TrimSpace(string(body)), "\n")
}

func pluginJSON(version string, enabled bool) string {
	enabledText := "false"
	if enabled {
		enabledText = "true"
	}
	return `[{"id":"` + PluginID + `","version":"` + version + `","enabled":` + enabledText + `}]`
}

func marketplaceJSON(known bool) string {
	if known {
		return `[{"name":"` + MarketplaceName + `"}]`
	}
	return `[]`
}

func hasCall(calls []string, prefix string) bool {
	for _, call := range calls {
		if strings.HasPrefix(call, prefix) {
			return true
		}
	}
	return false
}

func TestInstallClaudeFailsWithoutBinary(t *testing.T) {
	t.Setenv("PATH", t.TempDir())
	if err := InstallClaude(t.TempDir()); err == nil || !strings.Contains(err.Error(), "not installed") {
		t.Fatalf("err = %v; want a missing-claude error", err)
	}
	if err := UninstallClaude(); err == nil {
		t.Fatal("UninstallClaude must fail without claude")
	}
}

func TestInstallClaudeFreshMachineAddsMarketplaceAndInstalls(t *testing.T) {
	fake := newFakeClaude(t, `[]`, marketplaceJSON(false))
	dbDir := t.TempDir()
	if err := InstallClaude(dbDir); err != nil {
		t.Fatal(err)
	}
	calls := fake.calls(t)
	if !hasCall(calls, "plugin marketplace add "+MarketplaceDir(dbDir)) {
		t.Fatalf("marketplace was not added: %v", calls)
	}
	if !hasCall(calls, "plugin install "+PluginID) {
		t.Fatalf("plugin was not installed: %v", calls)
	}
	if _, err := os.Stat(MarketplaceDir(dbDir)); err != nil {
		t.Fatalf("marketplace directory was not written: %v", err)
	}
}

func TestInstallClaudeUpdatesOutdatedPlugin(t *testing.T) {
	fake := newFakeClaude(t, pluginJSON("0.0.0-old", true), marketplaceJSON(true))
	if err := InstallClaude(t.TempDir()); err != nil {
		t.Fatal(err)
	}
	calls := fake.calls(t)
	if !hasCall(calls, "plugin marketplace update "+MarketplaceName) {
		t.Fatalf("known marketplace must be updated: %v", calls)
	}
	if !hasCall(calls, "plugin update "+PluginID) {
		t.Fatalf("outdated plugin must be updated: %v", calls)
	}
	if hasCall(calls, "plugin install") || hasCall(calls, "plugin marketplace add") {
		t.Fatalf("unexpected fresh-install calls: %v", calls)
	}
}

func TestInstallClaudeEnablesDisabledCurrentPlugin(t *testing.T) {
	fake := newFakeClaude(t, pluginJSON(EmbeddedVersion(), false), marketplaceJSON(true))
	if err := InstallClaude(t.TempDir()); err != nil {
		t.Fatal(err)
	}
	if !hasCall(fake.calls(t), "plugin enable "+PluginID) {
		t.Fatalf("disabled plugin must be enabled: %v", fake.calls(t))
	}
}

func TestInstallClaudeLeavesCurrentPluginAlone(t *testing.T) {
	fake := newFakeClaude(t, pluginJSON(EmbeddedVersion(), true), marketplaceJSON(true))
	if err := InstallClaude(t.TempDir()); err != nil {
		t.Fatal(err)
	}
	for _, verb := range []string{"plugin install", "plugin update", "plugin enable"} {
		if hasCall(fake.calls(t), verb) {
			t.Fatalf("%q must not run for a current plugin: %v", verb, fake.calls(t))
		}
	}
}

func TestUninstallClaudeRemovesPluginAndMarketplace(t *testing.T) {
	fake := newFakeClaude(t, pluginJSON(EmbeddedVersion(), true), marketplaceJSON(true))
	if err := UninstallClaude(); err != nil {
		t.Fatal(err)
	}
	calls := fake.calls(t)
	if !hasCall(calls, "plugin uninstall "+PluginID) || !hasCall(calls, "plugin marketplace remove "+MarketplaceName) {
		t.Fatalf("calls = %v", calls)
	}
}

func TestUninstallClaudeWhenNothingInstalled(t *testing.T) {
	fake := newFakeClaude(t, `[]`, marketplaceJSON(false))
	if err := UninstallClaude(); err != nil {
		t.Fatal(err)
	}
	if hasCall(fake.calls(t), "plugin uninstall") || hasCall(fake.calls(t), "plugin marketplace remove") {
		t.Fatalf("nothing to remove, got %v", fake.calls(t))
	}
}

func TestSetClaudeEnabledUsesMatchingVerb(t *testing.T) {
	fake := newFakeClaude(t, `[]`, `[]`)
	if err := SetClaudeEnabled(true); err != nil {
		t.Fatal(err)
	}
	if err := SetClaudeEnabled(false); err != nil {
		t.Fatal(err)
	}
	calls := fake.calls(t)
	if len(calls) != 2 || calls[0] != "plugin enable "+PluginID || calls[1] != "plugin disable "+PluginID {
		t.Fatalf("calls = %v", calls)
	}
}

func TestDoctorReportsPluginState(t *testing.T) {
	fake := newFakeClaude(t, pluginJSON(EmbeddedVersion(), true), marketplaceJSON(true))
	report := Doctor(t.TempDir())
	if report.ClaudePath == "" || !report.Plugin.Installed || !report.Plugin.Enabled {
		t.Fatalf("report = %+v", report)
	}
	if !report.Current() {
		t.Fatalf("plugin at the embedded version must be current: %+v", report)
	}

	fake.set(t, pluginJSON("0.0.0-old", true), marketplaceJSON(true))
	if Doctor(t.TempDir()).Current() {
		t.Fatal("an outdated plugin must not be current")
	}
}

func TestDoctorWithoutClaude(t *testing.T) {
	t.Setenv("PATH", t.TempDir())
	report := Doctor(t.TempDir())
	if report.ClaudePath != "" || report.Plugin.Installed || report.Current() {
		t.Fatalf("report = %+v", report)
	}
}

func TestDoctorReportsUnreadablePluginList(t *testing.T) {
	newFakeClaude(t, `not json`, marketplaceJSON(true))
	if report := Doctor(t.TempDir()); report.Problem == "" {
		t.Fatalf("want a problem for unreadable output: %+v", report)
	}
}

func TestRunClaudeReportsFirstLineOfFailure(t *testing.T) {
	dir := t.TempDir()
	script := "#!/bin/sh\necho 'boom on line one'\necho 'second line'\nexit 3\n"
	if err := os.WriteFile(filepath.Join(dir, "claude"), []byte(script), 0o755); err != nil {
		t.Fatal(err)
	}
	t.Setenv("PATH", dir+string(os.PathListSeparator)+"/usr/bin:/bin")
	_, err := runClaude(5*time.Second, "plugin", "list")
	if err == nil || err.Error() != "claude: boom on line one" {
		t.Fatalf("err = %v", err)
	}
}

func TestRunClaudeTimesOut(t *testing.T) {
	dir := t.TempDir()
	if err := os.WriteFile(filepath.Join(dir, "claude"), []byte("#!/bin/sh\nexec sleep 5\n"), 0o755); err != nil {
		t.Fatal(err)
	}
	t.Setenv("PATH", dir+string(os.PathListSeparator)+"/usr/bin:/bin")
	_, err := runClaude(200*time.Millisecond, "plugin", "list")
	if err == nil || !strings.Contains(err.Error(), "timed out") {
		t.Fatalf("err = %v; want timeout", err)
	}
}
