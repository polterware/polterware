package mind_test

import (
	"context"
	"errors"
	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"strings"
	"sync"
	"testing"
)

func TestTreeRules(t *testing.T) {
	f := setup()
	ctx := f.Ctx
	a := actor.UI()
	root, e := f.NS.Create(ctx, a, mind.Create{Name: "Root", Color: ptr("#AABBCC"), Icon: ptr("folder")})
	check(t, e)
	child, e := f.NS.Create(ctx, a, mind.Create{Name: "Child", ParentID: &root.ID})
	check(t, e)
	leaf, e := f.NS.Create(ctx, a, mind.Create{Name: "Leaf", ParentID: &child.ID})
	check(t, e)
	for _, tc := range []struct {
		name string
		p    mind.Patch
		code string
	}{{"self", mind.Patch{ParentID: &root.ID}, fault.CodeCycle}, {"descendant", mind.Patch{ParentID: &leaf.ID}, fault.CodeCycle}, {"missing", mind.Patch{ParentID: ptr("missing")}, fault.CodeNotFound}, {"blank", mind.Patch{Name: ptr(" ")}, fault.CodeInvalid}, {"long", mind.Patch{Name: ptr(strings.Repeat("a", 121))}, fault.CodeInvalid}, {"color", mind.Patch{Color: ptr("red")}, fault.CodeInvalid}, {"conflicting parent", mind.Patch{ParentID: &child.ID, ClearParent: true}, fault.CodeInvalid}, {"conflicting color", mind.Patch{Color: ptr("#ABCDEF"), ClearColor: true}, fault.CodeInvalid}, {"conflicting icon", mind.Patch{Icon: ptr("x"), ClearIcon: true}, fault.CodeInvalid}} {
		t.Run(tc.name, func(t *testing.T) { _, e := f.NS.Update(ctx, a, root.ID, 1, tc.p); code(t, e, tc.code) })
	}
	_, e = f.NS.Update(ctx, a, root.ID, 99, mind.Patch{})
	code(t, e, fault.CodeConflict)
	_, e = f.NS.Update(ctx, a, "missing", 1, mind.Patch{})
	code(t, e, fault.CodeNotFound)
	updated, e := f.NS.Update(ctx, a, root.ID, 1, mind.Patch{Name: ptr("Renamed"), Color: ptr("#010203"), Icon: ptr("star"), SortOrder: ptr(9)})
	check(t, e)
	if updated.Rev != 2 || updated.Name != "Renamed" {
		t.Fatal(updated)
	}
	updated, e = f.NS.Update(ctx, a, root.ID, 2, mind.Patch{ClearColor: true, ClearIcon: true})
	check(t, e)
	if updated.Color != nil || updated.Icon != nil {
		t.Fatal(updated)
	}
	child, e = f.NS.Update(ctx, a, child.ID, 1, mind.Patch{ClearParent: true})
	check(t, e)
	child, e = f.NS.Update(ctx, a, child.ID, 2, mind.Patch{ParentID: &root.ID})
	check(t, e)
	_, e = f.NS.Get(ctx, root.ID)
	check(t, e)
	rows, e := f.NS.List(ctx)
	check(t, e)
	if len(rows) != 3 {
		t.Fatal(rows)
	}
	found, e := mind.ResolveName(ctx, f.Minds, "renamed")
	check(t, e)
	if found.ID != root.ID {
		t.Fatal(found)
	}
	_, e = mind.ResolveName(ctx, f.Minds, "rename")
	code(t, e, fault.CodeNotFound)
	_, e = f.MS.Create(ctx, a, memory.Create{Title: "Survives", MindID: &leaf.ID})
	check(t, e)
	result, e := f.NS.Delete(ctx, a, root.ID, 0)
	check(t, e)
	if len(result.DeletedMindIDs) != 3 || result.MovedMemoryCount != 1 {
		t.Fatal(result)
	}
	page, e := f.MS.List(ctx, memory.ListFilter{InboxOnly: true})
	check(t, e)
	if len(page.Memories) != 1 {
		t.Fatal(page)
	}
	_, e = f.NS.Delete(ctx, a, "missing", 0)
	code(t, e, fault.CodeNotFound)
}
func TestMindCreateAndReorder(t *testing.T) {
	f := setup()
	for _, in := range []mind.Create{{Name: ""}, {Name: "X", Color: ptr("#GGGGGG")}, {Name: "X", ParentID: ptr("missing")}} {
		_, e := f.NS.Create(f.Ctx, actor.UI(), in)
		if e == nil {
			t.Fatal(in)
		}
	}
	one, e := f.NS.Create(f.Ctx, actor.CLI(), mind.Create{Name: "One"})
	check(t, e)
	two, e := f.NS.Create(f.Ctx, actor.CLI(), mind.Create{Name: "Two"})
	check(t, e)
	for _, ids := range [][]string{{one.ID}, {one.ID, one.ID}, {one.ID, "missing"}} {
		_, e := f.NS.Reorder(f.Ctx, actor.CLI(), nil, ids)
		code(t, e, fault.CodeInvalid)
	}
	rows, e := f.NS.Reorder(f.Ctx, actor.CLI(), nil, []string{two.ID, one.ID})
	check(t, e)
	if rows[0].SortOrder != 0 || rows[1].SortOrder != 1 || rows[0].Rev != 2 {
		t.Fatal(rows)
	}
}

type brokenMind struct {
	mind.Repository
	method string
	err    error
}

func (r brokenMind) List(c context.Context) ([]mind.Mind, error) {
	if r.method == "list" {
		return nil, r.err
	}
	return r.Repository.List(c)
}
func (r brokenMind) Get(c context.Context, id string) (mind.Mind, error) {
	if r.method == "get" {
		return mind.Mind{}, r.err
	}
	return r.Repository.Get(c, id)
}
func (r brokenMind) Update(c context.Context, rev int64, m mind.Mind) error {
	if r.method == "update" {
		return r.err
	}
	return r.Repository.Update(c, rev, m)
}

type brokenCount struct{ memory.Repository }

func (r brokenCount) CountByMinds(context.Context, []string) (int, error) {
	return 0, errors.New("count failed")
}
func TestMindFailureRollback(t *testing.T) {
	for _, method := range []string{"list", "get", "update", "count"} {
		t.Run(method, func(t *testing.T) {
			f := setup()
			m, e := f.NS.Create(f.Ctx, actor.UI(), mind.Create{Name: "X"})
			check(t, e)
			err := errors.New("failed")
			counter := mind.MemoryCounter(f.Mem)
			if method == "count" {
				counter = brokenCount{f.Mem}
			}
			s := mind.NewService(brokenMind{f.Minds, method, err}, counter, f.Audit)
			switch method {
			case "list":
				_, e = mind.ResolveName(f.Ctx, brokenMind{f.Minds, method, err}, "X")
				if !errors.Is(e, err) {
					t.Fatal(e)
				}
				_, e = s.Delete(f.Ctx, actor.UI(), m.ID, 0)
				if e == nil {
					t.Fatal("delete succeeded")
				}
				_, e = s.Reorder(f.Ctx, actor.UI(), nil, []string{m.ID})
				if e == nil {
					t.Fatal("reorder succeeded")
				}
			case "get":
				_, e = s.Update(f.Ctx, actor.UI(), m.ID, 1, mind.Patch{})
			case "update":
				_, e = s.Update(f.Ctx, actor.UI(), m.ID, 1, mind.Patch{})
			case "count":
				_, e = s.Delete(f.Ctx, actor.UI(), m.ID, 0)
			}
			if e == nil {
				t.Fatal("failure not propagated")
			}
			got, e := f.NS.Get(f.Ctx, m.ID)
			check(t, e)
			if got.Rev != 1 {
				t.Fatal(got)
			}
		})
	}
}

func TestMindDeleteExpectedRev(t *testing.T) {
	a := actor.UI()
	for _, tc := range []struct {
		name string
		rev  int64
		want string
	}{{"zero skips", 0, ""}, {"match", 1, ""}, {"stale", 99, fault.CodeConflict}} {
		t.Run(tc.name, func(t *testing.T) {
			f := setup()
			m, e := f.NS.Create(f.Ctx, a, mind.Create{Name: "Work"})
			check(t, e)
			_, e = f.NS.Delete(f.Ctx, a, m.ID, tc.rev)
			if tc.want == "" {
				check(t, e)
				return
			}
			code(t, e, tc.want)
			got, e := f.NS.Get(f.Ctx, m.ID)
			check(t, e)
			if got.Rev != 1 {
				t.Fatal(got)
			}
		})
	}
}

func TestConcurrentMindDeleteWriters(t *testing.T) {
	f := setup()
	a := actor.UI()
	m, e := f.NS.Create(f.Ctx, a, mind.Create{Name: "Race"})
	check(t, e)
	errs := make(chan error, 2)
	var wg sync.WaitGroup
	wg.Add(2)
	go func() {
		defer wg.Done()
		name := "UI"
		_, err := f.NS.Update(f.Ctx, a, m.ID, 1, mind.Patch{Name: &name})
		errs <- err
	}()
	go func() {
		defer wg.Done()
		_, err := f.NS.Delete(f.Ctx, a, m.ID, 1)
		errs <- err
	}()
	wg.Wait()
	close(errs)
	ok, lost := 0, 0
	for err := range errs {
		if err == nil {
			ok++
			continue
		}
		if fault.IsCode(err, fault.CodeConflict) || fault.IsCode(err, fault.CodeNotFound) {
			lost++
			continue
		}
		t.Fatal(err)
	}
	if ok != 1 || lost != 1 {
		t.Fatalf("ok=%d lost=%d", ok, lost)
	}
}
