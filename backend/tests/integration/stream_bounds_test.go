package integration

import (
	"strings"
	"testing"

	"github.com/polterware/polterware/backend/domains/logs"
)

func TestStreamBounds(t *testing.T) {
	b := logs.NewBuffer(3)
	for i := 0; i < 10; i++ {
		b.Append("line")
	}
	if b.Len() != 3 || b.Dropped() == 0 {
		t.Fatal("bounded drop-oldest")
	}
	q := logs.NewEventQueue(2)
	if !q.TrySend("a") || !q.TrySend("b") || q.TrySend("c") {
		// full returns false
	}
	if q.TrySend("c") {
		t.Fatal("resource_exhausted expected")
	}
	long := strings.Repeat("x", logs.DefaultMaxLineBytes+5)
	if !b.Append(long) {
		t.Fatal("expect truncate")
	}
}
