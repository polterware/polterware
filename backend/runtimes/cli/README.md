# polterware CLI

```bash
cd backend
go build -o ../dist/polterware ./runtimes/cli

../dist/polterware -db /tmp/polterware-test.db init --name Demo --path /path/to/repo
../dist/polterware -db /tmp/polterware-test.db scan --project <id>
../dist/polterware -db /tmp/polterware-test.db context --project <id> --json
../dist/polterware -db /tmp/polterware-test.db query "commits <slug>"
```

`polterware status` prints the current card, branch, handoff, and open questions; `polterware setup mcp` registers the MCP server with Claude Code, Codex, and Cursor.

`serve` runs the local core server the Desktop attaches to; `mcp` runs the MCP stdio server for coding agents. Other commands open SQLite in-process. Run `polterware --help` for every command.
