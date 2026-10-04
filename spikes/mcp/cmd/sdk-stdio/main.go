package main

import (
	"context"
	"fmt"
	"os"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/polterware/polterware/spikes/mcp/internal/mem"
	"github.com/polterware/polterware/spikes/mcp/sdkserver"
)

func main() {
	server := sdkserver.New(mem.New())
	if err := server.MCP().Run(context.Background(), &mcp.StdioTransport{}); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
