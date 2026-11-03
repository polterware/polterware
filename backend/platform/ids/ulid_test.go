package ids

import (
	"strings"
	"testing"
	"time"
)

func TestNewULIDShape(t *testing.T) {
	id := NewULID()
	if len(id) != 26 {
		t.Fatalf("len = %d, want 26: %q", len(id), id)
	}
	for _, r := range id {
		if !strings.ContainsRune(crockford, r) {
			t.Fatalf("%q contains %q outside the Crockford alphabet", id, r)
		}
	}
}

func TestNewULIDIsUnique(t *testing.T) {
	seen := map[string]bool{}
	for range 5000 {
		id := NewULID()
		if seen[id] {
			t.Fatalf("duplicate id %q", id)
		}
		seen[id] = true
	}
}

func TestULIDsSortByCreationTime(t *testing.T) {
	base := time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
	earlier := ulidAt(base)
	later := ulidAt(base.Add(time.Millisecond))
	muchLater := ulidAt(base.Add(48 * time.Hour))
	if !(earlier < later && later < muchLater) {
		t.Fatalf("ids must sort by time: %s %s %s", earlier, later, muchLater)
	}
}

func TestULIDTimestampPrefixIsStableWithinAMillisecond(t *testing.T) {
	at := time.Date(2026, 6, 1, 12, 0, 0, 0, time.UTC)
	if a, b := ulidAt(at)[:10], ulidAt(at)[:10]; a != b {
		t.Fatalf("timestamp prefix differs: %s vs %s", a, b)
	}
}
