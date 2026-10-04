package mcp

import (
	"context"
	"sync"
	"testing"
	"time"

	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
)

// Only the use cases exercised here are implemented. Embedded unused methods
// are deliberately unavailable; this is not a replacement domain service.
type revisionService struct {
	memory.Service
	mu           sync.Mutex
	current      memory.Memory
	lastActor    actor.Actor
	lastExpected int64
}

func (f *revisionService) Get(context.Context, string, memory.Include) (memory.GetResult, error) {
	f.mu.Lock()
	defer f.mu.Unlock()
	return memory.GetResult{Memory: f.current}, nil
}

func (f *revisionService) List(context.Context, memory.ListFilter) (memory.ListPage, error) {
	f.mu.Lock()
	defer f.mu.Unlock()
	return memory.ListPage{Total: 1, Memories: []memory.Summary{{ID: f.current.ID, Rev: f.current.Rev}}}, nil
}

func (f *revisionService) Update(_ context.Context, who actor.Actor, id string, expected int64, p memory.Patch) (memory.Memory, error) {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.lastActor, f.lastExpected = who, expected
	if expected != 0 && expected != f.current.Rev {
		return memory.Memory{}, fault.Conflict("memory", id, f.current.Rev, f.current)
	}
	if p.Title != nil {
		f.current.Title = *p.Title
	}
	f.current.Rev++
	return f.current, nil
}

func (f *revisionService) SetStatus(ctx context.Context, who actor.Actor, id string, expected int64, status string, _ *time.Time) (memory.Memory, error) {
	m, err := f.Update(ctx, who, id, expected, memory.Patch{})
	if err == nil {
		m.Status = status
	}
	return m, err
}

func (f *revisionService) ToggleCheckItem(ctx context.Context, who actor.Actor, id, itemID string, expected int64, _ *bool) (memory.CheckItem, error) {
	if _, err := f.Update(ctx, who, id, expected, memory.Patch{}); err != nil {
		return memory.CheckItem{}, err
	}
	return memory.CheckItem{ID: itemID, MemoryID: id, Title: "A", Done: true}, nil
}

func (f *revisionService) Delete(_ context.Context, who actor.Actor, id string, expected int64) error {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.lastActor, f.lastExpected = who, expected
	if expected != 0 && expected != f.current.Rev {
		return fault.Conflict("memory", id, f.current.Rev, f.current)
	}
	f.current = memory.Memory{}
	return nil
}

func TestSessionRejectsInterveningUIWrite(t *testing.T) {
	for _, read := range []string{"get", "list"} {
		for _, write := range []string{"update", "status"} {
			t.Run(read+"/"+write, func(t *testing.T) {
				ctx := context.Background()
				service := &revisionService{current: memory.Memory{ID: "memory-1", Title: "Before", Rev: 1}}
				s := NewMemorySession(service, "Claude Code")
				var err error
				if read == "get" {
					_, err = s.Get(ctx, "memory-1", memory.Include{})
				} else {
					_, err = s.List(ctx, memory.ListFilter{})
				}
				if err != nil {
					t.Fatal(err)
				}
				title := "UI edit"
				if _, err := service.Update(ctx, actor.UI(), "memory-1", 1, memory.Patch{Title: &title}); err != nil {
					t.Fatal(err)
				}
				agentTitle := "Agent edit"
				if write == "update" {
					_, err = s.Update(ctx, "memory-1", nil, memory.Patch{Title: &agentTitle})
				} else {
					_, err = s.SetStatus(ctx, "memory-1", nil, memory.StatusCompleted, nil)
				}
				if !fault.IsCode(err, fault.CodeConflict) {
					t.Fatalf("expected conflict, got %v", err)
				}
				current, _ := service.Get(ctx, "memory-1", memory.Include{})
				if current.Memory.Title != "UI edit" || current.Memory.Rev != 2 {
					t.Fatal("UI write overwritten")
				}
				if service.lastActor.Raw != "mcp:Claude Code" || service.lastExpected != 1 {
					t.Fatal("actor/revision not delegated")
				}
			})
		}
	}
}

func TestSessionFirstWriteExplicitRevAndIsolation(t *testing.T) {
	ctx := context.Background()
	service := &revisionService{current: memory.Memory{ID: "memory-1", Rev: 5}}
	a := NewMemorySession(service, "same-client")
	b := NewMemorySession(service, "same-client")
	if _, err := a.Get(ctx, "memory-1", memory.Include{}); err != nil {
		t.Fatal(err)
	}
	if _, err := b.Update(ctx, "memory-1", nil, memory.Patch{}); err != nil {
		t.Fatal(err)
	}
	if service.lastExpected != 0 {
		t.Fatal("revision leaked between sessions")
	}
	if _, err := a.Update(ctx, "memory-1", nil, memory.Patch{}); !fault.IsCode(err, fault.CodeConflict) {
		t.Fatal("stale session overwrote")
	}
	explicit := int64(6)
	if _, err := a.Update(ctx, "memory-1", &explicit, memory.Patch{}); err != nil {
		t.Fatal(err)
	}
	if service.lastExpected != 6 {
		t.Fatal("explicit revision ignored")
	}
	if _, err := a.Update(ctx, "memory-1", nil, memory.Patch{}); err != nil {
		t.Fatal(err)
	}
	if service.lastExpected != 7 {
		t.Fatal("returned revision not observed")
	}
}

func TestSessionDeleteAndToggleHonorReadRev(t *testing.T) {
	ctx := context.Background()
	service := &revisionService{current: memory.Memory{ID: "memory-1", Title: "Before", Rev: 1}}
	s := NewMemorySession(service, "Claude Code")
	if _, err := s.Get(ctx, "memory-1", memory.Include{}); err != nil {
		t.Fatal(err)
	}
	title := "UI edit"
	if _, err := service.Update(ctx, actor.UI(), "memory-1", 1, memory.Patch{Title: &title}); err != nil {
		t.Fatal(err)
	}
	if _, err := s.ToggleCheckItem(ctx, "memory-1", "item-1", nil, nil); !fault.IsCode(err, fault.CodeConflict) {
		t.Fatalf("toggle: %v", err)
	}
	if err := s.Delete(ctx, "memory-1", nil); !fault.IsCode(err, fault.CodeConflict) {
		t.Fatalf("delete: %v", err)
	}
	explicit := int64(2)
	if _, err := s.ToggleCheckItem(ctx, "memory-1", "item-1", &explicit, nil); err != nil {
		t.Fatal(err)
	}
	if service.lastExpected != 2 {
		t.Fatal("explicit toggle revision ignored")
	}
}

func TestSessionConcurrentObservations(t *testing.T) {
	s := NewMemorySession(nil, "test")
	var wg sync.WaitGroup
	for i := 1; i <= 100; i++ {
		wg.Add(1)
		go func(rev int64) { defer wg.Done(); s.observe("memory-1", rev); _ = s.expected("memory-1", nil) }(int64(i))
	}
	wg.Wait()
	if got := s.expected("memory-1", nil); got != 100 {
		t.Fatalf("revision regressed: %d", got)
	}
}

func TestSessionRejectsExplicitNonpositiveRevision(t *testing.T) {
	for _, rev := range []int64{0, -1} {
		service := &revisionService{current: memory.Memory{ID: "m", Rev: 1}}
		sess := NewMemorySession(service, "test")
		if _, err := sess.Update(t.Context(), "m", &rev, memory.Patch{}); !fault.IsCode(err, fault.CodeInvalid) {
			t.Fatalf("update %d: %v", rev, err)
		}
		if _, err := sess.SetStatus(t.Context(), "m", &rev, "completed", nil); !fault.IsCode(err, fault.CodeInvalid) {
			t.Fatalf("status %d: %v", rev, err)
		}
		if _, err := sess.ToggleCheckItem(t.Context(), "m", "i", &rev, nil); !fault.IsCode(err, fault.CodeInvalid) {
			t.Fatalf("toggle %d: %v", rev, err)
		}
		if err := sess.Delete(t.Context(), "m", &rev); !fault.IsCode(err, fault.CodeInvalid) {
			t.Fatalf("delete %d: %v", rev, err)
		}
		if service.current.Rev != 1 {
			t.Fatal("invalid revision mutated memory")
		}
	}
}

type interveningToggle struct{ *revisionService }

func (f interveningToggle) ToggleCheckItem(ctx context.Context, who actor.Actor, id, itemID string, rev int64, done *bool) (memory.CheckItem, error) {
	item, err := f.revisionService.ToggleCheckItem(ctx, who, id, itemID, rev, done)
	if err != nil {
		return item, err
	}
	// Force the exact race from the review before the MCP service returns.
	title := "UI edit after toggle"
	_, err = f.revisionService.Update(ctx, actor.UI(), id, rev+1, memory.Patch{Title: &title})
	return item, err
}
func TestToggleReturnsOwnRevisionWithoutObservingLaterUIWrite(t *testing.T) {
	service := &revisionService{current: memory.Memory{ID: "m", Rev: 1}}
	sess := NewMemorySession(interveningToggle{service}, "test")
	_, rev, err := sess.ToggleWithRevision(t.Context(), "m", "i", nil, nil)
	if err != nil {
		t.Fatal(err)
	}
	if rev != 2 || sess.expected("m", nil) != 2 {
		t.Fatalf("returned %d, observed %d", rev, sess.expected("m", nil))
	}
	if _, err := sess.Update(t.Context(), "m", nil, memory.Patch{}); !fault.IsCode(err, fault.CodeConflict) {
		t.Fatalf("UI write was not protected: %v", err)
	}
}
