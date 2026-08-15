package operations

import "testing"

func TestDirectStateMachine(t *testing.T) {
	op := &Operation{ID: "op", Status: StatusQueued}
	if err := op.Transition(StatusRunning); err != nil {
		t.Fatal(err)
	}
	if err := op.Transition(StatusSucceeded); err != nil {
		t.Fatal(err)
	}
	if err := op.Transition(StatusFailed); err == nil {
		t.Fatal("expected terminal error")
	}
}
