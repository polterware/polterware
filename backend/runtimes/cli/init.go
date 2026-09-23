package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"

	"github.com/polterware/polterware/backend/compositions/wedge"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func runInit(store *storepkg.Store, args []string) error {
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
	project, err := wedge.InitProject(store, wedge.InitInput{
		ID:       *id,
		Name:     *name,
		RootPath: root,
	})
	if err != nil {
		return err
	}
	if *asJSON {
		return writeJSON(map[string]any{
			"id":        project.ID,
			"name":      project.Name,
			"root_path": project.RootPath,
			"status":    project.Status,
		})
	}
	abs, _ := filepath.Abs(project.RootPath)
	fmt.Printf("Initialized project %s (%s) at %s\n", project.Name, project.ID, abs)
	return nil
}
