package main

import (
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"os"

	"github.com/polterware/polterware/backend/platform/coreclient"
	"github.com/polterware/polterware/backend/providers/agentsetup"
)

const setupMcpUsage = "usage: polterware setup mcp [list] [--json] | add <claude|codex|cursor|all> | remove <claude|codex|cursor|all>"

// mcpClientRow is one client in `setup mcp list --json`, which the Desktop reads.
type mcpClientRow struct {
	ID     string `json:"id"`
	Name   string `json:"name"`
	Status string `json:"status"`
	Error  string `json:"error,omitempty"`
}

// setupMcp registers this binary's MCP server (`polterware -db <db> mcp`) with Claude Code,
// Codex, and Cursor, removes it, or lists where it stands.
func setupMcp(dbPath string, args []string) error {
	action := "list"
	if len(args) > 0 && args[0] != "--json" {
		action, args = args[0], args[1:]
	}
	core, err := coreclient.Executable()
	if err != nil {
		return err
	}
	switch action {
	case "list":
		flags := flag.NewFlagSet("setup mcp list", flag.ContinueOnError)
		flags.SetOutput(os.Stderr)
		asJSON := flags.Bool("json", false, "print JSON")
		if err := flags.Parse(args); err != nil {
			return err
		}
		return listMcpClients(core, dbPath, *asJSON)
	case "add", "remove":
		if len(args) != 1 {
			return errors.New(setupMcpUsage)
		}
		clients, err := pickMcpClients(args[0])
		if err != nil {
			return err
		}
		for _, client := range clients {
			if action == "add" {
				if err := agentsetup.ConfigureMcp(client, core, dbPath); err != nil {
					return err
				}
				fmt.Printf("%s will launch Polterware MCP in new sessions\n", client.Name)
				continue
			}
			if err := client.Remove(); err != nil {
				return err
			}
			fmt.Printf("Polterware removed from %s\n", client.Name)
		}
		return nil
	}
	return errors.New(setupMcpUsage)
}

// pickMcpClients names one client, or every installed one for `all`.
func pickMcpClients(id string) ([]agentsetup.McpClient, error) {
	if id != "all" {
		client, ok := agentsetup.FindMcpClient(id)
		if !ok {
			return nil, fmt.Errorf("unknown MCP client %q (claude, codex, cursor, or all)", id)
		}
		return []agentsetup.McpClient{client}, nil
	}
	core, err := coreclient.Executable()
	if err != nil {
		return nil, err
	}
	var installed []agentsetup.McpClient
	for _, client := range agentsetup.McpClients() {
		if status, err := client.Status(core, ""); err == nil && status != agentsetup.McpNotInstalled {
			installed = append(installed, client)
		}
	}
	return installed, nil
}

func listMcpClients(core, dbPath string, asJSON bool) error {
	var rows []mcpClientRow
	for _, client := range agentsetup.McpClients() {
		row := mcpClientRow{ID: client.ID, Name: client.Name}
		status, err := client.Status(core, dbPath)
		row.Status = status.String()
		if err != nil {
			row.Error = err.Error()
		}
		rows = append(rows, row)
	}
	if asJSON {
		return json.NewEncoder(os.Stdout).Encode(map[string]any{"command": core, "dbPath": dbPath, "clients": rows})
	}
	labels := map[string]string{
		"not_installed":    "not installed",
		"not_configured":   "not set up",
		"configured":       "set up",
		"configured_other": "set up for another binary or database",
		"unknown":          "unknown",
	}
	for _, row := range rows {
		line := fmt.Sprintf("%-12s %s", row.Name, labels[row.Status])
		if row.Error != "" {
			line += " (" + row.Error + ")"
		}
		fmt.Println(line)
	}
	return nil
}
