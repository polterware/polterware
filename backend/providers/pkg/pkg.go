package pkg

import (
	"context"

	execpkg "github.com/polterware/polterware/backend/providers/exec"
)

func RunScript(ctx context.Context, cwd, script string) (string, error) {
	res := execpkg.Run(ctx, "pnpm", []string{"run", script}, cwd)
	if res.Err != nil {
		return res.Stderr, res.Err
	}
	return res.Stdout, nil
}
