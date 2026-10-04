package main

import (
	"context"
	"flag"
	"fmt"
	"os"

	"github.com/polterware/polterware/spikes/mcp/proxy"
)

func main() {
	endpoint := flag.String("url", os.Getenv("SPARKY_MCP_URL"), "streamable HTTP endpoint")
	token := flag.String("token", os.Getenv("SPARKY_MCP_TOKEN"), "bearer token")
	flag.Parse()
	if *endpoint == "" {
		fmt.Fprintln(os.Stderr, "stdio-proxy: -url is required")
		os.Exit(2)
	}
	if *token == "" {
		*token = "spike-token"
	}
	if err := proxy.Run(context.Background(), *endpoint, *token, os.Stdin, os.Stdout); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
