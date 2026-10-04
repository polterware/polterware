package sqlite

import (
	"sync"
	"testing"

	"github.com/polterware/polterware/sparky/backend/domains/activity"
	"github.com/polterware/polterware/sparky/backend/domains/actor"
	"github.com/polterware/polterware/sparky/backend/domains/fault"
	"github.com/polterware/polterware/sparky/backend/domains/memory"
	"github.com/polterware/polterware/sparky/backend/domains/mind"
	"github.com/polterware/polterware/sparky/backend/platform/clock"
	"github.com/polterware/polterware/sparky/backend/platform/ids"
)

func TestConcurrentToggleAndDeleteRev(t *testing.T) {
	store := openStore(t)
	ctx := t.Context()
	audit := activity.Audit{Repository: store.Activity(), Transactions: store, Clock: clock.Clock{}, IDs: ids.Generator{}}
	memories := memory.NewService(store.Memories(), store.Minds(), store.Tags(), nil, nil, audit)
	minds := mind.NewService(store.Minds(), store.Memories(), audit)
	who := actor.UI()

	created, err := memories.Create(ctx, who, memory.Create{Title: "Race", Checklist: []memory.CheckItemInput{{Title: "A"}}})
	if err != nil {
		t.Fatal(err)
	}
	items, err := store.Memories().ListCheckItems(ctx, created.ID)
	if err != nil || len(items) != 1 {
		t.Fatal(items, err)
	}

	errs := make(chan error, 2)
	var wg sync.WaitGroup
	wg.Add(2)
	for i := 0; i < 2; i++ {
		go func() {
			defer wg.Done()
			_, err := memories.ToggleCheckItem(ctx, who, created.ID, items[0].ID, 1, nil)
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
		t.Fatalf("toggle ok=%d conflict=%d", ok, conflict)
	}

	stale := created
	stale.Title = "Nope"
	stale.Rev = 2
	if err := store.Memories().Update(ctx, 99, stale); !fault.IsCode(err, fault.CodeConflict) {
		t.Fatalf("memory repo conflict: %v", err)
	}
	if err := store.Memories().Delete(ctx, 1, created.ID); !fault.IsCode(err, fault.CodeConflict) {
		t.Fatalf("delete stale: %v", err)
	}

	node, err := minds.Create(ctx, who, mind.Create{Name: "Race"})
	if err != nil {
		t.Fatal(err)
	}
	if err := store.Minds().Delete(ctx, 99, node.ID); !fault.IsCode(err, fault.CodeConflict) {
		t.Fatalf("mind delete stale: %v", err)
	}
	if _, err := minds.Delete(ctx, who, node.ID, 1); err != nil {
		t.Fatal(err)
	}
}
