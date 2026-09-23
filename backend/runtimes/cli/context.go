package main

import (
	"flag"
	"fmt"
	"os"

	"github.com/polterware/polterware/backend/compositions/wedge"
	contextpkg "github.com/polterware/polterware/backend/domains/context"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func runContext(store *storepkg.Store, args []string) error {
	fs := flag.NewFlagSet("context", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	projectID := fs.String("project", "", "project id")
	query := fs.String("query", "", "optional search query")
	entityID := fs.String("entity", "", "optional entity id")
	budget := fs.Int("budget", contextpkg.DefaultTokenBudget, "token budget")
	agentRunID := fs.String("agent-run-id", "", "optional open AgentRun id")
	asJSON := fs.Bool("json", false, "print JSON")
	if err := fs.Parse(args); err != nil {
		return err
	}
	pkg, err := wedge.CompileContext(store, wedge.ContextInput{
		ProjectID:   *projectID,
		Query:       *query,
		EntityID:    *entityID,
		TokenBudget: *budget,
		AgentRunID:  *agentRunID,
	})
	if err != nil {
		return err
	}
	if *asJSON {
		return writeJSON(pkg)
	}
	fmt.Printf("context project=%s items=%d tokens=%d truncated=%v\n",
		pkg.ProjectID, len(pkg.Items), pkg.TokensUsed, pkg.Truncated)
	for _, item := range pkg.Items {
		fmt.Printf("- [%s] %s (%s)\n", item.Kind, item.Title, item.ID)
	}
	return nil
}
