package schedule

import (
	"fmt"
	"reflect"
	"slices"
	"strings"
	"testing"
	"time"
	"unicode/utf8"

	"github.com/polterware/polterware/sparky/backend/domains/fault"
)

func TestPlanNearestFirstGlobalBudget(t *testing.T) {
	now := instant(t, "2026-10-04T10:00:00Z")
	fast := daily(t)
	fast.Anchor = now
	fast.Freq = FreqMinutely
	slow := fast
	slow.Freq = FreqHourly
	slow.Anchor = now.Add(30 * time.Second)
	subjects := []PlanSubject{
		{MemoryID: "fast", Rule: fast, Title: "Fast", Note: "# **Hello** [world](https://example.com)", CompletedAt: []time.Time{now}},
		{MemoryID: "slow", Rule: slow, Title: "Slow", FocusEnabled: true},
		{MemoryID: "inactive", Rule: Rule{}, ScheduleInactive: true},
		{MemoryID: "done", Rule: Rule{}, Status: "completed"},
	}
	ones := []PendingOneOff{
		{ID: "focus", Kind: "focus_phase", FireAt: now.Add(15 * time.Second), Title: "Break"},
		{ID: "snooze", Kind: "snooze", MemoryID: "fast", FireAt: now.Add(45 * time.Second)},
		{ID: "past", Kind: "snooze", FireAt: now.Add(-time.Second)},
		{ID: "outside", Kind: "snooze", FireAt: now.Add(2 * time.Hour)},
	}
	q := PlanQuery{Now: now, Budget: 5, Horizon: time.Hour}
	got, err := Plan(q, subjects, ones, "")
	if err != nil {
		t.Fatal(err)
	}
	wantIDs := []string{"one:focus", reminderID("slow", slow.Anchor), "one:snooze", reminderID("fast", now.Add(time.Minute)), reminderID("fast", now.Add(2*time.Minute))}
	if len(got) != len(wantIDs) {
		t.Fatalf("got %v", got)
	}
	for i, id := range wantIDs {
		if got[i].ID != id {
			t.Fatalf("[%d] got %s want %s", i, got[i].ID, id)
		}
	}
	if got[0].Category != CategoryFocusPhase || got[1].Category != CategoryReminderFocus || got[2].Category != CategoryReminder || got[3].Body != "Hello world" {
		t.Fatalf("shaping/categories: %v", got)
	}
	// Results and inputs must remain stable across repeated calls and input order.
	slices.Reverse(subjects)
	slices.Reverse(ones)
	again, err := Plan(q, subjects, ones, "")
	if err != nil || !reflect.DeepEqual(got, again) {
		t.Fatalf("not deterministic: %v %v", again, err)
	}
}

func TestPlanDefaultsAndBoundaries(t *testing.T) {
	r := daily(t)
	r.Freq = FreqMinutely
	subjects := []PlanSubject{{MemoryID: "m", Rule: r}}
	got, err := Plan(PlanQuery{Now: r.Anchor}, subjects, nil, "")
	if err != nil || len(got) != DefaultBudget {
		t.Fatalf("default budget: %d %v", len(got), err)
	}
	if !got[0].FireAt.Equal(r.Anchor) {
		t.Fatal("now is inclusive")
	}
	got, err = Plan(PlanQuery{Now: r.Anchor, Horizon: 2 * time.Minute}, subjects, nil, "")
	if err != nil || len(got) != 3 {
		t.Fatalf("horizon inclusive: %d %v", len(got), err)
	}
	r.End = End{Type: EndCount, Count: 2}
	subjects[0].Rule = r
	got, err = Plan(PlanQuery{Now: r.Anchor}, subjects, nil, "")
	if err != nil || len(got) != 2 {
		t.Fatalf("bounded plan: %v %v", got, err)
	}
	for _, q := range []PlanQuery{{}, {Now: r.Anchor, Budget: -1}, {Now: r.Anchor, Budget: 61}, {Now: r.Anchor, Horizon: -time.Hour}} {
		if _, err := Plan(q, subjects, nil, ""); !fault.IsCode(err, fault.CodeInvalid) {
			t.Fatalf("got %v", err)
		}
	}
	for _, clock := range []string{"9:00", "24:00", "09:60", "-1:00", "+1:00", "09:xx"} {
		if _, err := Plan(PlanQuery{Now: r.Anchor}, subjects, nil, clock); !fault.IsCode(err, fault.CodeInvalid) {
			t.Fatalf("clock %s: %v", clock, err)
		}
	}
	if _, err := Plan(PlanQuery{Now: r.Anchor}, nil, []PendingOneOff{{Kind: "unknown"}}, ""); !fault.IsCode(err, fault.CodeInvalid) {
		t.Fatal(err)
	}
}

func TestPlanAllDayCompletionKey(t *testing.T) {
	r := daily(t)
	r.TZ = location(t, "America/Sao_Paulo")
	r.AllDay = true
	r.Anchor = instant(t, "2026-10-03T18:30:00-03:00") // anchor time ignored
	key := instant(t, "2026-10-03T00:00:00-03:00")
	subjects := []PlanSubject{{MemoryID: "all", Rule: r}}
	now := instant(t, "2026-10-03T08:00:00-03:00")
	got, err := Plan(PlanQuery{Now: now, Budget: 3, Horizon: 4 * 24 * time.Hour}, subjects, nil, "")
	if err != nil || len(got) != 3 {
		t.Fatalf("%v %v", got, err)
	}
	if !got[0].OccurrenceAt.Equal(key) || !got[0].FireAt.Equal(instant(t, "2026-10-03T09:00:00-03:00")) {
		t.Fatalf("all day %v", got[0])
	}
	subjects[0].CompletedAt = []time.Time{key.In(time.FixedZone("same instant", 7200))}
	got, err = Plan(PlanQuery{Now: now, Budget: 1}, subjects, nil, "08:30")
	if err != nil || len(got) != 1 || !got[0].FireAt.Equal(instant(t, "2026-10-04T08:30:00-03:00")) {
		t.Fatalf("completion %v %v", got, err)
	}
	subjects[0].CompletedAt = nil
	got, err = Plan(PlanQuery{Now: now.Add(2 * time.Hour), Budget: 1}, subjects, nil, "")
	if err != nil || !got[0].OccurrenceAt.Equal(key.AddDate(0, 0, 1)) {
		t.Fatalf("past all-day delivery %v %v", got, err)
	}
}

func TestPlanDelta(t *testing.T) {
	r := daily(t)
	subjects := []PlanSubject{{MemoryID: "m", Rule: r, Title: "Title", Note: strings.Repeat("a", 140) + "tail"}}
	q := PlanQuery{Now: r.Anchor, Budget: 2}
	before, err := Plan(q, subjects, nil, "")
	if err != nil {
		t.Fatal(err)
	}
	subjects[0].Note = strings.Repeat("a", 140) + "different tail"
	after, err := Plan(q, subjects, nil, "")
	if err != nil {
		t.Fatal(err)
	}
	if delta := DiffPlan(before, after); len(delta.Upsert) != 0 || len(delta.Remove) != 0 {
		t.Fatalf("tail edit causes changes: %v", delta)
	}
	subjects[0].Title = "Changed"
	after, err = Plan(q, subjects, nil, "")
	if err != nil {
		t.Fatal(err)
	}
	if delta := DiffPlan(before, after); len(delta.Upsert) != 2 || len(delta.Remove) != 0 {
		t.Fatalf("title edit %v", delta)
	}
	if delta := DiffPlan(before, nil); len(delta.Upsert) != 0 || len(delta.Remove) != 2 {
		t.Fatalf("cancel %v", delta)
	}
	before[0].FireAt = before[0].FireAt.In(location(t, "Asia/Kolkata"))
	after = slices.Clone(before)
	after[0].FireAt = after[0].FireAt.UTC()
	if delta := DiffPlan(before, after); len(delta.Upsert) != 0 {
		t.Fatal("equivalent instant must not replace")
	}
}

func TestShapeBody(t *testing.T) {
	for _, tc := range []struct{ note, want string }{
		{"# Heading\n\n**bold** _italic_ ~~old~~ `code`", "Heading bold italic old code"},
		{"- [x] Done\n1. [Link](https://example.com/a(b)) ![Photo](image.png)", "Done Link Photo"},
		{"<b>Hello</b> &amp; goodbye", "Hello & goodbye"},
		{"&lt;literal&gt; &nbsp; text", "<literal> text"},
		{strings.Repeat("&amp;", 10000), strings.Repeat("&", 140)},
		{"```go\nfmt.Println(1)\n```\nAfter", "fmt.Println(1) After"},
		{"Text ```inline```", "Text inline"},
		{"\\*literal\\* <https://example.com>", "*literal* https://example.com"},
		{"Hello [reference][id]", "Hello reference"},
		{strings.Repeat("🧠", 141), strings.Repeat("🧠", 140)},
	} {
		if got := ShapeBody(tc.note); got != tc.want || utf8.RuneCountInString(got) > 140 {
			t.Errorf("%q: got %q want %q", tc.note, got, tc.want)
		}
	}
}

func benchmarkSubjects(b *testing.B, frequency Freq) ([]PlanSubject, time.Time) {
	r := daily(b)
	r.Freq = frequency
	r.Anchor = instant(b, "2020-01-01T09:00:00Z")
	memories := make([]PlanSubject, 5000)
	for i := range memories {
		memories[i] = PlanSubject{MemoryID: fmt.Sprintf("m%05d", i), Rule: r, Title: "Memory", Note: "# Note\n" + strings.Repeat("Long note content ", 100)}
		memories[i].Rule.TZ = location(b, []string{"America/Sao_Paulo", "Europe/London", "Pacific/Auckland", "Asia/Kolkata"}[i%4])
	}
	return memories, instant(b, "2026-10-04T10:00:00Z")
}

func BenchmarkPlan5000(b *testing.B) {
	subjects, now := benchmarkSubjects(b, FreqDaily)
	b.ReportAllocs()
	for b.Loop() {
		if _, err := Plan(PlanQuery{Now: now}, subjects, nil, ""); err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkPlan5000Minutely(b *testing.B) {
	subjects, now := benchmarkSubjects(b, FreqMinutely)
	b.ReportAllocs()
	for b.Loop() {
		if _, err := Plan(PlanQuery{Now: now}, subjects, nil, ""); err != nil {
			b.Fatal(err)
		}
	}
}
