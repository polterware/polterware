package main

import (
	"context"
	"flag"
	"fmt"
	"io"
	"strings"
	"time"

	"connectrpc.com/connect"
	"github.com/polterware/polterware/sparky/backend/platform/coreclient"
	v1 "github.com/polterware/polterware/sparky/backend/platform/gen/sparky/v1"
	"google.golang.org/protobuf/types/known/timestamppb"
)

func runAdd(ctx context.Context, dbPath string, args []string, out, stderr io.Writer) error {
	fs := flag.NewFlagSet("add", flag.ContinueOnError)
	fs.SetOutput(stderr)
	mindRef := fs.String("mind", "", "Mind name or id")
	dueRaw := fs.String("due", "", "RFC 3339, YYYY-MM-DD, today, tomorrow, or +2h")
	repeatRaw := fs.String("repeat", "", "daily|weekly|monthly|yearly or every N <unit>")
	pos, err := parseCommand(fs, args)
	if err != nil {
		return mapRPC(err)
	}
	title := strings.TrimSpace(strings.Join(pos, " "))
	if title == "" {
		return usagef("usage: sparky add \"title\" [--mind X --due … --repeat …]")
	}
	clients, err := ensureClients(ctx, dbPath)
	if err != nil {
		return err
	}
	req := &v1.CreateMemoryRequest{Title: title}
	if *mindRef != "" {
		id, err := lookupMindID(ctx, clients, *mindRef)
		if err != nil {
			return err
		}
		req.MindId = id
	}
	var due parsedTime
	if *dueRaw != "" {
		due, err = parseWhen(*dueRaw, time.Now(), cliLocation())
		if err != nil {
			return err
		}
		req.DueAt = timestamppb.New(due.At.UTC())
		req.DueAllDay = due.AllDay
	}
	created, err := clients.Memory.Create(ctx, connect.NewRequest(req))
	if err != nil {
		return mapRPC(err)
	}
	mem := created.Msg.Memory
	if *repeatRaw != "" {
		rep, err := parseRepeat(*repeatRaw)
		if err != nil {
			return err
		}
		fire := time.Now().In(cliLocation())
		allDay := false
		tz := zoneName(cliLocation())
		if !due.At.IsZero() {
			fire = due.At
			allDay = due.AllDay
			if due.TZ != "" {
				tz = due.TZ
			}
		}
		active := true
		_, err = clients.Schedule.Set(ctx, connect.NewRequest(&v1.SetScheduleRequest{Schedule: &v1.Schedule{
			MemoryId: mem.Id,
			Active:   &active,
			FireAt:   timestamppb.New(fire.UTC()),
			Tz:       tz,
			AllDay:   allDay,
			Freq:     rep.Freq,
			Every:    int32ptr(rep.Every),
			EndType:  "never",
		}}))
		if err != nil {
			return mapRPC(err)
		}
	}
	fmt.Fprintf(out, "created\t%s\t%s\n", mem.Id, mem.Title)
	return nil
}

func runList(ctx context.Context, dbPath string, args []string, out, stderr io.Writer) error {
	fs := flag.NewFlagSet("list", flag.ContinueOnError)
	fs.SetOutput(stderr)
	jsonOutput := fs.Bool("json", false, "print JSON")
	status := fs.String("status", "", "active|completed")
	mindRef := fs.String("mind", "", "Mind name or id")
	inbox := fs.Bool("inbox", false, "inbox only")
	limit := fs.Int("limit", 0, "page size")
	pos, err := parseCommand(fs, args)
	if err != nil {
		return mapRPC(err)
	}
	if len(pos) != 0 {
		return usagef("unexpected list arguments")
	}
	clients, err := ensureClients(ctx, dbPath)
	if err != nil {
		return err
	}
	req := &v1.ListMemoriesRequest{Status: *status, InboxOnly: *inbox, Limit: int32(*limit)}
	if *mindRef != "" {
		id, err := lookupMindID(ctx, clients, *mindRef)
		if err != nil {
			return err
		}
		req.MindId = id
	}
	page, err := clients.Memory.List(ctx, connect.NewRequest(req))
	if err != nil {
		return mapRPC(err)
	}
	if *jsonOutput {
		return writeJSON(out, page.Msg)
	}
	if len(page.Msg.Memories) == 0 {
		fmt.Fprintln(out, "0 memories")
		return nil
	}
	for _, m := range page.Msg.Memories {
		fmt.Fprintf(out, "%s\t%s\t%s\t%s\n", m.Id, m.Status, formatStamp(m.DueAt), m.Title)
	}
	return nil
}

func runShow(ctx context.Context, dbPath string, args []string, out, stderr io.Writer) error {
	fs := flag.NewFlagSet("show", flag.ContinueOnError)
	fs.SetOutput(stderr)
	jsonOutput := fs.Bool("json", false, "print JSON")
	pos, err := parseCommand(fs, args)
	if err != nil {
		return mapRPC(err)
	}
	if len(pos) != 1 {
		return usagef("usage: sparky show <id> [--json]")
	}
	clients, err := ensureClients(ctx, dbPath)
	if err != nil {
		return err
	}
	include := &v1.MemoryInclude{Note: true, Checklist: true, Schedule: true, Tags: true, Place: true, Completions: true, Attachments: true}
	mem, err := getMemory(ctx, clients, pos[0], include)
	if err != nil {
		return err
	}
	if *jsonOutput {
		return writeJSON(out, mem)
	}
	fmt.Fprintf(out, "id\t%s\n", mem.Id)
	fmt.Fprintf(out, "title\t%s\n", mem.Title)
	fmt.Fprintf(out, "status\t%s\n", mem.Status)
	fmt.Fprintf(out, "mind\t%s\n", dash(mem.MindId))
	fmt.Fprintf(out, "due\t%s\n", formatStamp(mem.DueAt))
	fmt.Fprintf(out, "pinned\t%t\n", mem.Pinned)
	fmt.Fprintf(out, "rev\t%d\n", mem.Rev)
	if mem.Note != "" {
		fmt.Fprintf(out, "note\t%s\n", strings.ReplaceAll(mem.Note, "\n", "\\n"))
	}
	if mem.Schedule != nil && mem.Schedule.Freq != "" {
		fmt.Fprintf(out, "repeat\t%s every %d\n", mem.Schedule.Freq, mem.Schedule.GetEvery())
	}
	return nil
}

func runDone(ctx context.Context, dbPath string, args []string, out, stderr io.Writer) error {
	fs := flag.NewFlagSet("done", flag.ContinueOnError)
	fs.SetOutput(stderr)
	pos, err := parseCommand(fs, args)
	if err != nil {
		return mapRPC(err)
	}
	if len(pos) != 1 {
		return usagef("usage: sparky done <id>")
	}
	clients, err := ensureClients(ctx, dbPath)
	if err != nil {
		return err
	}
	return completeMemory(ctx, clients, pos[0], out)
}

func completeMemory(ctx context.Context, clients coreclient.Clients, id string, out io.Writer) error {
	mem, err := getMemory(ctx, clients, id, nil)
	if err != nil {
		return err
	}
	return setMemoryStatus(ctx, clients, mem.Id, "completed", mem.Rev, out)
}

func setMemoryStatus(ctx context.Context, clients coreclient.Clients, id, status string, rev int64, out io.Writer) error {
	got, err := clients.Memory.SetStatus(ctx, connect.NewRequest(&v1.SetMemoryStatusRequest{
		Id: id, Status: status, ExpectedRev: int64ptr(rev),
	}))
	if err != nil {
		return mapRPC(err)
	}
	fmt.Fprintf(out, "completed\t%s\t%s\n", got.Msg.Memory.Id, got.Msg.Memory.Title)
	return nil
}

func runRm(ctx context.Context, dbPath string, args []string, out, stderr io.Writer) error {
	fs := flag.NewFlagSet("rm", flag.ContinueOnError)
	fs.SetOutput(stderr)
	yes := fs.Bool("yes", false, "confirm deletion")
	pos, err := parseCommand(fs, args)
	if err != nil {
		return mapRPC(err)
	}
	if len(pos) != 1 {
		return usagef("usage: sparky rm <id> --yes")
	}
	if !*yes {
		return refuseYes("delete memory " + pos[0])
	}
	clients, err := ensureClients(ctx, dbPath)
	if err != nil {
		return err
	}
	return deleteMemory(ctx, clients, pos[0], out)
}

func deleteMemory(ctx context.Context, clients coreclient.Clients, id string, out io.Writer) error {
	mem, err := getMemory(ctx, clients, id, nil)
	if err != nil {
		return err
	}
	_, err = clients.Memory.Delete(ctx, connect.NewRequest(&v1.DeleteMemoryRequest{Id: id, ExpectedRev: int64ptr(mem.Rev)}))
	if err != nil {
		return mapRPC(err)
	}
	fmt.Fprintf(out, "deleted\t%s\t%s\n", mem.Id, mem.Title)
	return nil
}
