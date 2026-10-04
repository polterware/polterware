package schedule

import (
	"container/heap"
	"fmt"
	"sort"
	"strconv"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/clock"
)

const (
	DefaultBudget         = 60
	DefaultHorizon        = 30 * 24 * time.Hour
	DefaultPerMemoryLimit = 256
)

// Engine adapts the pure functions to the frozen Planner port. Calendar overdue
// state uses an injected clock; Plan always uses its explicit query.Now.
type Engine struct {
	Clock          clock.Clock
	PerMemoryLimit int
	AllDayTime     string
}

var _ Planner = Engine{}

func (e Engine) Plan(q PlanQuery, subjects []PlanSubject, oneOffs []PendingOneOff, allDayTime string) ([]ReminderItem, error) {
	return Plan(q, subjects, oneOffs, allDayTime)
}

func (e Engine) Preview(rule Rule, limit int) ([]time.Time, string, error) {
	return Preview(rule, limit)
}

func (e Engine) Occurrences(subjects []PlanSubject, from, to time.Time, includeCompleted bool, limit int) ([]Occurrence, error) {
	if e.Clock == nil {
		return nil, invalid("clock", "an injected clock is required for overdue state")
	}
	return ListOccurrences(subjects, OccurrenceQuery{From: from, To: to, Now: e.Clock.Now(), IncludeCompleted: includeCompleted,
		Limit: limit, PerMemoryLimit: e.PerMemoryLimit, AllDayTime: e.AllDayTime})
}

// Preview seeks only the requested prefix, without expanding a fixed horizon.
func Preview(rule Rule, limit int) ([]time.Time, string, error) {
	if limit == 0 {
		limit = 10
	}
	if limit < 1 || limit > MaxOccurrences {
		return nil, "", invalid("limit", "must be between 1 and 10000")
	}
	s, err := prepare(rule)
	if err != nil {
		return nil, "", err
	}
	c := cursor{s: s}
	items := make([]time.Time, 0, min(limit, 64))
	for len(items) < limit {
		t, ok := c.next()
		if !ok {
			break
		}
		items = append(items, t)
	}
	return items, Describe(rule, "en"), nil
}

func allDayClock(value string) (int, int, error) {
	if value == "" {
		value = "09:00"
	}
	if len(value) != 5 || value[2] != ':' {
		return 0, 0, invalid("all_day_time", "must be HH:MM")
	}
	h, eh := strconv.Atoi(value[:2])
	m, em := strconv.Atoi(value[3:])
	if eh != nil || em != nil || h < 0 || h > 23 || m < 0 || m > 59 {
		return 0, 0, invalid("all_day_time", "must be HH:MM")
	}
	for _, c := range value {
		if c != ':' && (c < '0' || c > '9') {
			return 0, 0, invalid("all_day_time", "must be HH:MM")
		}
	}
	return h, m, nil
}

func fireTime(at time.Time, rule Rule, hour, minute int) time.Time {
	if !rule.AllDay {
		return at
	}
	local := at.In(rule.TZ)
	y, m, d := local.Date()
	return resolveWall(time.Date(y, m, d, hour, minute, 0, 0, time.UTC), rule.TZ)
}

func completionSet(times []time.Time) map[time.Time]struct{} {
	if len(times) == 0 {
		return nil
	}
	set := make(map[time.Time]struct{}, len(times))
	for _, at := range times {
		set[at.UTC().Round(0)] = struct{}{}
	}
	return set
}

func completed(set map[time.Time]struct{}, at time.Time) bool {
	_, ok := set[at.UTC().Round(0)]
	return ok
}

type planCandidate struct {
	subject   int // -1 for a one-off
	one       int
	cursor    cursor
	completed map[time.Time]struct{}
	at, fire  time.Time
	id        string
}

type planHeap []planCandidate

func (h planHeap) Len() int { return len(h) }
func (h planHeap) Less(i, j int) bool {
	if h[i].fire.Equal(h[j].fire) {
		return h[i].id < h[j].id
	}
	return h[i].fire.Before(h[j].fire)
}
func (h planHeap) Swap(i, j int) { h[i], h[j] = h[j], h[i] }
func (h *planHeap) Push(v any)   { *h = append(*h, v.(planCandidate)) }
func (h *planHeap) Pop() any     { old := *h; v := old[len(old)-1]; *h = old[:len(old)-1]; return v }

func reminderID(memoryID string, at time.Time) string {
	return "rem:" + memoryID + ":" + strconv.FormatInt(at.UnixMilli(), 10)
}

func (c *planCandidate) advance(subject PlanSubject, now, until time.Time, hour, minute int) bool {
	for {
		at, ok := c.cursor.next()
		if !ok {
			return false
		}
		fire := fireTime(at, subject.Rule, hour, minute)
		if fire.After(until) {
			return false
		}
		if fire.Before(now) || completed(c.completed, at) {
			continue
		}
		c.at, c.fire, c.id = at, fire, reminderID(subject.MemoryID, at)
		return true
	}
}

// Plan merges one upcoming occurrence per memory and pending one-offs using a
// heap. Only a selected series advances; title/body shaping happens only for
// selected items. No database, wall clock, cache, or full-horizon expansion.
// Native-repeat transport is blocked by requests/w1-recurrence-1.md: the frozen
// ReminderItem RPC has no repeat field. This returns concrete one-shot items.
func Plan(q PlanQuery, subjects []PlanSubject, oneOffs []PendingOneOff, allDayTime string) ([]ReminderItem, error) {
	if q.Now.IsZero() {
		return nil, invalid("now", "an explicit instant is required")
	}
	if q.Budget == 0 {
		q.Budget = DefaultBudget
	}
	if q.Horizon == 0 {
		q.Horizon = DefaultHorizon
	}
	if q.Budget < 1 || q.Budget > DefaultBudget {
		return nil, invalid("budget", "must be between 1 and the hard ceiling 60")
	}
	if q.Horizon < 0 {
		return nil, invalid("horizon", "must be positive")
	}
	hour, minute, err := allDayClock(allDayTime)
	if err != nil {
		return nil, err
	}
	until := q.Now.Add(q.Horizon)
	if err := checkRange(q.Now, until); err != nil {
		return nil, err
	}
	candidates := make(planHeap, 0, len(subjects)+len(oneOffs))
	for i, subject := range subjects {
		if subject.ScheduleInactive || (subject.Status != "" && subject.Status != "active") {
			continue
		}
		s, err := prepare(subject.Rule)
		if err != nil {
			return nil, fmt.Errorf("memory %s: %w", subject.MemoryID, err)
		}
		if err := subject.Rule.checkRange(q.Now, until); err != nil {
			return nil, fmt.Errorf("memory %s: %w", subject.MemoryID, err)
		}
		from := q.Now
		if subject.Rule.AllDay {
			local := from.In(subject.Rule.TZ)
			from = resolveWall(time.Date(local.Year(), local.Month(), local.Day(), 0, 0, 0, 0, time.UTC), subject.Rule.TZ)
		}
		c := planCandidate{subject: i, cursor: cursor{s: s, k: s.seek(from)}, completed: completionSet(subject.CompletedAt)}
		if c.advance(subject, q.Now, until, hour, minute) {
			candidates = append(candidates, c)
		}
	}
	for i, one := range oneOffs {
		if one.Kind != "focus_phase" && one.Kind != "snooze" {
			return nil, invalid("one_off.kind", "must be focus_phase or snooze")
		}
		if one.FireAt.Before(q.Now) || one.FireAt.After(until) {
			continue
		}
		candidates = append(candidates, planCandidate{subject: -1, one: i, fire: one.FireAt.UTC(), at: one.FireAt.UTC(), id: "one:" + one.ID})
	}
	heap.Init(&candidates)
	items := make([]ReminderItem, 0, q.Budget)
	for len(items) < q.Budget && len(candidates) != 0 {
		c := heap.Pop(&candidates).(planCandidate)
		item := ReminderItem{ID: c.id, OccurrenceAt: c.at, FireAt: c.fire, Category: CategoryReminder}
		if c.subject < 0 {
			one := oneOffs[c.one]
			item.MemoryID, item.Title, item.Body = one.MemoryID, one.Title, ShapeBody(one.Body)
			if one.Kind == "focus_phase" {
				item.Category = CategoryFocusPhase
			}
		} else {
			subject := subjects[c.subject]
			item.MemoryID, item.Title, item.Body = subject.MemoryID, subject.Title, ShapeBody(subject.Note)
			if subject.FocusEnabled {
				item.Category = CategoryReminderFocus
			}
			if len(items)+1 < q.Budget && c.advance(subject, q.Now, until, hour, minute) {
				heap.Push(&candidates, c)
			}
		}
		items = append(items, item)
	}
	return items, nil
}

// PlanDelta is the minimal delivery update. Unchanged notification identifiers
// and content are preserved, so repeated calls do not restart OS triggers.
type PlanDelta struct {
	Upsert []ReminderItem
	Remove []string
}

func DiffPlan(previous, next []ReminderItem) PlanDelta {
	old := make(map[string]ReminderItem, len(previous))
	for _, item := range previous {
		old[item.ID] = item
	}
	delta := PlanDelta{}
	for _, item := range next {
		before, exists := old[item.ID]
		if !exists || before.MemoryID != item.MemoryID || !before.OccurrenceAt.Equal(item.OccurrenceAt) || !before.FireAt.Equal(item.FireAt) ||
			before.Title != item.Title || before.Body != item.Body || before.Category != item.Category {
			delta.Upsert = append(delta.Upsert, item)
		}
		delete(old, item.ID)
	}
	for id := range old {
		delta.Remove = append(delta.Remove, id)
	}
	sort.Strings(delta.Remove)
	return delta
}
