package main

import (
	"encoding/json"
	"fmt"
	"os"
)

func writeJSON(v any) error {
	enc := json.NewEncoder(os.Stdout)
	enc.SetIndent("", "  ")
	return enc.Encode(v)
}

func fail(err error) {
	fmt.Fprintf(os.Stderr, "polterware: %v\n", err)
	os.Exit(1)
}

func usage() {
	fmt.Fprintf(os.Stderr, `Usage:
  polterware [-db path]                (interactive status and MCP setup; requires a terminal)
  polterware [-db path] init --name <name> [--path <root>] [--id <id>] [--json]
  polterware [-db path] scan --project <id> [--json]
  polterware [-db path] context --project <id> [--query ...] [--entity ...] [--budget N] [--agent-run-id ...] [--json]
  polterware [-db path] drift --project <id> [--entity ...] [--agent-run-id ...] [--json]
  polterware [-db path] serve [--listen addr] [--token token]   (local core server for the Desktop)
  polterware [-db path] mcp                                     (MCP server over stdio for coding agents)
`)
}
