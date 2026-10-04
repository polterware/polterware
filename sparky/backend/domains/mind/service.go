package mind

import (
	"context"
	"github.com/polterware/polterware/sparky/backend/domains/activity"
	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/fault"

	"sort"
	"strings"
)

// MemoryCounter is the narrow memory port needed by recursive deletion.
type MemoryCounter interface {
	CountByMinds(context.Context, []string) (int, error)
}

type service struct {
	repo     Repository
	memories MemoryCounter
	audit    activity.Audit
}

func NewService(r Repository, m MemoryCounter, a activity.Audit) Service {
	return &service{r, m, a}
}
func (s *service) List(ctx context.Context) ([]Mind, error)         { return s.repo.List(ctx) }
func (s *service) Get(ctx context.Context, id string) (Mind, error) { return s.repo.Get(ctx, id) }
func (s *service) Lookup(ctx context.Context, name string) (Mind, error) {
	rows, err := s.repo.List(ctx)
	if err != nil {
		return Mind{}, err
	}
	names := map[string]string{}
	for _, v := range rows {
		names[v.ID] = v.Name
	}
	id, err := activity.Lookup(name, names)
	if err != nil {
		return Mind{}, err
	}
	return s.repo.Get(ctx, id)
}

// ResolveName is used by MCP without changing the frozen service interface.
func ResolveName(ctx context.Context, r Repository, name string) (Mind, error) {
	s := &service{repo: r}
	return s.Lookup(ctx, name)
}
func (s *service) validate(ctx context.Context, m Mind) error {
	if err := activity.ValidateName("name", m.Name, 120); err != nil {
		return err
	}
	if err := activity.ValidateColor(m.Color); err != nil {
		return err
	}
	seen := map[string]bool{m.ID: true}
	parent := m.ParentID
	for parent != nil {
		if seen[*parent] {
			return &fault.Error{Code: fault.CodeCycle, Message: "mind has cycle", Entity: "mind", EntityID: m.ID}
		}
		seen[*parent] = true
		n, err := s.repo.Get(ctx, *parent)
		if err != nil {
			return err
		}
		parent = n.ParentID
	}
	return nil
}
func (s *service) Create(ctx context.Context, a actor.Actor, in Create) (m Mind, err error) {
	now := s.audit.Clock.Now().UTC()
	m = Mind{ID: s.audit.IDs.New(), ParentID: in.ParentID, Name: strings.TrimSpace(in.Name), Color: in.Color, Icon: in.Icon, SortOrder: in.SortOrder, CreatedAt: now, UpdatedAt: now, Rev: 1}
	err = s.audit.Write(ctx, a, "mind", m.ID, activity.OpCreate, "Create mind", func(ctx context.Context) error {
		if e := s.validate(ctx, m); e != nil {
			return e
		}
		return s.repo.Insert(ctx, m)
	})
	return
}
func (s *service) Update(ctx context.Context, a actor.Actor, id string, rev int64, p Patch) (out Mind, err error) {
	err = s.audit.Write(ctx, a, "mind", id, activity.OpUpdate, "Update mind", func(ctx context.Context) error {
		m, e := s.repo.Get(ctx, id)
		if e != nil {
			return e
		}
		if rev != 0 && m.Rev != rev {
			return fault.Conflict("mind", id, m.Rev, m)
		}
		if p.ClearParent && p.ParentID != nil || p.ClearColor && p.Color != nil || p.ClearIcon && p.Icon != nil {
			return activity.Invalid("patch", "cannot set and clear the same field")
		}
		if p.Name != nil {
			m.Name = strings.TrimSpace(*p.Name)
		}
		if p.ClearParent {
			m.ParentID = nil
		} else if p.ParentID != nil {
			m.ParentID = p.ParentID
		}
		if p.ClearColor {
			m.Color = nil
		} else if p.Color != nil {
			m.Color = p.Color
		}
		if p.ClearIcon {
			m.Icon = nil
		} else if p.Icon != nil {
			m.Icon = p.Icon
		}
		if p.SortOrder != nil {
			m.SortOrder = *p.SortOrder
		}
		if e = s.validate(ctx, m); e != nil {
			return e
		}
		oldRev := m.Rev
		m.Rev++
		m.UpdatedAt = s.audit.Clock.Now().UTC()
		if e = s.repo.Update(ctx, oldRev, m); e != nil {
			return e
		}
		out = m
		return nil
	})
	return
}
func (s *service) Delete(ctx context.Context, a actor.Actor, id string) (out DeleteResult, err error) {
	err = s.audit.Write(ctx, a, "mind", id, activity.OpDelete, "Delete mind tree", func(ctx context.Context) error {
		if _, e := s.repo.Get(ctx, id); e != nil {
			return e
		}
		rows, e := s.repo.List(ctx)
		if e != nil {
			return e
		}
		seen := map[string]bool{id: true}
		for changed := true; changed; {
			changed = false
			for _, m := range rows {
				if m.ParentID != nil && seen[*m.ParentID] && !seen[m.ID] {
					seen[m.ID] = true
					changed = true
				}
			}
		}
		for id := range seen {
			out.DeletedMindIDs = append(out.DeletedMindIDs, id)
		}
		sort.Strings(out.DeletedMindIDs)
		out.MovedMemoryCount, e = s.memories.CountByMinds(ctx, out.DeletedMindIDs)
		if e != nil {
			return e
		}
		return s.repo.Delete(ctx, id)
	})
	return
}
func equalParent(a, b *string) bool { return a == nil && b == nil || a != nil && b != nil && *a == *b }
func (s *service) Reorder(ctx context.Context, a actor.Actor, parent *string, ids []string) (out []Mind, err error) {
	err = s.audit.Transactions.Within(ctx, func(ctx context.Context) error {
		rows, e := s.repo.List(ctx)
		if e != nil {
			return e
		}
		siblings := map[string]Mind{}
		for _, m := range rows {
			if equalParent(m.ParentID, parent) {
				siblings[m.ID] = m
			}
		}
		if len(ids) != len(siblings) {
			return activity.Invalid("ordered_ids", "must contain every sibling exactly once")
		}
		seen := map[string]bool{}
		for i, id := range ids {
			m, ok := siblings[id]
			if !ok || seen[id] {
				return activity.Invalid("ordered_ids", "invalid sibling or duplicate")
			}
			seen[id] = true
			order := i
			updated, e := s.Update(ctx, a, id, m.Rev, Patch{SortOrder: &order})
			if e != nil {
				return e
			}
			out = append(out, updated)
		}
		return nil
	})
	return
}
