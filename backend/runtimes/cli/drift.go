package main

import (
	"flag"
	"fmt"
	"os"

	"github.com/polterware/polterware/backend/compositions/wedge"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func runDrift(store *storepkg.Store, args []string) error {
	fs := flag.NewFlagSet("drift", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	projectID := fs.String("project", "", "project id")
	entityID := fs.String("entity", "", "optional entity id")
	agentRunID := fs.String("agent-run-id", "", "optional open AgentRun id")
	asJSON := fs.Bool("json", false, "print JSON")
	if err := fs.Parse(args); err != nil {
		return err
	}
	result, err := wedge.CheckDrift(store, wedge.DriftInput{
		ProjectID:  *projectID,
		EntityID:   *entityID,
		AgentRunID: *agentRunID,
	})
	if err != nil {
		return err
	}
	if *asJSON {
		return writeJSON(result)
	}
	fmt.Printf("drift project=%s checker=%s findings=%d status=%s\n",
		result.ProjectID, result.Checker, len(result.Findings), result.Status)
	for _, finding := range result.Findings {
		fmt.Printf("- [%s/%s] %s\n", finding.Severity, finding.Type, finding.Summary)
	}
	return nil
}
