package main

import (
	"context"
	"fmt"
	"os"

	"github.com/polterware/polterware/spikes/mcp/handserver"
	"github.com/polterware/polterware/spikes/mcp/internal/mem"
)

func main() {
	server := handserver.New(mem.New())
	if err := server.ServeStdio(context.Background(), os.Stdin, os.Stdout); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
