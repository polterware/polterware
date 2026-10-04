package main

import (
	"context"
	"flag"
	"fmt"
	"io"
	"strings"

	"connectrpc.com/connect"
	v1 "github.com/polterware/polterware/sparky/backend/platform/gen/sparky/v1"
)

func runMind(ctx context.Context, dbPath string, args []string, out, stderr io.Writer) error {
	if len(args) == 0 {
		return usagef("usage: sparky mind add|ls|rm")
	}
	switch args[0] {
	case "add":
		return runMindAdd(ctx, dbPath, args[1:], out, stderr)
	case "ls":
		return runMindLS(ctx, dbPath, args[1:], out, stderr)
	case "rm":
		return runMindRm(ctx, dbPath, args[1:], out, stderr)
	default:
		return usagef("usage: sparky mind add|ls|rm")
	}
}

func runMindAdd(ctx context.Context, dbPath string, args []string, out, stderr io.Writer) error {
	fs := flag.NewFlagSet("mind add", flag.ContinueOnError)
	fs.SetOutput(stderr)
	parent := fs.String("parent", "", "parent Mind name or id")
	pos, err := parseCommand(fs, args)
	if err != nil {
		return mapRPC(err)
	}
	name := strings.TrimSpace(strings.Join(pos, " "))
	if name == "" {
		return usagef("usage: sparky mind add \"name\" [--parent X]")
	}
	clients, err := ensureClients(ctx, dbPath)
	if err != nil {
		return err
	}
	req := &v1.CreateMindRequest{Name: name}
	if *parent != "" {
		id, err := lookupMindID(ctx, clients, *parent)
		if err != nil {
			return err
		}
		req.ParentId = id
	}
	created, err := clients.Mind.Create(ctx, connect.NewRequest(req))
	if err != nil {
		return mapRPC(err)
	}
	fmt.Fprintf(out, "created\t%s\t%s\n", created.Msg.Mind.Id, created.Msg.Mind.Name)
	return nil
}

func runMindLS(ctx context.Context, dbPath string, args []string, out, stderr io.Writer) error {
	fs := flag.NewFlagSet("mind ls", flag.ContinueOnError)
	fs.SetOutput(stderr)
	jsonOutput := fs.Bool("json", false, "print JSON")
	pos, err := parseCommand(fs, args)
	if err != nil {
		return mapRPC(err)
	}
	if len(pos) != 0 {
		return usagef("unexpected mind ls arguments")
	}
	clients, err := ensureClients(ctx, dbPath)
	if err != nil {
		return err
	}
	page, err := clients.Mind.List(ctx, connect.NewRequest(&v1.ListMindsRequest{}))
	if err != nil {
		return mapRPC(err)
	}
	if *jsonOutput {
		return writeJSON(out, page.Msg)
	}
	if len(page.Msg.Minds) == 0 {
		fmt.Fprintln(out, "0 minds")
		return nil
	}
	for _, m := range page.Msg.Minds {
		fmt.Fprintf(out, "%s\t%s\t%s\n", m.Id, m.Name, dash(m.ParentId))
	}
	return nil
}

func runMindRm(ctx context.Context, dbPath string, args []string, out, stderr io.Writer) error {
	fs := flag.NewFlagSet("mind rm", flag.ContinueOnError)
	fs.SetOutput(stderr)
	yes := fs.Bool("yes", false, "confirm deletion")
	pos, err := parseCommand(fs, args)
	if err != nil {
		return mapRPC(err)
	}
	if len(pos) != 1 {
		return usagef("usage: sparky mind rm <id|name> --yes")
	}
	ref := pos[0]
	if !*yes {
		return refuseYes("delete mind " + ref)
	}
	clients, err := ensureClients(ctx, dbPath)
	if err != nil {
		return err
	}
	mind, err := getMind(ctx, clients, ref)
	if err != nil {
		return err
	}
	res, err := clients.Mind.Delete(ctx, connect.NewRequest(&v1.DeleteMindRequest{Id: mind.Id, ExpectedRev: int64ptr(mind.Rev)}))
	if err != nil {
		return mapRPC(err)
	}
	fmt.Fprintf(out, "deleted\t%s\t%s\tdeletedMinds=%d\tmovedMemories=%d\n", mind.Id, mind.Name, len(res.Msg.DeletedMindIds), res.Msg.MovedMemoryCount)
	return nil
}
