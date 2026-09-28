package main

import (
	"context"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"

	"connectrpc.com/connect"

	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
)

// Agent session environment, exported by `hook session-start` into Claude
// Code's CLAUDE_ENV_FILE so every `polterware apply` in the session joins one
// AgentRun per project.
const (
	envSession    = "POLTERWARE_SESSION"
	envAgentLabel = "POLTERWARE_AGENT_LABEL"
)

// hookInput is the subset of the Claude Code hook payload (stdin JSON) we read.
type hookInput struct {
	SessionID string `json:"session_id"`
	Cwd       string `json:"cwd"`
	ToolInput struct {
		FilePath string `json:"file_path"`
	} `json:"tool_input"`
}

func runHook(dbPath string, args []string) error {
	if len(args) == 0 {
		return errors.New("usage: polterware hook session-start [--label name] | session-end | lint")
	}
	input, err := readHookInput(os.Stdin)
	if err != nil {
		return err
	}
	switch args[0] {
	case "session-start":
		flags := flag.NewFlagSet("hook session-start", flag.ContinueOnError)
		flags.SetOutput(os.Stderr)
		label := flags.String("label", "claude-code", "AgentRun label for the session")
		if err := flags.Parse(args[1:]); err != nil {
			return err
		}
		return hookSessionStart(input, *label)
	case "session-end":
		return hookSessionEnd(dbPath, input.SessionID)
	case "lint":
		return hookLint(dbPath, input)
	default:
		return fmt.Errorf("unknown hook %q: use session-start, session-end, or lint", args[0])
	}
}

func readHookInput(r io.Reader) (hookInput, error) {
	var input hookInput
	body, err := io.ReadAll(r)
	if err != nil || len(strings.TrimSpace(string(body))) == 0 {
		return input, err
	}
	if err := json.Unmarshal(body, &input); err != nil {
		return input, fmt.Errorf("hook input is not JSON: %w", err)
	}
	return input, nil
}

// hookSessionStart exports the session id for later Bash commands. The run
// itself opens lazily on the first write, so read-only sessions leave no run.
func hookSessionStart(input hookInput, label string) error {
	envFile := os.Getenv("CLAUDE_ENV_FILE")
	if envFile == "" || input.SessionID == "" {
		return nil
	}
	file, err := os.OpenFile(envFile, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o600)
	if err != nil {
		return err
	}
	defer file.Close()
	_, err = fmt.Fprintf(file, "export %s=%s\nexport %s=%s\n",
		envSession, shellQuote(input.SessionID), envAgentLabel, shellQuote(label))
	return err
}

// hookSessionEnd completes the session's open runs in the core.
func hookSessionEnd(dbPath, sessionID string) error {
	if sessionID == "" {
		return nil
	}
	c, err := dial(dbPath)
	if err != nil {
		return err
	}
	_, err = c.Work.EndSession(context.Background(), connect.NewRequest(&localv1.EndSessionRequest{SessionId: sessionID}))
	return err
}

// hookLint lints a written *.pw.md file. Exit code 2 sends the diagnostics
// (errors and similarity hints) back to the agent (Claude Code PostToolUse
// contract).
func hookLint(dbPath string, input hookInput) error {
	file := input.ToolInput.FilePath
	if !strings.HasSuffix(file, ".pw.md") {
		return nil
	}
	body, err := os.ReadFile(file)
	if err != nil {
		return nil
	}
	dir := input.Cwd
	if dir == "" {
		dir = filepath.Dir(file)
	}
	c, err := dial(dbPath)
	if err != nil {
		return err
	}
	project := ""
	if resolved, err := resolveProjectIn(c, "", dir); err == nil {
		project = resolved.Id
	}
	res, err := c.Work.LintPolter(context.Background(), connect.NewRequest(&localv1.LintPolterRequest{ProjectId: project, Text: string(body)}))
	if err != nil {
		return err
	}
	diags := diagnosticsFrom(res.Msg.Diagnostics)
	if len(diags) == 0 {
		return nil
	}
	writeDiagnostics(os.Stderr, file, diags)
	return exitError{code: 2}
}

func shellQuote(value string) string {
	return "'" + strings.ReplaceAll(value, "'", `'\''`) + "'"
}
