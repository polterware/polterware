package main

import (
	"context"
	"flag"
	"fmt"
	"os"

	"connectrpc.com/connect"

	contextpkg "github.com/polterware/polterware/backend/domains/context"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
)

func runContext(dbPath string, args []string) error {
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
	c, err := dial(dbPath)
	if err != nil {
		return err
	}
	res, err := c.Context.CompileContext(context.Background(), connect.NewRequest(&localv1.CompileContextRequest{
		ProjectId:   *projectID,
		Query:       *query,
		EntityId:    *entityID,
		TokenBudget: int32(*budget),
		AgentRunId:  *agentRunID,
	}))
	if err != nil {
		return err
	}
	pkg := res.Msg
	if *asJSON {
		return writeProtoJSON(pkg)
	}
	fmt.Printf("context project=%s items=%d tokens=%d truncated=%v\n",
		pkg.ProjectId, len(pkg.Items), pkg.TokensUsed, pkg.Truncated)
	for _, item := range pkg.Items {
		fmt.Printf("- [%s] %s (%s)\n", item.Kind, item.Title, item.Id)
	}
	return nil
}
