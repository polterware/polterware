package main

import (
	"context"
	"flag"
	"fmt"
	"os"

	"github.com/polterware/polterware/spikes/mcp/internal/guard"
	"github.com/polterware/polterware/spikes/mcp/internal/mem"
	"github.com/polterware/polterware/spikes/mcp/sdkserver"
)

func main() {
	addr := flag.String("addr", "127.0.0.1:47610", "loopback listen address")
	token := flag.String("token", os.Getenv("SPARKY_MCP_TOKEN"), "bearer token")
	flag.Parse()
	if *token == "" {
		*token = "spike-token"
	}
	store := mem.New()
	handler := sdkserver.New(store).Handler()
	fmt.Fprintf(os.Stderr, "sdk mcp http://%s/mcp\n", *addr)
	if err := guard.Serve(context.Background(), *addr, *token, handler); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
