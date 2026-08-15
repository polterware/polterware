package observability

import "testing"

func TestCorrelationID(t *testing.T) {
	a, b := NewCorrelationID(), NewCorrelationID()
	if a == "" || a == b {
		t.Fatal(a, b)
	}
}
