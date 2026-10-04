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
