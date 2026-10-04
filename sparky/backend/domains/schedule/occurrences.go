package schedule

import (
	"container/heap"
	"fmt"
	"slices"
	"sort"
	"time"
)

// OccurrenceQuery is a pure calendar query. MindIDs is the already-resolved
// descendant scope (the core owns hierarchy traversal); TagIDs uses AND matching.
// PerMemoryLimit defaults to 256 and caps examined occurrences per memory before
// completion filtering. Dense series return their earliest prefix in the range,
// never an eager full expansion. Narrow the range to see later occurrences.
type OccurrenceQuery struct {
	From, To, Now    time.Time
	MindID           *string
	MindIDs          []string
	TagIDs           []string
	IncludeCompleted bool
	Limit            int
	PerMemoryLimit   int
	AllDayTime       string
}

func matches(subject PlanSubject, q OccurrenceQuery) bool {
	if subject.ScheduleInactive {
		return false
	}
	if q.MindID != nil || len(q.MindIDs) != 0 {
		if subject.MindID == nil {
			return false
		}
		if (q.MindID == nil || *subject.MindID != *q.MindID) && !slices.Contains(q.MindIDs, *subject.MindID) {
			return false
		}
	}
	for _, id := range q.TagIDs {
		if !slices.Contains(subject.TagIDs, id) {
			return false
		}
	}
	return q.IncludeCompleted || subject.Status != "completed"
}

func occurrenceLess(a, b Occurrence) bool {
	if a.OccurrenceAt.Equal(b.OccurrenceAt) {
		return a.MemoryID < b.MemoryID
	}
	return a.OccurrenceAt.Before(b.OccurrenceAt)
}

// A max heap retains the earliest global prefix with O(log Limit) insertion.
type occurrenceHeap []Occurrence

func (h occurrenceHeap) Len() int           { return len(h) }
func (h occurrenceHeap) Less(i, j int) bool { return occurrenceLess(h[j], h[i]) }
func (h occurrenceHeap) Swap(i, j int)      { h[i], h[j] = h[j], h[i] }
func (h *occurrenceHeap) Push(v any)        { *h = append(*h, v.(Occurrence)) }
func (h *occurrenceHeap) Pop() any {
	old := *h
	item := old[len(old)-1]
	*h = old[:len(old)-1]
	return item
}

// ListOccurrences returns globally ordered occurrences in [From,To), completion
// keyed by exact instants, and overdue derived from explicit Now and FireAt.
// Its per-memory expansion cap is separate from the global returned-item limit.
func ListOccurrences(subjects []PlanSubject, q OccurrenceQuery) ([]Occurrence, error) {
	if q.Now.IsZero() {
		return nil, invalid("now", "an explicit instant is required for overdue state")
	}
	if err := checkRange(q.From, q.To); err != nil {
		return nil, err
	}
	if q.PerMemoryLimit == 0 {
		q.PerMemoryLimit = DefaultPerMemoryLimit
	}
	if q.Limit == 0 {
		q.Limit = MaxOccurrences
	}
	if q.PerMemoryLimit < 1 || q.PerMemoryLimit > MaxOccurrences {
		return nil, invalid("per_memory_limit", "must be between 1 and 10000")
	}
	if q.Limit < 1 || q.Limit > MaxOccurrences {
		return nil, invalid("limit", "must be between 1 and 10000")
	}
	hour, minute, err := allDayClock(q.AllDayTime)
	if err != nil {
		return nil, err
	}
	// Keep at most the global limit, rather than allocating memories*cap results.
	items := make(occurrenceHeap, 0, min(q.Limit, 64))
	for _, subject := range subjects {
		if !matches(subject, q) {
			continue
		}
		s, err := prepare(subject.Rule)
		if err != nil {
			return nil, fmt.Errorf("memory %s: %w", subject.MemoryID, err)
		}
		if err := subject.Rule.checkRange(q.From, q.To); err != nil {
			return nil, fmt.Errorf("memory %s: %w", subject.MemoryID, err)
		}
		c := cursor{s: s, k: s.seek(q.From)}
		completions := completionSet(subject.CompletedAt)
		for examined := 0; examined < q.PerMemoryLimit; examined++ {
			at, ok := c.next()
			if !ok || !at.Before(q.To) {
				break
			}
			done := completed(completions, at) || subject.Status == "completed"
			if done && !q.IncludeCompleted {
				continue
			}
			fire := fireTime(at, subject.Rule, hour, minute)
			item := Occurrence{MemoryID: subject.MemoryID, Title: subject.Title, OccurrenceAt: at, FireAt: fire,
				Completed: done, Overdue: !done && fire.Before(q.Now), AllDay: subject.Rule.AllDay, MindID: subject.MindID, TZ: subject.Rule.TZ.String()}
			if len(items) < q.Limit {
				heap.Push(&items, item)
			} else {
				if !occurrenceLess(item, items[0]) {
					break
				}
				items[0] = item
				heap.Fix(&items, 0)
			}
		}
	}
	sort.Slice(items, func(i, j int) bool { return occurrenceLess(items[i], items[j]) })
	return items, nil
}
