// Static manual for the `polterware` CLI.
// Keep in sync with backend/runtimes/cli (output.go usage and each command's FlagSet).

export type CliFlag = {
  name: string;
  value?: string;
  required?: boolean;
  description: string;
};

export type CliCommand = {
  id: string;
  name: string;
  synopsis: string;
  summary: string;
  flags: readonly CliFlag[];
  example: string;
};

export type CliKey = {
  keys: string;
  description: string;
};

export const CLI_DEFAULT_DB =
  "~/Library/Application Support/polterware/local.db";

export const CLI_USAGE = "polterware [-db path] <command> [flags]";

export const CLI_USAGE_FLAGS: readonly CliFlag[] = [
  {
    name: "-db",
    value: "path",
    description: `SQLite database path. Defaults to ${CLI_DEFAULT_DB}, the same database the Desktop uses. Place it before the command.`,
  },
];

const JSON_FLAG: CliFlag = {
  name: "--json",
  description: "Print the result as JSON instead of plain text.",
};

const AGENT_RUN_FLAG: CliFlag = {
  name: "--agent-run-id",
  value: "id",
  description: "Attach the result to an open AgentRun.",
};

export const CLI_COMMANDS: readonly CliCommand[] = [
  {
    id: "tui",
    name: "polterware",
    synopsis: "polterware [-db path]",
    summary:
      "Interactive status and MCP setup. Requires a terminal; without one it prints usage.",
    flags: [],
    example: "polterware",
  },
  {
    id: "init",
    name: "polterware init",
    synopsis: "polterware init --name <name> [--path <root>] [--id <id>] [--json]",
    summary: "Register a project rooted at a local directory.",
    flags: [
      { name: "--name", value: "name", required: true, description: "Project name." },
      {
        name: "--path",
        value: "root",
        description: "Project root path. Defaults to the current directory.",
      },
      { name: "--id", value: "id", description: "Optional explicit project id." },
      JSON_FLAG,
    ],
    example: 'polterware init --name "My App"',
  },
  {
    id: "scan",
    name: "polterware scan",
    synopsis: "polterware scan --project <id> [--json]",
    summary:
      "Import facts, evidence, and sources from the project root. Prints fact, evidence, source, and skipped counts.",
    flags: [
      { name: "--project", value: "id", required: true, description: "Project id." },
      JSON_FLAG,
    ],
    example: "polterware scan --project <project-id>",
  },
  {
    id: "context",
    name: "polterware context",
    synopsis:
      "polterware context --project <id> [--query <text>] [--entity <id>] [--budget N] [--agent-run-id <id>] [--json]",
    summary:
      "Compile a context package for a task within a token budget.",
    flags: [
      { name: "--project", value: "id", required: true, description: "Project id." },
      { name: "--query", value: "text", description: "Optional search query." },
      { name: "--entity", value: "id", description: "Optional entity id to focus on." },
      { name: "--budget", value: "N", description: "Token budget." },
      AGENT_RUN_FLAG,
      JSON_FLAG,
    ],
    example: 'polterware context --project <project-id> --query "auth flow" --json',
  },
  {
    id: "drift",
    name: "polterware drift",
    synopsis:
      "polterware drift --project <id> [--entity <id>] [--agent-run-id <id>] [--json]",
    summary:
      "Check for drift between the project and its architecture, decisions, or specs. Findings are reported, never applied.",
    flags: [
      { name: "--project", value: "id", required: true, description: "Project id." },
      { name: "--entity", value: "id", description: "Optional entity id to check." },
      AGENT_RUN_FLAG,
      JSON_FLAG,
    ],
    example: "polterware drift --project <project-id>",
  },
  {
    id: "apply",
    name: "polterware apply",
    synopsis:
      "polterware apply <file|-> [--dry-run] [--project <id>] [--summary <text>] [--rationale <text>] [--agent-run-id <id>]",
    summary:
      "Write Polter text (Work cards, tasks, questions, invariants) to the store as one audited change. A text only changes what it mentions.",
    flags: [
      { name: "--dry-run", description: "Print the plan without writing." },
      {
        name: "--project",
        value: "id",
        description: "Project id. Defaults to the project whose root contains the current directory.",
      },
      { name: "--summary", value: "text", description: "Change summary. Overrides the frontmatter." },
      { name: "--rationale", value: "text", description: "Change rationale. Overrides the frontmatter." },
      AGENT_RUN_FLAG,
    ],
    example: "polterware apply roadmap.pw.md --dry-run",
  },
  {
    id: "lint",
    name: "polterware lint",
    synopsis: "polterware lint <file|dir|-> [--project <id>]",
    summary:
      "Validate Polter text. Silent on success; prints one line per error with the fix and exits 1. Directories are searched for *.pw.md files.",
    flags: [
      {
        name: "--project",
        value: "id",
        description: "Project id for reference checks. Without a resolvable project, only syntax is checked.",
      },
    ],
    example: "polterware lint roadmap.pw.md",
  },
  {
    id: "format",
    name: "polterware format",
    synopsis: "polterware format",
    summary: "Print the Polter format spec, short enough to teach an agent in one read.",
    flags: [],
    example: "polterware format",
  },
  {
    id: "query",
    name: "polterware query",
    synopsis: 'polterware query "cards" | "card <slug>" [--project <id>]',
    summary:
      "Read Work state in Polter format: one line per active card, or one full card.",
    flags: [
      {
        name: "--project",
        value: "id",
        description: "Project id. Defaults to the project whose root contains the current directory.",
      },
    ],
    example: "polterware query cards",
  },
  {
    id: "hook",
    name: "polterware hook",
    synopsis: "polterware hook session-start [--label <name>] | session-end | lint",
    summary:
      "Claude Code hook entry points (JSON on stdin), installed by the Polterware plugin. session-start makes every apply in the session join one AgentRun per project; session-end completes it; lint checks written .pw.md files and returns errors to the agent.",
    flags: [
      {
        name: "--label",
        value: "name",
        description: "AgentRun label for the session. Defaults to claude-code.",
      },
    ],
    example: "polterware hook lint",
  },
  {
    id: "serve",
    name: "polterware serve",
    synopsis: "polterware [-db path] serve [--listen <addr>] [--token <token>]",
    summary:
      "Run the local core server on a loopback address. The Desktop starts this for you; run it directly for a standalone core.",
    flags: [
      {
        name: "--listen",
        value: "addr",
        description: "Loopback listen address. Defaults to 127.0.0.1:4020.",
      },
      {
        name: "--token",
        value: "token",
        description:
          "Session bearer token. Defaults to POLTERWARE_SESSION_TOKEN, or a freshly minted token.",
      },
    ],
    example: "polterware serve",
  },
  {
    id: "mcp",
    name: "polterware mcp",
    synopsis: "polterware [-db path] mcp",
    summary:
      "Run the MCP server over stdio for coding agents. Tools are experimental and not a stable API contract.",
    flags: [],
    example: "polterware mcp",
  },
  {
    id: "help",
    name: "polterware help",
    synopsis: "polterware help",
    summary: "Print usage. Also available as -h and --help.",
    flags: [],
    example: "polterware help",
  },
];

export const CLI_TUI_KEYS: readonly CliKey[] = [
  { keys: "Tab / Shift+Tab", description: "Move between sections." },
  { keys: "↑ ↓ / k j", description: "Move within a section." },
  { keys: "Enter / Space", description: "Act on the selected row." },
  { keys: "s", description: "Start or stop the local core." },
  { keys: "i", description: "Register the current directory as a project." },
  { keys: "p", description: "Cycle the selected project." },
  { keys: "r", description: "Refresh status." },
  { keys: "q", description: "Quit." },
];

function flagTableMarkdown(flags: readonly CliFlag[], withRequired: boolean): string[] {
  const header = withRequired
    ? ["| Flag | Value | Required | Description |", "| --- | --- | --- | --- |"]
    : ["| Flag | Value | Description |", "| --- | --- | --- |"];
  const rows = flags.map((flag) => {
    const cells = [
      `\`${flag.name}\``,
      flag.value ? `\`<${flag.value}>\`` : "",
      ...(withRequired ? [flag.required ? "Yes" : ""] : []),
      flag.description,
    ];
    return `| ${cells.join(" | ")} |`;
  });
  return [...header, ...rows];
}

function commandIndexMarkdown(commands: readonly CliCommand[]): string[] {
  return [
    "| Command | Description |",
    "| --- | --- |",
    ...commands.map((command) => `| \`${command.name}\` | ${command.summary} |`),
  ];
}

function commandMarkdown(command: CliCommand): string[] {
  const lines = [`### ${command.name}`, "", command.summary, "", "```sh", command.synopsis, "```"];
  if (command.flags.length > 0) {
    lines.push("", "Options:", "", ...flagTableMarkdown(command.flags, true));
  }
  lines.push("", "Example:", "", "```sh", command.example, "```");
  return lines;
}

/** The whole reference as Markdown, for pasting into agents, docs, or issues. */
export function cliReferenceMarkdown(): string {
  const lines = [
    "# polterware CLI reference",
    "",
    "The polterware CLI reads and writes the same local database as the Desktop. init, scan, context, and drift accept --json for scripts and agents.",
    "",
    "## Usage",
    "",
    "```sh",
    CLI_USAGE,
    "```",
    "",
    ...flagTableMarkdown(CLI_USAGE_FLAGS, false),
    "",
    "## Commands",
    "",
    ...commandIndexMarkdown(CLI_COMMANDS),
    "",
    ...CLI_COMMANDS.flatMap((command) => [...commandMarkdown(command), ""]),
    "## Interactive keys",
    "",
    "| Key | Action |",
    "| --- | --- |",
    ...CLI_TUI_KEYS.map((key) => `| \`${key.keys}\` | ${key.description} |`),
  ];
  return `${lines.join("\n").trimEnd()}\n`;
}
