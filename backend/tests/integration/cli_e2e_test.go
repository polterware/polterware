package integration

import (
	"bufio"
	"bytes"
	"encoding/json"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/polterware/polterware/backend/platform/coreclient"
)

// cliHarness runs the real `polterware` binary against an isolated database,
// the way Claude Code, Codex, and Cursor do. The core it starts lives in the
// temp directory and is stopped when the test ends.
type cliHarness struct {
	t    *testing.T
	bin  string
	db   string
	home string
}

func newCLIHarness(t *testing.T) *cliHarness {
	t.Helper()
	if testing.Short() {
		t.Skip("builds the polterware binary")
	}
	dir := t.TempDir()
	h := &cliHarness{t: t, bin: filepath.Join(dir, "polterware"), db: filepath.Join(dir, "data", "local.db"), home: filepath.Join(dir, "home")}
	if err := os.MkdirAll(h.home, 0o755); err != nil {
		t.Fatal(err)
	}
	build := exec.Command("go", "build", "-o", h.bin, "./runtimes/cli")
	build.Dir = "../.."
	if out, err := build.CombinedOutput(); err != nil {
		t.Fatalf("go build: %v\n%s", err, out)
	}
	t.Cleanup(func() { _ = coreclient.Stop(h.db) })
	return h
}

func (h *cliHarness) command(args ...string) *exec.Cmd {
	cmd := exec.Command(h.bin, append([]string{"-db", h.db}, args...)...)
	cmd.Env = append(os.Environ(), "HOME="+h.home, "XDG_CONFIG_HOME="+filepath.Join(h.home, ".config"), "POLTERWARE_CORE_IDLE=1m")
	return cmd
}

func (h *cliHarness) run(stdin string, args ...string) (string, error) {
	h.t.Helper()
	cmd := h.command(args...)
	if stdin != "" {
		cmd.Stdin = strings.NewReader(stdin)
	}
	var stdout, stderr bytes.Buffer
	cmd.Stdout, cmd.Stderr = &stdout, &stderr
	done := make(chan error, 1)
	if err := cmd.Start(); err != nil {
		h.t.Fatal(err)
	}
	go func() { done <- cmd.Wait() }()
	select {
	case err := <-done:
		if err != nil {
			return stdout.String() + stderr.String(), err
		}
		return stdout.String(), nil
	case <-time.After(60 * time.Second):
		_ = cmd.Process.Kill()
		h.t.Fatalf("polterware %v timed out\nstdout: %s\nstderr: %s", args, stdout.String(), stderr.String())
		return "", nil
	}
}

func (h *cliHarness) mustRun(stdin string, args ...string) string {
	h.t.Helper()
	out, err := h.run(stdin, args...)
	if err != nil {
		h.t.Fatalf("polterware %v: %v\n%s", args, err, out)
	}
	return out
}

const e2eRoadmap = `---
polter: 1
summary: Plan the local build
rationale: The build must be usable from every agent
---

## local-build: Local build
stage: plan

Ship a build that agents can use on this machine.

- [ ] T1 Wire the MCP bridge
? Ship unsigned? [blocking] (yes | no)
`

func TestBinaryFormatNeedsNoCore(t *testing.T) {
	h := newCLIHarness(t)
	out := h.mustRun("", "format")
	if !strings.Contains(out, "Polter format v1") {
		t.Fatalf("format output: %s", out)
	}
	if _, err := os.Stat(h.db); err == nil {
		t.Fatal("format must not create the database")
	}
}

func TestBinaryUnknownCommandFails(t *testing.T) {
	h := newCLIHarness(t)
	out, err := h.run("", "definitely-not-a-command")
	if err == nil {
		t.Fatalf("an unknown command must exit non-zero: %s", out)
	}
	var exit *exec.ExitError
	if !asExit(err, &exit) || exit.ExitCode() != 2 {
		t.Fatalf("exit = %v; want status 2", err)
	}
}

func asExit(err error, target **exec.ExitError) bool {
	exit, ok := err.(*exec.ExitError)
	if ok {
		*target = exit
	}
	return ok
}

func TestBinaryInitApplyQueryRoundTrip(t *testing.T) {
	h := newCLIHarness(t)
	root := t.TempDir()
	out := h.mustRun("", "init", "--id", "proj-e2e", "--name", "E2E", "--path", root)
	if !strings.Contains(out, "proj-e2e") {
		t.Fatalf("init output: %s", out)
	}

	plan := h.mustRun(e2eRoadmap, "apply", "-", "--project", "proj-e2e", "--dry-run")
	if !strings.Contains(plan, "local-build") {
		t.Fatalf("dry run plan: %s", plan)
	}
	if cards := h.mustRun("", "query", "--project", "proj-e2e", "cards"); strings.Contains(cards, "local-build") {
		t.Fatalf("a dry run must not write: %s", cards)
	}

	h.mustRun(e2eRoadmap, "apply", "-", "--project", "proj-e2e")
	card := h.mustRun("", "query", "--project", "proj-e2e", "card local-build")
	for _, want := range []string{"local-build", "T1", "Ship unsigned?"} {
		if !strings.Contains(card, want) {
			t.Fatalf("card is missing %q:\n%s", want, card)
		}
	}
	if status := h.mustRun("", "status", "--project", "proj-e2e"); status == "" {
		t.Fatal("status printed nothing")
	}
}

func TestBinaryApplyRejectsInvalidPolterWithDiagnostics(t *testing.T) {
	h := newCLIHarness(t)
	h.mustRun("", "init", "--id", "proj-bad", "--name", "Bad", "--path", t.TempDir())
	bad := "---\npolter: 1\nsummary: x\nrationale: y\n---\n\n- [ ] orphan task\n"
	out, err := h.run(bad, "apply", "-", "--project", "proj-bad")
	if err == nil {
		t.Fatalf("invalid Polter must fail: %s", out)
	}
	if !strings.Contains(out, "<stdin>:7") {
		t.Fatalf("want a diagnostic anchored at <stdin>:7, got: %s", out)
	}
}

func TestBinaryStateSurvivesACoreRestart(t *testing.T) {
	h := newCLIHarness(t)
	h.mustRun("", "init", "--id", "proj-keep", "--name", "Keep", "--path", t.TempDir())
	h.mustRun(e2eRoadmap, "apply", "-", "--project", "proj-keep")
	if err := coreclient.Stop(h.db); err != nil {
		t.Fatal(err)
	}
	card := h.mustRun("", "query", "--project", "proj-keep", "card local-build")
	if !strings.Contains(card, "local-build") {
		t.Fatalf("the card was lost across a core restart:\n%s", card)
	}
}

type mcpLine struct {
	ID     json.RawMessage `json:"id"`
	Result json.RawMessage `json:"result"`
	Error  *struct {
		Message string `json:"message"`
	} `json:"error"`
}

// TestBinaryMCPBridgeServesStdio is the path Cursor, Claude Code, and Codex
// use: they spawn `polterware -db <db> mcp` and speak JSON-RPC on stdio.
func TestBinaryMCPBridgeServesStdio(t *testing.T) {
	h := newCLIHarness(t)
	h.mustRun("", "init", "--id", "proj-mcp", "--name", "MCP", "--path", t.TempDir())

	cmd := h.command("mcp")
	stdin, err := cmd.StdinPipe()
	if err != nil {
		t.Fatal(err)
	}
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		t.Fatal(err)
	}
	if err := cmd.Start(); err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = stdin.Close(); _ = cmd.Process.Kill(); _ = cmd.Wait() })

	lines := make(chan string)
	go func() {
		scanner := bufio.NewScanner(stdout)
		scanner.Buffer(make([]byte, 0, 64*1024), 4<<20)
		for scanner.Scan() {
			lines <- scanner.Text()
		}
		close(lines)
	}()
	call := func(message string) mcpLine {
		t.Helper()
		if _, err := stdin.Write([]byte(message + "\n")); err != nil {
			t.Fatal(err)
		}
		select {
		case line, ok := <-lines:
			if !ok {
				t.Fatal("bridge closed stdout")
			}
			var reply mcpLine
			if err := json.Unmarshal([]byte(line), &reply); err != nil {
				t.Fatalf("not JSON-RPC: %q", line)
			}
			return reply
		case <-time.After(30 * time.Second):
			t.Fatalf("no reply to %s", message)
			return mcpLine{}
		}
	}

	init := call(`{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","clientInfo":{"name":"e2e-client","version":"1"}}}`)
	if init.Error != nil || !strings.Contains(string(init.Result), "polterware-mcp") {
		t.Fatalf("initialize = %+v", init)
	}
	// Notifications get no reply, so the next reply must belong to tools/list.
	if _, err := stdin.Write([]byte(`{"jsonrpc":"2.0","method":"notifications/initialized"}` + "\n")); err != nil {
		t.Fatal(err)
	}
	tools := call(`{"jsonrpc":"2.0","id":2,"method":"tools/list"}`)
	if tools.Error != nil || !strings.Contains(string(tools.Result), "polter.apply") {
		t.Fatalf("tools/list = %+v", tools)
	}
	format := call(`{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"polter.format","arguments":{}}}`)
	if format.Error != nil || !strings.Contains(string(format.Result), "Polter format v1") {
		t.Fatalf("polter.format = %+v", format)
	}

	apply, _ := json.Marshal(map[string]any{
		"jsonrpc": "2.0", "id": 4, "method": "tools/call",
		"params": map[string]any{"name": "polter.apply", "arguments": map[string]any{"project_id": "proj-mcp", "text": e2eRoadmap}},
	})
	if reply := call(string(apply)); reply.Error != nil || strings.Contains(string(reply.Result), `"isError":true`) {
		t.Fatalf("polter.apply over MCP = %+v", reply)
	}
	// A card written through MCP is visible to the CLI: one shared core.
	card := h.mustRun("", "query", "--project", "proj-mcp", "card local-build")
	if !strings.Contains(card, "local-build") {
		t.Fatalf("MCP write is not visible to the CLI:\n%s", card)
	}
}
