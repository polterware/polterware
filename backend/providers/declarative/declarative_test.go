package declarative

import (
	"context"
	"testing"
)

func TestPlanGit(t *testing.T) {
	r := Plan(context.Background(), "git")
	if !r.Present {
		t.Fatal(r.Message)
	}
}
