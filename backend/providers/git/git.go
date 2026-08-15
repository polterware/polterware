package git

import (
	"context"

	"github.com/polterware/polterware/backend/providers/process"
)

func Status(ctx context.Context, cwd string) (string, error) {
	res := process.Run(ctx, "git", []string{"status", "--porcelain"}, cwd)
	if res.Err != nil && res.ExitCode != 0 {
		return res.Stderr, res.Err
	}
	return res.Stdout, nil
}
