package core

import (
	"context"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/activity"
	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/clock"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/domains/schedule"
	"github.com/polterware/polterware/sparky/backend/domains/settings"
)

type scheduleRuntime struct {
	clock     clock.Clock
	audit     activity.Audit
	memories  memory.Repository
	minds     mind.Repository
	schedules schedule.Repository
	planner   schedule.Planner
	settings  settings.Service
	complete  memory.Service
}

func loadZone(name string) (*time.Location, error) {
	if name == "" {
		return time.UTC, nil
	}
	loc, err := time.LoadLocation(name)
	if err != nil {
		return nil, invalid("tz", "invalid IANA time zone")
	}
	return loc, nil
}

func (s scheduleRuntime) rule(row schedule.Schedule) (schedule.Rule, error) {
	loc, err := loadZone(row.TZ)
	if err != nil {
		return schedule.Rule{}, err
	}
	return row.Rule(loc)
}

func (s scheduleRuntime) allDay(ctx context.Context) (string, error) {
	doc, err := s.settings.Get(ctx)
	if err != nil {
		return "", err
	}
	if doc.NotificationsAllDayTime == "" {
		return settings.DefaultAllDayTime, nil
	}
	return doc.NotificationsAllDayTime, nil
}

func descendantIDs(nodes []mind.Mind, root string) []string {
	children := map[string][]string{}
	for _, node := range nodes {
		if node.ParentID != nil {
			children[*node.ParentID] = append(children[*node.ParentID], node.ID)
		}
	}
	out := []string{}
	var walk func(string)
	walk = func(id string) {
		for _, child := range children[id] {
			out = append(out, child)
			walk(child)
		}
	}
	walk(root)
	return out
}

func (s scheduleRuntime) subjects(ctx context.Context) ([]schedule.PlanSubject, error) {
	rows, err := s.schedules.ListActive(ctx)
	if err != nil {
		return nil, err
	}
	out := make([]schedule.PlanSubject, 0, len(rows))
	for _, row := range rows {
		m, err := s.memories.Get(ctx, row.MemoryID)
		if err != nil {
			if fault.IsCode(err, fault.CodeNotFound) {
				continue
			}
			return nil, err
		}
		rule, err := s.rule(row)
		if err != nil {
			return nil, err
		}
		completions, err := s.memories.ListCompletions(ctx, row.MemoryID)
		if err != nil {
			return nil, err
		}
		tags, err := s.memories.ListTagIDs(ctx, row.MemoryID)
		if err != nil {
			return nil, err
		}
		done := make([]time.Time, 0, len(completions))
		for _, c := range completions {
			done = append(done, c.OccurrenceAt)
		}
		out = append(out, schedule.PlanSubject{
			MemoryID: m.ID, Title: m.Title, Note: m.Note, Rule: rule, CompletedAt: done,
			FocusEnabled: row.FocusEnabled, MindID: m.MindID, TagIDs: tags, Status: m.Status,
			ScheduleInactive: !row.Active,
		})
	}
	return out, nil
}

func (s scheduleRuntime) Set(ctx context.Context, who actor.Actor, in schedule.Schedule) (schedule.Schedule, error) {
	if in.MemoryID == "" {
		return schedule.Schedule{}, invalid("schedule.memory_id", "required")
	}
	if _, err := s.memories.Get(ctx, in.MemoryID); err != nil {
		return schedule.Schedule{}, err
	}
	if in.Every < 1 {
		in.Every = 1
	}
	if in.EndType == "" {
		in.EndType = schedule.EndNever
	}
	if _, err := s.rule(in); err != nil {
		return schedule.Schedule{}, err
	}
	in.FireAt = in.FireAt.UTC().Truncate(time.Millisecond)
	in.UpdatedAt = s.clock.Now().UTC()
	err := s.audit.Write(ctx, who, "schedule", in.MemoryID, activity.OpUpdate, "Set schedule", func(ctx context.Context) error {
		return s.schedules.Upsert(ctx, in)
	})
	if err != nil {
		return schedule.Schedule{}, err
	}
	return in, nil
}

func (s scheduleRuntime) Clear(ctx context.Context, who actor.Actor, memoryID string) error {
	if memoryID == "" {
		return invalid("memory_id", "required")
	}
	return s.audit.Write(ctx, who, "schedule", memoryID, activity.OpDelete, "Clear schedule", func(ctx context.Context) error {
		return s.schedules.Delete(ctx, memoryID)
	})
}

func (s scheduleRuntime) Preview(ctx context.Context, in schedule.Schedule, limit int) ([]time.Time, string, error) {
	rule, err := s.rule(in)
	if err != nil {
		return nil, "", err
	}
	return s.planner.Preview(rule, limit)
}

func (s scheduleRuntime) ListOccurrences(ctx context.Context, q OccurrenceQuery) ([]schedule.Occurrence, error) {
	subjects, err := s.subjects(ctx)
	if err != nil {
		return nil, err
	}
	allDay, err := s.allDay(ctx)
	if err != nil {
		return nil, err
	}
	query := schedule.OccurrenceQuery{
		From: q.From, To: q.To, Now: s.clock.Now().UTC(), MindID: q.MindID, TagIDs: q.TagIDs,
		IncludeCompleted: q.IncludeCompleted, Limit: q.Limit, AllDayTime: allDay,
	}
	if q.MindID != nil && q.IncludeDescendants {
		nodes, err := s.minds.List(ctx)
		if err != nil {
			return nil, err
		}
		query.MindIDs = descendantIDs(nodes, *q.MindID)
	}
	return schedule.ListOccurrences(subjects, query)
}

func (s scheduleRuntime) CompleteOccurrence(ctx context.Context, who actor.Actor, memoryID string, at time.Time) (memory.Completion, error) {
	if _, err := s.complete.SetStatus(ctx, who, memoryID, 0, memory.StatusCompleted, &at); err != nil {
		return memory.Completion{}, err
	}
	items, err := s.memories.ListCompletions(ctx, memoryID)
	if err != nil {
		return memory.Completion{}, err
	}
	for _, item := range items {
		if item.OccurrenceAt.Equal(at.UTC()) {
			return item, nil
		}
	}
	return memory.Completion{MemoryID: memoryID, OccurrenceAt: at.UTC()}, nil
}

func (s scheduleRuntime) ReopenOccurrence(ctx context.Context, who actor.Actor, memoryID string, at time.Time) error {
	_, err := s.complete.SetStatus(ctx, who, memoryID, 0, memory.StatusActive, &at)
	return err
}

func (s scheduleRuntime) SetFocusRecipe(ctx context.Context, who actor.Actor, memoryID string, recipe schedule.FocusRecipe) (schedule.Schedule, error) {
	row, err := s.schedules.Get(ctx, memoryID)
	if err != nil {
		return schedule.Schedule{}, err
	}
	if recipe.Enabled {
		for field, value := range map[string]int{"work_min": recipe.WorkMin, "short_break_min": recipe.ShortBreakMin, "long_break_min": recipe.LongBreakMin} {
			if value < 1 || value > 60 {
				return schedule.Schedule{}, invalid(field, "must be between 1 and 60")
			}
		}
		if recipe.Cycles < 1 || recipe.Cycles > 12 {
			return schedule.Schedule{}, invalid("cycles", "must be between 1 and 12")
		}
	}
	row.FocusEnabled = recipe.Enabled
	row.FocusWorkMin = intPtrFrom(recipe.WorkMin)
	row.FocusShortBreakMin = intPtrFrom(recipe.ShortBreakMin)
	row.FocusLongBreakMin = intPtrFrom(recipe.LongBreakMin)
	row.FocusCycles = intPtrFrom(recipe.Cycles)
	row.FocusAutoContinue = recipe.AutoContinue
	row.UpdatedAt = s.clock.Now().UTC()
	err = s.audit.Write(ctx, who, "schedule", memoryID, activity.OpUpdate, "Set focus recipe", func(ctx context.Context) error {
		return s.schedules.Upsert(ctx, row)
	})
	if err != nil {
		return schedule.Schedule{}, err
	}
	return row, nil
}

func intPtrFrom(n int) *int {
	if n == 0 {
		return nil
	}
	v := n
	return &v
}
