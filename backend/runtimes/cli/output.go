package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"

	"connectrpc.com/connect"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/proto"
)

func writeJSON(v any) error {
	enc := json.NewEncoder(os.Stdout)
	enc.SetIndent("", "  ")
	return enc.Encode(v)
}

// writeProtoJSON prints a core response with snake_case field names.
func writeProtoJSON(message proto.Message) error {
	body, err := protojson.MarshalOptions{Multiline: true, Indent: "  ", UseProtoNames: true, EmitUnpopulated: true}.Marshal(message)
	if err != nil {
		return err
	}
	_, err = fmt.Fprintf(os.Stdout, "%s\n", body)
	return err
}

// exitError ends the process with code after the command already printed its
// own output (for example lint diagnostics).
type exitError struct{ code int }

func (e exitError) Error() string { return fmt.Sprintf("exit status %d", e.code) }

func fail(err error) {
	var exit exitError
	if errors.As(err, &exit) {
		os.Exit(exit.code)
	}
	// Core errors carry a Connect code prefix; the message alone is the fix.
	var connectErr *connect.Error
	if errors.As(err, &connectErr) {
		fmt.Fprintf(os.Stderr, "polterware: %s\n", connectErr.Message())
		os.Exit(1)
	}
	fmt.Fprintf(os.Stderr, "polterware: %v\n", err)
	os.Exit(1)
}

func usage() {
	fmt.Fprintf(os.Stderr, `Usage:
  polterware [-db path]                (interactive status and MCP setup; requires a terminal)
  polterware [-db path] init --name <name> [--path <root>] [--id <id>] [--json]
  polterware [-db path] scan --project <id> [--json]
  polterware [-db path] repo list --project <id> [--json]
  polterware [-db path] repo add --project <id> --name <name> --path <dir> [--json]
  polterware [-db path] repo remove --project <id> --name <name>
  polterware [-db path] context --project <id> [--query ...] [--entity ...] [--budget N] [--agent-run-id ...] [--json]
  polterware [-db path] drift --project <id> [--entity ...] [--agent-run-id ...] [--json]
  polterware [-db path] apply <file|-> [--dry-run] [--project <id>] [--summary ...] [--rationale ...] [--agent-run-id ...]
  polterware [-db path] lint <file|dir|-> [--project <id>]   (silent on success)
  polterware [-db path] format                               (print the Polter format spec)
  polterware [-db path] query "cards" | "card <slug>" | "status" | "handoff <slug>" [--project <id>]
  polterware [-db path] status [--project <id>]
  polterware [-db path] handoff [--card <slug>] [--note "..."] [--project <id>]
  polterware [-db path] hook session-start [--label name] | session-end | lint | status | stop   (Claude Code hooks; JSON on stdin)
  polterware [-db path] serve [--listen addr] [--token token]   (local core server for the Desktop)
  polterware [-db path] mcp                                     (MCP server over stdio for coding agents)
`)
}
