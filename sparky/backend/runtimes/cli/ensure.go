package main

import (
	"context"
	"strings"

	"connectrpc.com/connect"
	"github.com/polterware/polterware/sparky/backend/platform/coreclient"
	v1 "github.com/polterware/polterware/sparky/backend/platform/gen/sparky/v1"
)

func ensureClients(ctx context.Context, dbPath string) (coreclient.Clients, error) {
	state, err := coreclient.Ensure(ctx, dbPath)
	if err != nil {
		return coreclient.Clients{}, err
	}
	return coreclient.New(state, "cli")
}

func lookupMindID(ctx context.Context, clients coreclient.Clients, ref string) (string, error) {
	ref = strings.TrimSpace(ref)
	if ref == "" {
		return "", nil
	}
	got, err := clients.Mind.Get(ctx, connect.NewRequest(&v1.GetMindRequest{Id: ref}))
	if err == nil {
		return got.Msg.Mind.Id, nil
	}
	if connect.CodeOf(err) != connect.CodeNotFound {
		return "", mapRPC(err)
	}
	page, err := clients.Mind.List(ctx, connect.NewRequest(&v1.ListMindsRequest{}))
	if err != nil {
		return "", mapRPC(err)
	}
	var matches []string
	valid := make([]string, 0, len(page.Msg.Minds))
	for _, m := range page.Msg.Minds {
		valid = append(valid, m.Name)
		if strings.EqualFold(m.Name, ref) {
			matches = append(matches, m.Id)
		}
	}
	if len(matches) == 1 {
		return matches[0], nil
	}
	if len(matches) > 1 {
		return "", exitf(ExitError, "mind %q is ambiguous", ref)
	}
	return "", exitf(ExitNotFound, "Mind %q not found. Valid: %s", ref, strings.Join(valid, ", "))
}

func lookupTagID(ctx context.Context, clients coreclient.Clients, ref string) (string, error) {
	ref = strings.TrimSpace(ref)
	if ref == "" {
		return "", usagef("empty tag")
	}
	page, err := clients.Tag.List(ctx, connect.NewRequest(&v1.ListTagsRequest{}))
	if err != nil {
		return "", mapRPC(err)
	}
	valid := make([]string, 0, len(page.Msg.Tags))
	for _, t := range page.Msg.Tags {
		valid = append(valid, t.Name)
		if t.Id == ref || strings.EqualFold(t.Name, ref) {
			return t.Id, nil
		}
	}
	return "", exitf(ExitNotFound, "Tag %q not found. Valid: %s", ref, strings.Join(valid, ", "))
}

func getMemory(ctx context.Context, clients coreclient.Clients, id string, include *v1.MemoryInclude) (*v1.Memory, error) {
	got, err := clients.Memory.Get(ctx, connect.NewRequest(&v1.GetMemoryRequest{Id: id, Include: include}))
	if err != nil {
		return nil, mapRPC(err)
	}
	return got.Msg.Memory, nil
}

func getMind(ctx context.Context, clients coreclient.Clients, ref string) (*v1.Mind, error) {
	id, err := lookupMindID(ctx, clients, ref)
	if err != nil {
		return nil, err
	}
	got, err := clients.Mind.Get(ctx, connect.NewRequest(&v1.GetMindRequest{Id: id}))
	if err != nil {
		return nil, mapRPC(err)
	}
	return got.Msg.Mind, nil
}

func int64ptr(n int64) *int64 { return &n }

func int32ptr(n int32) *int32 { return &n }
