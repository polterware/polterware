//go:build !linux

package main

import (
	"context"
	"fmt"
	"io"
)

func runAgent(ctx context.Context, in io.Reader, out io.Writer) error {
	return fmt.Errorf("vitals-agent supports linux only")
}
