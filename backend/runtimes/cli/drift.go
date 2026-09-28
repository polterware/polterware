package main

import (
	"context"
	"flag"
	"fmt"
	"os"

	"connectrpc.com/connect"

	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
)

func runDrift(dbPath string, args []string) error {
	fs := flag.NewFlagSet("drift", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	projectID := fs.String("project", "", "project id")
	entityID := fs.String("entity", "", "optional entity id")
	agentRunID := fs.String("agent-run-id", "", "optional open AgentRun id")
	asJSON := fs.Bool("json", false, "print JSON")
	if err := fs.Parse(args); err != nil {
		return err
	}
	c, err := dial(dbPath)
	if err != nil {
		return err
	}
	res, err := c.Context.CheckDrift(context.Background(), connect.NewRequest(&localv1.CheckDriftRequest{
		ProjectId:  *projectID,
		EntityId:   *entityID,
		AgentRunId: *agentRunID,
	}))
	if err != nil {
		return err
	}
	result := res.Msg
	if *asJSON {
		return writeProtoJSON(result)
	}
	fmt.Printf("drift project=%s checker=%s findings=%d status=%s\n",
		result.ProjectId, result.Checker, len(result.Findings), result.Status)
	for _, finding := range result.Findings {
		fmt.Printf("- [%s/%s] %s\n", finding.Severity, finding.Type, finding.Summary)
	}
	return nil
}
