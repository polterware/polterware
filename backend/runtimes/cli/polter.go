package main

import (
	"context"
	"errors"
	"flag"
	"fmt"
	"io"
	"io/fs"
	"os"
	"path/filepath"
	"strings"

	"connectrpc.com/connect"

	"github.com/polterware/polterware/backend/domains/polter"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
)

func runApply(dbPath string, args []string) error {
	flags := flag.NewFlagSet("apply", flag.ContinueOnError)
	flags.SetOutput(os.Stderr)
	dryRun := flags.Bool("dry-run", false, "print the plan without writing")
	projectID := flags.String("project", "", "project id (default: project containing the cwd)")
	summary := flags.String("summary", "", "change summary (overrides frontmatter)")
	rationale := flags.String("rationale", "", "change rationale (overrides frontmatter)")
	agentRunID := flags.String("agent-run-id", "", "optional open AgentRun id")
	positional, err := parseInterspersed(flags, args)
	if err != nil {
		return err
	}
	if len(positional) != 1 {
		return errors.New("usage: polterware apply <file|-> [--dry-run]")
	}
	name, text, err := readPolterInput(positional[0])
	if err != nil {
		return err
	}
	c, err := dial(dbPath)
	if err != nil {
		return err
	}
	project, err := resolveProject(c, *projectID)
	if err != nil {
		return err
	}
	request := &localv1.ApplyPolterRequest{
		ProjectId: project.Id,
		Text:      text,
		DryRun:    *dryRun,
		Change:    &localv1.ChangeEnvelope{Summary: *summary, Rationale: *rationale, AgentRunId: *agentRunID},
	}
	// Inside an agent session (hook session-start) writes join the session's run.
	if sessionID := os.Getenv(envSession); sessionID != "" && *agentRunID == "" {
		request.SessionId = sessionID
		request.SessionLabel = os.Getenv(envAgentLabel)
	}
	res, err := c.Work.ApplyPolter(context.Background(), connect.NewRequest(request))
	if diags, ok := polterDiagnostics(err); ok {
		printDiagnostics(name, diags)
		return exitError{code: 1}
	}
	if err != nil {
		return err
	}
	printDiagnostics(name, diagnosticsFrom(res.Msg.Diagnostics))
	switch {
	case len(res.Msg.Ops) == 0:
		fmt.Println("no changes")
	case *dryRun:
		fmt.Println(strings.Join(res.Msg.Ops, "\n"))
	default:
		fmt.Printf("applied %s\n", res.Msg.Summary)
	}
	return nil
}

func runLint(dbPath string, args []string) error {
	flags := flag.NewFlagSet("lint", flag.ContinueOnError)
	flags.SetOutput(os.Stderr)
	projectID := flags.String("project", "", "project id (default: project containing the cwd)")
	positional, err := parseInterspersed(flags, args)
	if err != nil {
		return err
	}
	if len(positional) != 1 {
		return errors.New("usage: polterware lint <file|dir|->")
	}
	files, err := lintTargets(positional[0])
	if err != nil {
		return err
	}
	c, err := dial(dbPath)
	if err != nil {
		return err
	}
	// Reference checks and hints run only when the project resolves; otherwise syntax only.
	project := ""
	if resolved, err := resolveProject(c, *projectID); err == nil {
		project = resolved.Id
	} else if *projectID != "" {
		return err
	}
	failed := false
	for _, file := range files {
		name, text, err := readPolterInput(file)
		if err != nil {
			return err
		}
		res, err := c.Work.LintPolter(context.Background(), connect.NewRequest(&localv1.LintPolterRequest{ProjectId: project, Text: text}))
		if err != nil {
			return err
		}
		diags := diagnosticsFrom(res.Msg.Diagnostics)
		printDiagnostics(name, diags)
		failed = failed || polter.HasErrors(diags)
	}
	if failed {
		return exitError{code: 1}
	}
	return nil
}

func runFormat(args []string) error {
	if len(args) > 0 {
		return errors.New("usage: polterware format")
	}
	fmt.Print(polter.Spec)
	return nil
}

func runQuery(dbPath string, args []string) error {
	flags := flag.NewFlagSet("query", flag.ContinueOnError)
	flags.SetOutput(os.Stderr)
	projectID := flags.String("project", "", "project id (default: project containing the cwd)")
	positional, err := parseInterspersed(flags, args)
	if err != nil {
		return err
	}
	if len(positional) == 0 {
		return errors.New(`usage: polterware query "cards" | "card <slug>"`)
	}
	c, err := dial(dbPath)
	if err != nil {
		return err
	}
	project, err := resolveProject(c, *projectID)
	if err != nil {
		return err
	}
	res, err := c.Work.QueryPolter(context.Background(), connect.NewRequest(&localv1.QueryPolterRequest{ProjectId: project.Id, Query: strings.Join(positional, " ")}))
	if err != nil {
		return err
	}
	fmt.Print(res.Msg.Text)
	return nil
}

// parseInterspersed allows flags after positional arguments
// (`polterware apply roadmap.pw.md --dry-run`).
func parseInterspersed(flags *flag.FlagSet, args []string) ([]string, error) {
	var positional []string
	for {
		if err := flags.Parse(args); err != nil {
			return nil, err
		}
		rest := flags.Args()
		if len(rest) == 0 {
			return positional, nil
		}
		positional = append(positional, rest[0])
		args = rest[1:]
	}
}

func readPolterInput(arg string) (string, string, error) {
	if arg == "-" {
		body, err := io.ReadAll(os.Stdin)
		return "<stdin>", string(body), err
	}
	body, err := os.ReadFile(arg)
	return arg, string(body), err
}

// lintTargets expands a directory to its *.pw.md files.
func lintTargets(arg string) ([]string, error) {
	if arg == "-" {
		return []string{arg}, nil
	}
	info, err := os.Stat(arg)
	if err != nil {
		return nil, err
	}
	if !info.IsDir() {
		return []string{arg}, nil
	}
	var files []string
	err = filepath.WalkDir(arg, func(path string, entry fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if entry.IsDir() && (entry.Name() == "node_modules" || entry.Name() == ".git") {
			return filepath.SkipDir
		}
		if !entry.IsDir() && strings.HasSuffix(entry.Name(), ".pw.md") {
			files = append(files, path)
		}
		return nil
	})
	return files, err
}

func printDiagnostics(name string, diags []polter.Diagnostic) {
	writeDiagnostics(os.Stdout, name, diags)
}

// writeDiagnostics prints one line per diagnostic; hints are marked so they
// read as advice, not failures.
func writeDiagnostics(w io.Writer, name string, diags []polter.Diagnostic) {
	for _, d := range diags {
		message := d.Message
		if d.Severity == polter.SeverityHint {
			message = "hint: " + message
		}
		if d.Line > 0 {
			fmt.Fprintf(w, "%s:%d %s\n", name, d.Line, message)
		} else {
			fmt.Fprintf(w, "%s %s\n", name, message)
		}
	}
}
