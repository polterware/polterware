package memory_test

import (
	"context"
	"errors"
	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/schedule"
	"github.com/polterware/polterware/sparky/backend/domains/tag"
	"strings"
	"sync"
	"testing"
	"time"
)

func TestMemoryCreatePatchAndRev(t *testing.T) {
	f := setup()
	ctx := f.Ctx
	a := actor.UI()
	n, e := f.NS.Create(ctx, a, mind.Create{Name: "Mind"})
	check(t, e)
	tag, e := f.TS.Create(ctx, a, "Tag", nil)
	check(t, e)
	due := time.Now().In(time.FixedZone("test", 3600))
	m, e := f.MS.Create(ctx, a, memory.Create{Title: " Title ", Note: "Note", MindID: &n.ID, Priority: ptr(3), DueAt: &due, DueAllDay: true, Pinned: true, TagIDs: []string{tag.ID}, Checklist: []memory.CheckItemInput{{Title: "A", Done: true}}})
	check(t, e)
	if m.Rev != 1 || m.Title != "Title" || m.DueAt.Location() != time.UTC {
		t.Fatal(m)
	}
	for _, tc := range []struct {
		in   memory.Create
		code string
	}{{memory.Create{Title: ""}, fault.CodeInvalid}, {memory.Create{Title: strings.Repeat("😀", 501)}, fault.CodeInvalid}, {memory.Create{Title: "X", Priority: ptr(4)}, fault.CodeInvalid}, {memory.Create{Title: "X", MindID: ptr("missing")}, fault.CodeNotFound}, {memory.Create{Title: "X", TagIDs: []string{"missing"}}, fault.CodeNotFound}, {memory.Create{Title: "X", TagIDs: []string{tag.ID, tag.ID}}, fault.CodeInvalid}, {memory.Create{Title: "X", Checklist: []memory.CheckItemInput{{ID: "foreign"}}}, fault.CodeInvalid}, {memory.Create{Title: "X", Checklist: []memory.CheckItemInput{{Title: strings.Repeat("x", 501)}}}, fault.CodeInvalid}} {
		_, e := f.MS.Create(ctx, a, tc.in)
		code(t, e, tc.code)
	}
	_, e = f.MS.Update(ctx, a, m.ID, 99, memory.Patch{})
	code(t, e, fault.CodeConflict)
	_, e = f.MS.Update(ctx, a, "missing", 0, memory.Patch{})
	code(t, e, fault.CodeNotFound)
	for _, p := range []memory.Patch{{ClearMindID: true, MindID: &n.ID}, {ClearPriority: true, Priority: ptr(1)}, {ClearDueAt: true, DueAt: &due}, {Title: ptr("")}, {Priority: ptr(-1)}} {
		_, e := f.MS.Update(ctx, a, m.ID, 1, p)
		code(t, e, fault.CodeInvalid)
	}
	m, e = f.MS.Update(ctx, a, m.ID, 1, memory.Patch{Title: ptr("New"), Note: ptr("Changed"), MindID: &n.ID, Priority: ptr(0), DueAt: &due, DueAllDay: ptr(false), SortOrder: ptr(8), Pinned: ptr(false), AutoCompleteOnChecklist: ptr(true)})
	check(t, e)
	if m.Rev != 2 || m.Status != memory.StatusCompleted || m.CompletedAt == nil {
		t.Fatal(m)
	}
	m, e = f.MS.Update(ctx, a, m.ID, 2, memory.Patch{ClearMindID: true, ClearPriority: true, ClearDueAt: true, AutoCompleteOnChecklist: ptr(false)})
	check(t, e)
	if m.MindID != nil || m.Priority != nil || m.DueAt != nil || m.DueAllDay {
		t.Fatal(m)
	}
	result, e := f.MS.Get(ctx, m.ID, memory.Include{Note: true, Checklist: true, Tags: true, Completions: true})
	check(t, e)
	if len(result.Checklist) != 1 || len(result.TagIDs) != 1 || result.Memory.Note != "Changed" {
		t.Fatal(result)
	}
	result, e = f.MS.Get(ctx, m.ID, memory.Include{})
	check(t, e)
	if result.Memory.Note != "" {
		t.Fatal(result)
	}
	_, e = f.MS.Get(ctx, "missing", memory.Include{})
	code(t, e, fault.CodeNotFound)
	m, e = f.MS.SetStatus(ctx, a, m.ID, 3, memory.StatusActive, nil)
	check(t, e)
	if m.CompletedAt != nil {
		t.Fatal(m)
	}
	m, e = f.MS.SetStatus(ctx, a, m.ID, 4, memory.StatusCompleted, nil)
	check(t, e)
	if m.CompletedAt == nil {
		t.Fatal(m)
	}
	_, e = f.MS.SetStatus(ctx, a, m.ID, 0, "bad", nil)
	code(t, e, fault.CodeInvalid)
	_, e = f.MS.SetStatus(ctx, a, m.ID, 0, memory.StatusCompleted, &due)
	code(t, e, fault.CodeUnavailable)
	check(t, f.MS.Delete(ctx, a, m.ID, 0))
	code(t, f.MS.Delete(ctx, a, m.ID, 0), fault.CodeNotFound)
}
func TestChecklistIdentityAndTags(t *testing.T) {
	f := setup()
	a := actor.UI()
	m, e := f.MS.Create(f.Ctx, a, memory.Create{Title: "Checklist", AutoCompleteOnChecklist: true, Checklist: []memory.CheckItemInput{{Title: "One"}, {Title: "Two", Done: true}}})
	check(t, e)
	if m.Status != memory.StatusActive {
		t.Fatal(m)
	}
	old, e := f.Mem.ListCheckItems(f.Ctx, m.ID)
	check(t, e)
	items, e := f.MS.ReplaceChecklist(f.Ctx, a, m.ID, 1, []memory.CheckItemInput{{ID: old[1].ID, Title: "Second", Done: true}, {Title: "New", Done: false}, {ID: old[0].ID, Title: "First", Done: false}})
	check(t, e)
	if items[0].ID != old[1].ID || !items[0].CreatedAt.Equal(old[1].CreatedAt) || items[0].CompletedAt == nil || items[2].CompletedAt != nil {
		t.Fatal(items)
	}
	for _, in := range [][]memory.CheckItemInput{{{ID: "foreign"}}, {{ID: items[0].ID}, {ID: items[0].ID}}, {{Title: strings.Repeat("x", 501)}}} {
		_, e := f.MS.ReplaceChecklist(f.Ctx, a, m.ID, 2, in)
		code(t, e, fault.CodeInvalid)
	}
	_, e = f.MS.ReplaceChecklist(f.Ctx, a, m.ID, 1, nil)
	code(t, e, fault.CodeConflict)
	item, e := f.MS.ToggleCheckItem(f.Ctx, a, m.ID, items[1].ID, 2, nil)
	check(t, e)
	if !item.Done || item.CompletedAt == nil {
		t.Fatal(item)
	}
	item, e = f.MS.ToggleCheckItem(f.Ctx, a, m.ID, items[1].ID, 3, ptr(false))
	check(t, e)
	if item.Done || item.CompletedAt != nil {
		t.Fatal(item)
	}
	_, e = f.MS.ToggleCheckItem(f.Ctx, a, m.ID, items[1].ID, 4, ptr(true))
	check(t, e)
	_, e = f.MS.ToggleCheckItem(f.Ctx, a, m.ID, items[2].ID, 5, ptr(true))
	check(t, e)
	result, e := f.MS.Get(f.Ctx, m.ID, memory.Include{})
	check(t, e)
	if result.Memory.Status != memory.StatusCompleted {
		t.Fatal(result)
	}
	other, e := f.MS.Create(f.Ctx, a, memory.Create{Title: "Other"})
	check(t, e)
	_, e = f.MS.ToggleCheckItem(f.Ctx, a, other.ID, items[0].ID, 0, nil)
	code(t, e, fault.CodeInvalid)
	_, e = f.MS.ToggleCheckItem(f.Ctx, a, m.ID, "missing", 0, nil)
	code(t, e, fault.CodeNotFound)
	_, e = f.MS.ReplaceChecklist(f.Ctx, a, m.ID, 0, nil)
	check(t, e)
	auto, e := f.MS.Create(f.Ctx, a, memory.Create{Title: "Auto", AutoCompleteOnChecklist: true, Checklist: []memory.CheckItemInput{{Done: true}}})
	check(t, e)
	if auto.Status != memory.StatusCompleted {
		t.Fatal(auto)
	}
	tg, e := f.TS.Create(f.Ctx, a, "Tag", nil)
	check(t, e)
	check(t, f.MS.SetTags(f.Ctx, a, m.ID, 0, []string{tg.ID}))
	code(t, f.MS.SetTags(f.Ctx, a, m.ID, 0, []string{"missing"}), fault.CodeNotFound)
	check(t, f.MS.SetTags(f.Ctx, a, m.ID, 0, nil))
}
func TestListReorderAndBulk(t *testing.T) {
	f := setup()
	a := actor.CLI()
	n, e := f.NS.Create(f.Ctx, a, mind.Create{Name: "Mind"})
	check(t, e)
	tg, e := f.TS.Create(f.Ctx, a, "Tag", nil)
	check(t, e)
	one, e := f.MS.Create(f.Ctx, a, memory.Create{Title: "One"})
	check(t, e)
	two, e := f.MS.Create(f.Ctx, a, memory.Create{Title: "Two"})
	check(t, e)
	for _, filter := range []memory.ListFilter{{Limit: -1}, {Limit: 501}, {Status: "bad"}, {Order: "bad"}, {DueFrom: ptr(time.Now()), DueTo: ptr(time.Now().Add(-time.Hour))}, {UpdatedFrom: ptr(time.Now()), UpdatedTo: ptr(time.Now().Add(-time.Hour))}} {
		_, e := f.MS.List(f.Ctx, filter)
		code(t, e, fault.CodeInvalid)
	}
	page, e := f.MS.List(f.Ctx, memory.ListFilter{Limit: 1})
	check(t, e)
	if len(page.Memories) != 1 || page.NextCursor == "" {
		t.Fatal(page)
	}
	next, e := f.MS.List(f.Ctx, memory.ListFilter{Limit: 1, Cursor: page.NextCursor})
	check(t, e)
	if next.Memories[0].ID == page.Memories[0].ID {
		t.Fatal(next)
	}
	check(t, f.MS.Reorder(f.Ctx, a, nil, []string{two.ID, one.ID}))
	code(t, f.MS.Reorder(f.Ctx, a, nil, []string{one.ID, one.ID}), fault.CodeInvalid)
	code(t, f.MS.Reorder(f.Ctx, a, &n.ID, []string{one.ID}), fault.CodeInvalid)
	code(t, f.MS.Reorder(f.Ctx, a, nil, []string{"missing"}), fault.CodeNotFound)
	for _, action := range []string{memory.BulkComplete, memory.BulkReopen, memory.BulkPin, memory.BulkUnpin, memory.BulkMove, memory.BulkTag, memory.BulkTag, memory.BulkUntag} {
		results, e := f.MS.BulkAction(f.Ctx, a, memory.BulkAction{IDs: []string{one.ID, two.ID, "missing"}, Action: action, MindID: &n.ID, TagIDs: []string{tg.ID}})
		check(t, e)
		if results[0].Err != nil || results[1].Err != nil || results[2].Err == nil {
			t.Fatal(results)
		}
	}
	_, e = f.MS.BulkAction(f.Ctx, a, memory.BulkAction{Action: "bad"})
	code(t, e, fault.CodeInvalid)
	_, e = f.MS.BulkAction(f.Ctx, a, memory.BulkAction{Action: memory.BulkPin, IDs: []string{one.ID, one.ID}})
	code(t, e, fault.CodeInvalid)
	results, e := f.MS.BulkAction(f.Ctx, a, memory.BulkAction{IDs: []string{one.ID}, Action: memory.BulkTag, TagIDs: []string{"missing"}})
	check(t, e)
	code(t, results[0].Err, fault.CodeNotFound)
	results, e = f.MS.BulkAction(f.Ctx, a, memory.BulkAction{IDs: []string{one.ID}, Action: memory.BulkMove})
	check(t, e)
	check(t, results[0].Err)
	results, e = f.MS.BulkAction(f.Ctx, a, memory.BulkAction{IDs: []string{one.ID, two.ID}, Action: memory.BulkDelete})
	check(t, e)
	check(t, results[0].Err)
	check(t, results[1].Err)
}

// Test planners expose explicit fixture occurrences; production math belongs to W1-RECURRENCE.
type scheduleRepo struct {
	schedule.Repository
	value schedule.Schedule
	err   error
}

func (r scheduleRepo) Get(context.Context, string) (schedule.Schedule, error) { return r.value, r.err }

type planner struct {
	schedule.Planner
	ats     []time.Time
	err     error
	tooMany bool
}

func (p planner) Preview(schedule.Rule, int) ([]time.Time, string, error) { return p.ats, "", p.err }
func (p planner) Occurrences(subjects []schedule.PlanSubject, from, to time.Time, include bool, limit int) ([]schedule.Occurrence, error) {
	if p.err != nil {
		return nil, p.err
	}
	if p.tooMany {
		return make([]schedule.Occurrence, 100001), nil
	}
	out := []schedule.Occurrence{}
	for _, at := range p.ats {
		if !at.Before(from) && at.Before(to) {
			out = append(out, schedule.Occurrence{MemoryID: subjects[0].MemoryID, OccurrenceAt: at})
		}
	}
	return out, nil
}
func TestRecurringCompletion(t *testing.T) {
	for _, end := range []schedule.EndType{schedule.EndNever, schedule.EndCount, schedule.EndUntil} {
		t.Run(string(end), func(t *testing.T) {
			f := setup()
			a := actor.UI()
			m, e := f.MS.Create(f.Ctx, a, memory.Create{Title: "Series"})
			check(t, e)
			now := f.Audit.Clock.Now()
			ats := []time.Time{now.Add(-48 * time.Hour), now.Add(-24 * time.Hour)}
			sc := schedule.Schedule{MemoryID: m.ID, FireAt: ats[0], TZ: "UTC", Freq: schedule.FreqDaily, Every: 1, EndType: end}
			if end == schedule.EndCount {
				sc.OccurrenceCount = ptr(2)
			}
			if end == schedule.EndUntil {
				sc.UntilAt = &ats[1]
			}
			s := memory.NewService(f.Mem, f.Minds, f.Tags, scheduleRepo{value: sc}, planner{ats: ats}, f.Audit)
			_, e = s.SetStatus(f.Ctx, a, m.ID, 0, memory.StatusCompleted, ptr(ats[0].Add(time.Minute)))
			code(t, e, fault.CodeInvalid)
			m, e = s.SetStatus(f.Ctx, a, m.ID, 1, memory.StatusCompleted, &ats[0])
			check(t, e)
			if m.Status != memory.StatusActive {
				t.Fatal(m)
			}
			m, e = s.SetStatus(f.Ctx, a, m.ID, 2, memory.StatusCompleted, &ats[0])
			check(t, e)
			cs, e := f.Mem.ListCompletions(f.Ctx, m.ID)
			check(t, e)
			if len(cs) != 1 {
				t.Fatal(cs)
			}
			m, e = s.SetStatus(f.Ctx, a, m.ID, 3, memory.StatusCompleted, &ats[1])
			check(t, e)
			if (m.Status == memory.StatusCompleted) != (end != schedule.EndNever) {
				t.Fatal(m)
			}
			m, e = s.SetStatus(f.Ctx, a, m.ID, 4, memory.StatusActive, &ats[0])
			check(t, e)
			if m.Status != memory.StatusActive || m.CompletedAt != nil {
				t.Fatal(m)
			}
			cs, e = f.Mem.ListCompletions(f.Ctx, m.ID)
			check(t, e)
			if len(cs) != 1 {
				t.Fatal(cs)
			}
		})
	}
}
func TestRecurringValidationAndFailures(t *testing.T) {
	f := setup()
	m, e := f.MS.Create(f.Ctx, actor.UI(), memory.Create{Title: "Series"})
	check(t, e)
	now := f.Audit.Clock.Now()
	at := now.Add(-time.Hour)
	good := schedule.Schedule{FireAt: at, TZ: "UTC", Freq: schedule.FreqHourly, Every: 1, EndType: schedule.EndCount, OccurrenceCount: ptr(1)}
	for _, tc := range []struct {
		name    string
		sc      schedule.Schedule
		p       planner
		repoErr error
		want    string
	}{{"one shot", schedule.Schedule{}, planner{}, nil, fault.CodeInvalid}, {"invalid zone", schedule.Schedule{Freq: schedule.FreqHourly, TZ: "bad"}, planner{}, nil, fault.CodeInvalid}, {"schedule missing", good, planner{}, fault.NotFound("schedule", m.ID), fault.CodeNotFound}, {"engine error", good, planner{err: errors.New("engine")}, nil, ""}, {"budget", func() schedule.Schedule { s := good; s.OccurrenceCount = ptr(100001); return s }(), planner{ats: []time.Time{at}}, nil, fault.CodeExhausted}, {"nil count", func() schedule.Schedule { s := good; s.OccurrenceCount = nil; return s }(), planner{ats: []time.Time{at}}, nil, fault.CodeExhausted}} {
		t.Run(tc.name, func(t *testing.T) {
			s := memory.NewService(f.Mem, f.Minds, f.Tags, scheduleRepo{value: tc.sc, err: tc.repoErr}, tc.p, f.Audit)
			_, e := s.SetStatus(f.Ctx, actor.UI(), m.ID, 1, memory.StatusCompleted, &at)
			if tc.want != "" {
				code(t, e, tc.want)
			} else if e == nil {
				t.Fatal("missing error")
			}
		})
	}
	// A finite series stays active until the final occurrence is due.
	good.OccurrenceCount = ptr(2)
	s := memory.NewService(f.Mem, f.Minds, f.Tags, scheduleRepo{value: good}, planner{ats: []time.Time{at, now.Add(time.Hour)}}, f.Audit)
	m, e = s.SetStatus(f.Ctx, actor.UI(), m.ID, 1, memory.StatusCompleted, &at)
	check(t, e)
	if m.Status != memory.StatusActive {
		t.Fatal(m)
	}
}

func TestDeleteAndToggleExpectedRev(t *testing.T) {
	a := actor.UI()
	for _, tc := range []struct {
		name string
		rev  int64
		want string
	}{{"zero skips", 0, ""}, {"match", 1, ""}, {"stale", 99, fault.CodeConflict}} {
		t.Run("delete/"+tc.name, func(t *testing.T) {
			f := setup()
			m, e := f.MS.Create(f.Ctx, a, memory.Create{Title: "Gone"})
			check(t, e)
			e = f.MS.Delete(f.Ctx, a, m.ID, tc.rev)
			if tc.want == "" {
				check(t, e)
				code(t, f.MS.Delete(f.Ctx, a, m.ID, 0), fault.CodeNotFound)
				return
			}
			code(t, e, tc.want)
			got, e := f.MS.Get(f.Ctx, m.ID, memory.Include{})
			check(t, e)
			if got.Memory.Rev != 1 {
				t.Fatal(got.Memory)
			}
		})
	}
	t.Run("toggle", func(t *testing.T) {
		f := setup()
		m, e := f.MS.Create(f.Ctx, a, memory.Create{Title: "List", Checklist: []memory.CheckItemInput{{Title: "A"}}})
		check(t, e)
		items, e := f.Mem.ListCheckItems(f.Ctx, m.ID)
		check(t, e)
		_, e = f.MS.ToggleCheckItem(f.Ctx, a, m.ID, items[0].ID, 99, nil)
		code(t, e, fault.CodeConflict)
		item, e := f.MS.ToggleCheckItem(f.Ctx, a, m.ID, items[0].ID, 1, nil)
		check(t, e)
		if !item.Done {
			t.Fatal(item)
		}
		_, e = f.MS.ToggleCheckItem(f.Ctx, a, m.ID, items[0].ID, 1, nil)
		code(t, e, fault.CodeConflict)
	})
}

func TestConcurrentToggleWriters(t *testing.T) {
	f := setup()
	a := actor.UI()
	m, e := f.MS.Create(f.Ctx, a, memory.Create{Title: "Race", Checklist: []memory.CheckItemInput{{Title: "A"}}})
	check(t, e)
	items, e := f.Mem.ListCheckItems(f.Ctx, m.ID)
	check(t, e)
	errs := make(chan error, 2)
	var wg sync.WaitGroup
	for i := 0; i < 2; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			_, err := f.MS.ToggleCheckItem(f.Ctx, a, m.ID, items[0].ID, 1, nil)
			errs <- err
		}()
	}
	wg.Wait()
	close(errs)
	ok, conflict := 0, 0
	for err := range errs {
		if err == nil {
			ok++
			continue
		}
		if fault.IsCode(err, fault.CodeConflict) {
			conflict++
			continue
		}
		t.Fatal(err)
	}
	if ok != 1 || conflict != 1 {
		t.Fatalf("ok=%d conflict=%d", ok, conflict)
	}
	got, e := f.MS.Get(f.Ctx, m.ID, memory.Include{})
	check(t, e)
	if got.Memory.Rev != 2 {
		t.Fatal(got.Memory)
	}
}

var _ tag.Tag
