package main

import (
	"context"
	"flag"
	"fmt"
	"os"

	"connectrpc.com/connect"

	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
)

func runRepo(dbPath string, args []string) error {
	if len(args) == 0 {
		return fmt.Errorf("usage: polterware repo list|add|remove")
	}
	switch args[0] {
	case "list":
		return runRepoList(dbPath, args[1:])
	case "add":
		return runRepoAdd(dbPath, args[1:])
	case "remove":
		return runRepoRemove(dbPath, args[1:])
	default:
		return fmt.Errorf("unknown repo command %q", args[0])
	}
}

func runRepoList(dbPath string, args []string) error {
	fs := flag.NewFlagSet("repo list", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	projectID := fs.String("project", "", "project id")
	asJSON := fs.Bool("json", false, "print JSON")
	if err := fs.Parse(args); err != nil {
		return err
	}
	if *projectID == "" {
		return fmt.Errorf("project id is required")
	}
	c, err := dial(dbPath)
	if err != nil {
		return err
	}
	res, err := c.Projects.ListProjectRepos(context.Background(), connect.NewRequest(&localv1.ListProjectReposRequest{ProjectId: *projectID}))
	if err != nil {
		return err
	}
	if *asJSON {
		return writeProtoJSON(res.Msg)
	}
	if len(res.Msg.Repos) == 0 {
		fmt.Println("no extra repositories")
		return nil
	}
	for _, repo := range res.Msg.Repos {
		fmt.Printf("%s  %s\n", repo.Name, repo.Path)
	}
	return nil
}

func runRepoAdd(dbPath string, args []string) error {
	fs := flag.NewFlagSet("repo add", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	projectID := fs.String("project", "", "project id")
	name := fs.String("name", "", "short lowercase name")
	path := fs.String("path", "", "absolute directory")
	asJSON := fs.Bool("json", false, "print JSON")
	if err := fs.Parse(args); err != nil {
		return err
	}
	c, err := dial(dbPath)
	if err != nil {
		return err
	}
	res, err := c.Projects.AddProjectRepo(context.Background(), connect.NewRequest(&localv1.AddProjectRepoRequest{
		ProjectId: *projectID,
		Name:      *name,
		Path:      *path,
	}))
	if err != nil {
		return err
	}
	repo := res.Msg.Repo
	if *asJSON {
		return writeJSON(map[string]any{"id": repo.Id, "name": repo.Name, "path": repo.Path})
	}
	fmt.Printf("Added %s at %s\n", repo.Name, repo.Path)
	return nil
}

func runRepoRemove(dbPath string, args []string) error {
	fs := flag.NewFlagSet("repo remove", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	projectID := fs.String("project", "", "project id")
	name := fs.String("name", "", "repository name")
	if err := fs.Parse(args); err != nil {
		return err
	}
	c, err := dial(dbPath)
	if err != nil {
		return err
	}
	_, err = c.Projects.RemoveProjectRepo(context.Background(), connect.NewRequest(&localv1.RemoveProjectRepoRequest{
		ProjectId: *projectID,
		Name:      *name,
	}))
	if err != nil {
		return err
	}
	fmt.Printf("Removed %s\n", *name)
	return nil
}
