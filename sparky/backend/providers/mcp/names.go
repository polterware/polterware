package mcp

import (
	"context"
	"fmt"
	"sort"
	"strings"

	"github.com/polterware/polterware/sparky/backend/domains/activity"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/tag"
)

func resolveMind(ctx context.Context, minds mind.Service, ref string) (mind.Mind, error) {
	ref = strings.TrimSpace(ref)
	if ref == "" {
		return mind.Mind{}, fault.Invalid(fault.Violation{Field: "mind", Reason: "required"})
	}
	if got, err := minds.Get(ctx, ref); err == nil {
		return got, nil
	} else if !fault.IsCode(err, fault.CodeNotFound) {
		return mind.Mind{}, err
	}
	rows, err := minds.List(ctx)
	if err != nil {
		return mind.Mind{}, err
	}
	names := make(map[string]string, len(rows))
	valid := make([]string, 0, len(rows))
	for _, row := range rows {
		names[row.ID] = row.Name
		valid = append(valid, row.Name)
	}
	id, err := activity.Lookup(ref, names)
	if err != nil {
		sort.Strings(valid)
		return mind.Mind{}, &fault.Error{Code: fault.CodeNotFound, Entity: "mind", Message: fmt.Sprintf("Mind %q not found. Valid: %s", ref, strings.Join(valid, ", "))}
	}
	return minds.Get(ctx, id)
}

func resolveTag(ctx context.Context, tags tag.Service, ref string) (tag.Tag, error) {
	ref = strings.TrimSpace(ref)
	if ref == "" {
		return tag.Tag{}, fault.Invalid(fault.Violation{Field: "tag", Reason: "required"})
	}
	rows, err := tags.List(ctx)
	if err != nil {
		return tag.Tag{}, err
	}
	names := make(map[string]string, len(rows))
	valid := make([]string, 0, len(rows))
	for _, row := range rows {
		if row.ID == ref {
			return row, nil
		}
		names[row.ID] = row.Name
		valid = append(valid, row.Name)
	}
	id, err := activity.Lookup(ref, names)
	if err != nil {
		sort.Strings(valid)
		return tag.Tag{}, &fault.Error{Code: fault.CodeNotFound, Entity: "tag", Message: fmt.Sprintf("Tag %q not found. Valid: %s", ref, strings.Join(valid, ", "))}
	}
	for _, row := range rows {
		if row.ID == id {
			return row, nil
		}
	}
	return tag.Tag{}, fault.NotFound("tag", ref)
}

func resolveTags(ctx context.Context, tags tag.Service, refs []string) ([]string, error) {
	ids := make([]string, 0, len(refs))
	for _, ref := range refs {
		got, err := resolveTag(ctx, tags, ref)
		if err != nil {
			return nil, err
		}
		ids = append(ids, got.ID)
	}
	return ids, nil
}

func optionalMind(ctx context.Context, minds mind.Service, ref string) (*string, error) {
	if strings.TrimSpace(ref) == "" {
		return nil, nil
	}
	got, err := resolveMind(ctx, minds, ref)
	if err != nil {
		return nil, err
	}
	id := got.ID
	return &id, nil
}
