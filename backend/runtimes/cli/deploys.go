package main

import (
	"context"
	"errors"
	"flag"
	"fmt"
	"os"
	"strings"

	"connectrpc.com/connect"

	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
)

const deploysUsage = `usage: polterware deploys [<path>]                  list deploy links with their latest check
       polterware deploys check [<path>] [--force]    ask each platform's CLI for the latest deploy
       polterware deploys targets <provider> [--node <path>]   list what a deploy can link to`

// runDeploys reads and checks deploy links. A link names the platform's
// target as env=provider:resource[@id]; the core finds the id and the dashboard
// page, and checks the latest deploy through the platform's own CLI.
func runDeploys(dbPath string, args []string) error {
	flags := flag.NewFlagSet("deploys", flag.ContinueOnError)
	flags.SetOutput(os.Stderr)
	projectID := flags.String("project", "", "project id (default: project containing the cwd)")
	force := flags.Bool("force", false, "check links checked in the last minute too")
	node := flags.String("node", "", "Structure path whose checkout is read for targets")
	positional, err := parseInterspersed(flags, args)
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
	ctx := context.Background()
	sub := ""
	if len(positional) > 0 {
		sub = positional[0]
	}
	switch sub {
	case "check":
		path := strings.Join(positional[1:], "")
		res, err := c.Integrations.CheckDeploys(ctx, connect.NewRequest(&localv1.CheckDeploysRequest{ProjectId: project.Id, NodeId: path, Force: *force}))
		if err != nil {
			return err
		}
		fmt.Printf("checked %d\n", res.Msg.Checked)
		return printDeploys(c.Work, project.Id, path)
	case "targets":
		if len(positional) != 2 {
			return errors.New(deploysUsage)
		}
		res, err := c.Integrations.ListDeployTargets(ctx, connect.NewRequest(&localv1.ListDeployTargetsRequest{ProjectId: project.Id, Provider: positional[1], NodeId: *node}))
		if err != nil {
			return err
		}
		for _, target := range res.Msg.Targets {
			mark := " "
			if target.Detected {
				mark = "*"
			}
			fmt.Printf("%s %s@%s  %s  %s\n", mark, target.Name, orDash(target.Id), target.Kind, orDash(target.Url))
		}
		if res.Msg.Message != "" {
			fmt.Fprintln(os.Stderr, res.Msg.Message)
		}
		return nil
	case "help", "-h", "--help":
		fmt.Println(deploysUsage)
		return nil
	default:
		if len(positional) > 1 {
			return errors.New(deploysUsage)
		}
		return printDeploys(c.Work, project.Id, strings.Join(positional, ""))
	}
}

func printDeploys(work interface {
	QueryPolter(context.Context, *connect.Request[localv1.QueryPolterRequest]) (*connect.Response[localv1.QueryPolterResponse], error)
}, projectID, path string) error {
	res, err := work.QueryPolter(context.Background(), connect.NewRequest(&localv1.QueryPolterRequest{ProjectId: projectID, Query: strings.TrimSpace("deploys " + path)}))
	if err != nil {
		return err
	}
	fmt.Print(res.Msg.Text)
	return nil
}
