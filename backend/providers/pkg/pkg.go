package pkg

import (
	"context"

	"github.com/polterware/polterware/backend/providers/process"
)

func RunScript(ctx context.Context, cwd, script string) (string, error) {
	// Prefer pnpm run when available
	res := process.Run(ctx, "pnpm", []string{"run", script}, cwd)
	if res.Err != nil {
		return res.Stderr, res.Err
	}
	return res.Stdout, nil
}
