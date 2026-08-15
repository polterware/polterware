package declarative

import (
	"context"
	"os/exec"
)

type PlanResult struct {
	Present bool
	Message string
}

func Plan(ctx context.Context, tool string) PlanResult {
	_, err := exec.LookPath(tool)
	if err != nil {
		return PlanResult{Present: false, Message: "tool not found: " + tool}
	}
	return PlanResult{Present: true, Message: "tool present; plan-only"}
}
