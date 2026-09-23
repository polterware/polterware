package main

import (
	"flag"
	"fmt"
	"os"

	"github.com/polterware/polterware/backend/compositions/wedge"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func runScan(store *storepkg.Store, args []string) error {
	fs := flag.NewFlagSet("scan", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	projectID := fs.String("project", "", "project id")
	asJSON := fs.Bool("json", false, "print JSON")
	if err := fs.Parse(args); err != nil {
		return err
	}
	result, err := wedge.ScanProject(store, *projectID)
	if err != nil {
		return err
	}
	if *asJSON {
		return writeJSON(result)
	}
	fmt.Printf("%s\n", result.Message)
	fmt.Printf("facts=%d evidence=%d sources=%d skipped=%d\n",
		result.FactCount, result.EvidenceCount, result.SourceCount, result.SkippedCount)
	return nil
}
