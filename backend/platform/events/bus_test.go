package events

import (
	"slices"
	"testing"
	"time"
)

func TestBusCoalescesAndMapsTables(t *testing.T) {
	bus := NewBus()
	bus.window = 10 * time.Millisecond
	_, _, _, changes, cancel := bus.Subscribe(0)
	defer cancel()

	bus.Record([]string{"work_cards", "change_ops"}, []string{"proj-a"}, []string{"cli"})
	bus.Record([]string{"projects", "store_health"}, []string{"proj-b"}, []string{"desktop"})
	bus.Record([]string{"store_health"}, nil, []string{"core"})

	select {
	case change := <-changes:
		if change.Seq != 1 {
			t.Fatalf("seq = %d", change.Seq)
		}
		if !slices.Equal(change.Kinds, []string{KindProjects, KindWork}) {
			t.Errorf("kinds = %v", change.Kinds)
		}
		if !slices.Equal(change.ProjectIDs, []string{"proj-a", "proj-b"}) {
			t.Errorf("projects = %v", change.ProjectIDs)
		}
		if !slices.Equal(change.Origins, []string{"cli", "desktop"}) {
			t.Errorf("origins = %v (unmapped-only transactions must not add origins)", change.Origins)
		}
	case <-time.After(time.Second):
		t.Fatal("no change event")
	}
}

func TestBusSubscribeBacklogAndResync(t *testing.T) {
	bus := NewBus()
	for i := 0; i < 3; i++ {
		bus.Record([]string{"work_cards"}, nil, nil)
		bus.flush()
	}
	current, backlog, resync, _, cancel := bus.Subscribe(1)
	cancel()
	if current != 3 || resync || len(backlog) != 2 || backlog[0].Seq != 2 {
		t.Fatalf("current=%d resync=%v backlog=%+v", current, resync, backlog)
	}
	// A seq from a previous core is ahead of this one.
	_, _, resync, _, cancel = bus.Subscribe(99)
	cancel()
	if !resync {
		t.Fatal("expected resync for a sequence this core never issued")
	}
}
