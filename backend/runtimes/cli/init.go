package main

import (
	"context"
	"flag"
	"fmt"
	"os"
	"path/filepath"

	"connectrpc.com/connect"

	"github.com/polterware/polterware/backend/compositions/wedge"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
)

func runInit(dbPath string, args []string) error {
	fs := flag.NewFlagSet("init", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	name := fs.String("name", "", "project name")
	path := fs.String("path", "", "project root path (default: cwd)")
	id := fs.String("id", "", "optional project id")
	asJSON := fs.Bool("json", false, "print JSON")
	if err := fs.Parse(args); err != nil {
		return err
	}
	root := *path
	if root == "" {
		cwd, err := os.Getwd()
		if err != nil {
			return err
		}
		root = cwd
	}
	// The core resolves paths against its own cwd, so send an absolute root.
	root, err := filepath.Abs(wedge.ExpandRootPath(root))
	if err != nil {
		return err
	}
	c, err := dial(dbPath)
	if err != nil {
		return err
	}
	res, err := c.Projects.InitProject(context.Background(), connect.NewRequest(&localv1.InitProjectRequest{Id: *id, Name: *name, RootPath: root}))
	if err != nil {
		return err
	}
	project := res.Msg.Project
	if *asJSON {
		return writeJSON(map[string]any{
			"id":        project.Id,
			"name":      project.Name,
			"root_path": project.RootPath,
			"status":    project.Status,
		})
	}
	fmt.Printf("Initialized project %s (%s) at %s\n", project.Name, project.Id, project.RootPath)
	return nil
}
