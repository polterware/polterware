package fake

import (
	"context"
	store "github.com/polterware/polterware/sparky/backend/domains/activity/fake"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"sort"
)

type Repository struct{ Store *store.Store }

func New(s *store.Store) *Repository { return &Repository{s} }

var _ mind.Repository = (*Repository)(nil)

func (r *Repository) List(ctx context.Context) ([]mind.Mind, error) {
	out, err := store.All[mind.Mind](ctx, r.Store, "minds")
	sort.Slice(out, func(i, j int) bool { return out[i].ID < out[j].ID })
	return out, err
}
func (r *Repository) Get(ctx context.Context, id string) (mind.Mind, error) {
	v, ok, err := store.Read[mind.Mind](ctx, r.Store, "minds", id)
	if err == nil && !ok {
		err = fault.NotFound("mind", id)
	}
	return v, err
}
func (r *Repository) Insert(ctx context.Context, v mind.Mind) error {
	return r.Store.Within(ctx, func(ctx context.Context) error {
		if _, err := r.Get(ctx, v.ID); err == nil {
			return fault.Exists("mind", v.ID)
		} else if !fault.IsCode(err, fault.CodeNotFound) {
			return err
		}
		if v.ParentID != nil {
			if _, err := r.Get(ctx, *v.ParentID); err != nil {
				return err
			}
		}
		return r.Store.Put(ctx, "minds", v.ID, v)
	})
}
func (r *Repository) Update(ctx context.Context, rev int64, v mind.Mind) error {
	return r.Store.Within(ctx, func(ctx context.Context) error {
		old, err := r.Get(ctx, v.ID)
		if err != nil {
			return err
		}
		if rev != 0 && old.Rev != rev {
			return fault.Conflict("mind", v.ID, old.Rev, old)
		}
		if v.ParentID != nil {
			if _, err := r.Get(ctx, *v.ParentID); err != nil {
				return err
			}
		}
		return r.Store.Put(ctx, "minds", v.ID, v)
	})
}
func (r *Repository) Delete(ctx context.Context, rev int64, id string) error {
	return r.Store.Within(ctx, func(ctx context.Context) error {
		old, e := r.Get(ctx, id)
		if e != nil {
			return e
		}
		if rev != 0 && old.Rev != rev {
			return fault.Conflict("mind", id, old.Rev, old)
		}
		rows, e := r.List(ctx)
		if e != nil {
			return e
		}
		removed := map[string]bool{id: true}
		for changed := true; changed; {
			changed = false
			for _, v := range rows {
				if v.ParentID != nil && removed[*v.ParentID] && !removed[v.ID] {
					removed[v.ID] = true
					changed = true
				}
			}
		}
		for id := range removed {
			if e = r.Store.Remove(ctx, "minds", id); e != nil {
				return e
			}
		}
		memories, e := store.All[memory.Memory](ctx, r.Store, "memories")
		if e != nil {
			return e
		}
		for _, m := range memories {
			if m.MindID != nil && removed[*m.MindID] {
				m.MindID = nil
				if e = r.Store.Put(ctx, "memories", m.ID, m); e != nil {
					return e
				}
			}
		}
		return nil
	})
}
