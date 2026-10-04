package fake_test

import (
	"context"
	"errors"
	"github.com/polterware/polterware/sparky/backend/domains/activity/fake/testkit"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"testing"
)

func TestMindRepositoryContract(t *testing.T) {
	f := testkit.Setup()
	ctx := f.Ctx
	parent := mind.Mind{ID: "parent", Name: "Parent", Rev: 1}
	child := mind.Mind{ID: "child", ParentID: &parent.ID, Name: "Child", Rev: 1}
	for _, m := range []mind.Mind{parent, child} {
		if e := f.Minds.Insert(ctx, m); e != nil {
			t.Fatal(e)
		}
	}
	if e := f.Minds.Insert(ctx, parent); !fault.IsCode(e, fault.CodeExists) {
		t.Fatal(e)
	}
	missing := "missing"
	bad := mind.Mind{ID: "bad", ParentID: &missing}
	if e := f.Minds.Insert(ctx, bad); !fault.IsCode(e, fault.CodeNotFound) {
		t.Fatal(e)
	}
	if _, e := f.Minds.Get(ctx, "missing"); !fault.IsCode(e, fault.CodeNotFound) {
		t.Fatal(e)
	}
	if e := f.Minds.Update(ctx, 2, parent); !fault.IsCode(e, fault.CodeConflict) {
		t.Fatal(e)
	}
	parent.Rev = 2
	if e := f.Minds.Update(ctx, 1, parent); e != nil {
		t.Fatal(e)
	}
	child.ParentID = &missing
	if e := f.Minds.Update(ctx, 0, child); !fault.IsCode(e, fault.CodeNotFound) {
		t.Fatal(e)
	}
	rows, e := f.Minds.List(ctx)
	if e != nil || len(rows) != 2 {
		t.Fatal(rows, e)
	}
	if e = f.Mem.Insert(ctx, memory.Memory{ID: "survivor", MindID: &parent.ID}); e != nil {
		t.Fatal(e)
	}
	if e = f.Minds.Delete(ctx, parent.ID); e != nil {
		t.Fatal(e)
	}
	rows, e = f.Minds.List(ctx)
	if e != nil || len(rows) != 0 {
		t.Fatal(rows, e)
	}
	m, e := f.Mem.Get(ctx, "survivor")
	if e != nil || m.MindID != nil {
		t.Fatal(m, e)
	}
	if e = f.Minds.Delete(ctx, "missing"); !fault.IsCode(e, fault.CodeNotFound) {
		t.Fatal(e)
	}
	c, cancel := context.WithCancel(ctx)
	cancel()
	for _, call := range []func() error{func() error { _, e := f.Minds.Get(c, "id"); return e }, func() error { _, e := f.Minds.List(c); return e }, func() error { return f.Minds.Insert(c, parent) }, func() error { return f.Minds.Update(c, 1, parent) }, func() error { return f.Minds.Delete(c, "id") }} {
		if e := call(); !errors.Is(e, context.Canceled) {
			t.Fatal(e)
		}
	}
}
