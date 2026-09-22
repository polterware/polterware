package git

import (
	"context"

	execpkg "github.com/polterware/polterware/backend/providers/exec"
)

func Status(ctx context.Context, cwd string) (string, error) {
	res := execpkg.Run(ctx, "git", []string{"status", "--porcelain"}, cwd)
	if res.Err != nil && res.ExitCode != 0 {
		return res.Stderr, res.Err
	}
	return res.Stdout, nil
}
