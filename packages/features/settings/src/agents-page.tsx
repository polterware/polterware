import { Match, Show, Switch, createMemo, createSignal, onMount } from "solid-js";
import {
  Button,
  History,
  Layers,
  PageContent,
  PageToolbar,
  Plug,
  RefreshCw,
  StatusChip,
  Tabs,
  pushToast,
  type TabItem,
} from "@polterware/frontend-design-system";
import type {
  AgentRunDetail,
  AgentRunRow,
  CompilePreviewResult,
  McpStatusView,
} from "./agents-types";
import { AgentRunHistoryCard } from "./agent-run-history-card";
import { CompilePreviewCard } from "./compile-preview-card";
import { McpStatusCard } from "./mcp-status-card";
import { SkillCard } from "./skill-card";

const AGENTS_TABS: TabItem[] = [
  {
    id: "access",
    label: "Access",
    tooltip: {
      title: "MCP access",
      description:
        "Start or stop the parked local MCP helper, copy Cursor MCP config, and copy or download the Polterware AI skill for this project.",
      icon: Plug,
    },
  },
  {
    id: "compile",
    label: "Compile",
    tooltip: {
      title: "Context compile",
      description:
        "Preview CompileContext for the current project: a bounded package of entities, facts with evidence, knowledge links, and search hits under a token budget.",
      icon: Layers,
    },
  },
  {
    id: "history",
    label: "History",
    tooltip: {
      title: "AgentRun audit",
      description:
        "Minimal AgentRun audit trail: context delivered, task, declared change, evidence, and drift findings for this project.",
      icon: History,
    },
  },
];

type AgentsTabId = "access" | "compile" | "history";

type AgentsBridge = {
  getMcpStatus: () => Promise<unknown>;
  startMcp: () => Promise<unknown>;
  stopMcp: () => Promise<unknown>;
  compileContext: (payload: {
    projectId: string;
    query: string;
    entityId?: string;
    tokenBudget: number;
  }) => Promise<unknown>;
  listAgentRuns: (payload: { projectId: string; limit?: number }) => Promise<unknown>;
  getAgentRun: (payload: { id: string }) => Promise<unknown>;
};

function agentsBridge(): AgentsBridge | undefined {
  const root = globalThis as {
    polterware?: {
      app?: {
        getMcpStatus?: () => Promise<unknown>;
        startMcp?: () => Promise<unknown>;
        stopMcp?: () => Promise<unknown>;
      };
      core?: {
        context?: {
          compile?: (payload: {
            projectId: string;
            query: string;
            entityId?: string;
            tokenBudget: number;
          }) => Promise<unknown>;
          listAgentRuns?: (payload: { projectId: string; limit?: number }) => Promise<unknown>;
          getAgentRun?: (payload: { id: string }) => Promise<unknown>;
        };
      };
    };
  };
  const app = root.polterware?.app;
  const context = root.polterware?.core?.context;
  if (
    !app?.getMcpStatus
    || !app.startMcp
    || !app.stopMcp
    || !context?.compile
    || !context.listAgentRuns
    || !context.getAgentRun
  ) {
    return undefined;
  }
  return {
    getMcpStatus: app.getMcpStatus,
    startMcp: app.startMcp,
    stopMcp: app.stopMcp,
    compileContext: context.compile,
    listAgentRuns: context.listAgentRuns,
    getAgentRun: context.getAgentRun,
  };
}

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function asString(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function asNumber(value: unknown, fallback = 0): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function asBoolean(value: unknown): boolean {
  return value === true;
}

function mapMcpStatus(raw: unknown): McpStatusView {
  const record = asRecord(raw);
  const pidRaw = record.pid;
  return {
    running: asBoolean(record.running),
    pid: typeof pidRaw === "number" ? pidRaw : null,
    dbPath: asString(record.dbPath),
    command: asString(record.command),
    cursorConfig: asString(record.cursorConfig),
  };
}

function mapAgentRuns(raw: unknown): AgentRunRow[] {
  const record = asRecord(raw);
  const runs = Array.isArray(record.runs) ? record.runs : [];
  return runs.flatMap((item) => {
    const run = asRecord(item);
    const id = asString(run.id);
    if (!id) return [];
    return [{
      id,
      projectId: asString(run.projectId),
      task: asString(run.task),
      agentLabel: asString(run.agentLabel),
      status: asString(run.status),
      createdAt: asString(run.createdAt),
      completedAt: asString(run.completedAt),
    }];
  });
}

function mapAgentRunDetail(raw: unknown): AgentRunDetail | undefined {
  const record = asRecord(raw);
  const runRaw = asRecord(record.run);
  const id = asString(runRaw.id);
  if (!id) return undefined;
  const stepsRaw = Array.isArray(record.steps) ? record.steps : [];
  return {
    run: {
      id,
      projectId: asString(runRaw.projectId),
      task: asString(runRaw.task),
      agentLabel: asString(runRaw.agentLabel),
      status: asString(runRaw.status),
      createdAt: asString(runRaw.createdAt),
      completedAt: asString(runRaw.completedAt),
    },
    steps: stepsRaw.flatMap((item) => {
      const step = asRecord(item);
      const stepId = asString(step.id);
      if (!stepId) return [];
      return [{
        id: stepId,
        agentRunId: asString(step.agentRunId),
        kind: asString(step.kind),
        summary: asString(step.summary),
        payloadJson: asString(step.payloadJson),
        createdAt: asString(step.createdAt),
      }];
    }),
  };
}

function mapCompileResult(raw: unknown): CompilePreviewResult {
  const record = asRecord(raw);
  const itemsRaw = Array.isArray(record.items) ? record.items : [];
  return {
    tokensUsed: asNumber(record.tokensUsed),
    tokenBudget: asNumber(record.tokenBudget),
    truncated: asBoolean(record.truncated),
    items: itemsRaw.flatMap((item) => {
      const row = asRecord(item);
      const id = asString(row.id);
      if (!id) return [];
      const reasonsRaw = Array.isArray(row.reasons) ? row.reasons : [];
      const evidenceRaw = Array.isArray(row.evidence) ? row.evidence : [];
      return [{
        kind: asString(row.kind),
        id,
        title: asString(row.title) || id,
        excerpt: asString(row.excerpt),
        truthClass: asString(row.truthClass),
        source: asString(row.source),
        path: asString(row.path),
        reasons: reasonsRaw.flatMap((value) => {
          const reason = asString(value);
          return reason ? [reason] : [];
        }),
        evidence: evidenceRaw.flatMap((value) => {
          const ev = asRecord(value);
          const evidenceId = asString(ev.id);
          if (!evidenceId) return [];
          return [{
            id: evidenceId,
            kind: asString(ev.kind),
            summary: asString(ev.summary),
            path: asString(ev.path),
            source: asString(ev.source),
          }];
        }),
      }];
    }),
  };
}

function normalizeAgentsTab(raw?: string): AgentsTabId {
  if (raw === "compile" || raw === "history" || raw === "access") return raw;
  return "access";
}

/** Agents: MCP access, compile preview, and AgentRun audit history. */
export function AgentsPage(props: {
  projectId: string;
  tab?: string;
  onTabChange?: (tab: string) => void;
}) {
  const tab = createMemo(() => normalizeAgentsTab(props.tab));
  const [mcpStatus, setMcpStatus] = createSignal<McpStatusView | undefined>();
  const [mcpBusy, setMcpBusy] = createSignal(false);
  const [query, setQuery] = createSignal("architecture");
  const [entityId, setEntityId] = createSignal("");
  const [budget, setBudget] = createSignal("1200");
  const [compileBusy, setCompileBusy] = createSignal(false);
  const [compileResult, setCompileResult] = createSignal<CompilePreviewResult | undefined>();
  const [compileError, setCompileError] = createSignal<string | undefined>();
  const [historyBusy, setHistoryBusy] = createSignal(false);
  const [runs, setRuns] = createSignal<AgentRunRow[]>([]);
  const [selectedRunId, setSelectedRunId] = createSignal("");
  const [runDetail, setRunDetail] = createSignal<AgentRunDetail | undefined>();

  const running = () => Boolean(mcpStatus()?.running);

  const refreshMcp = async () => {
    const bridge = agentsBridge();
    if (!bridge) {
      pushToast("MCP controls are unavailable in this session.", "danger");
      return;
    }
    setMcpBusy(true);
    try {
      setMcpStatus(mapMcpStatus(await bridge.getMcpStatus()));
    } catch (error: unknown) {
      pushToast(error instanceof Error ? error.message : "Could not read MCP status.", "danger");
    } finally {
      setMcpBusy(false);
    }
  };

  const loadRunDetail = async (id: string) => {
    const bridge = agentsBridge();
    if (!bridge || !id) {
      setRunDetail(undefined);
      return;
    }
    try {
      setRunDetail(mapAgentRunDetail(await bridge.getAgentRun({ id })));
    } catch (error: unknown) {
      setRunDetail(undefined);
      pushToast(error instanceof Error ? error.message : "Could not load agent run.", "danger");
    }
  };

  const refreshHistory = async () => {
    const bridge = agentsBridge();
    const projectId = props.projectId.trim();
    if (!bridge || !projectId) {
      setRuns([]);
      setRunDetail(undefined);
      setSelectedRunId("");
      return;
    }
    setHistoryBusy(true);
    try {
      const nextRuns = mapAgentRuns(await bridge.listAgentRuns({ projectId, limit: 50 }));
      setRuns(nextRuns);
      const current = selectedRunId();
      const nextId = nextRuns.some((run) => run.id === current)
        ? current
        : (nextRuns[0]?.id ?? "");
      setSelectedRunId(nextId);
      if (nextId) await loadRunDetail(nextId);
      else setRunDetail(undefined);
    } catch (error: unknown) {
      pushToast(error instanceof Error ? error.message : "Could not load agent runs.", "danger");
    } finally {
      setHistoryBusy(false);
    }
  };

  const startMcp = () => {
    const bridge = agentsBridge();
    if (!bridge) {
      pushToast("MCP controls are unavailable in this session.", "danger");
      return;
    }
    setMcpBusy(true);
    void bridge.startMcp()
      .then((status) => {
        setMcpStatus(mapMcpStatus(status));
        pushToast("MCP helper started.", "success");
      })
      .catch((error: unknown) => {
        pushToast(error instanceof Error ? error.message : "Could not start MCP.", "danger");
      })
      .finally(() => setMcpBusy(false));
  };

  const stopMcp = () => {
    const bridge = agentsBridge();
    if (!bridge) {
      pushToast("MCP controls are unavailable in this session.", "danger");
      return;
    }
    setMcpBusy(true);
    void bridge.stopMcp()
      .then((status) => {
        setMcpStatus(mapMcpStatus(status));
        pushToast("MCP helper stopped.", "success");
      })
      .catch((error: unknown) => {
        pushToast(error instanceof Error ? error.message : "Could not stop MCP.", "danger");
      })
      .finally(() => setMcpBusy(false));
  };

  onMount(() => {
    void refreshMcp();
    void refreshHistory();
  });

  return (
    <div class="flex h-full min-h-0 flex-col">
      <PageToolbar.Root>
        <PageToolbar.Tabs>
          <Tabs
            items={[...AGENTS_TABS]}
            value={tab()}
            onChange={(value) => props.onTabChange?.(value)}
            aria-label="Agents"
          />
        </PageToolbar.Tabs>
        <PageToolbar.Actions>
          <StatusChip tone={running() ? "ok" : "warn"} label={running() ? "Running" : "Stopped"} />
          <Show when={tab() === "access" || tab() === "history"}>
            <Button
              size="sm"
              variant="ghost"
              disabled={tab() === "history" ? historyBusy() : mcpBusy()}
              onClick={() => {
                if (tab() === "history") void refreshHistory();
                else void refreshMcp();
              }}
            >
              <RefreshCw size={14} />
              Refresh
            </Button>
          </Show>
          <Button
            size="sm"
            variant="primary"
            disabled={mcpBusy() || running()}
            onClick={() => startMcp()}
          >
            Start
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={mcpBusy() || !running()}
            onClick={() => stopMcp()}
          >
            Stop
          </Button>
        </PageToolbar.Actions>
      </PageToolbar.Root>

      <Switch>
        <Match when={tab() === "access"}>
          <PageContent class="max-w-2xl space-y-3 overflow-auto p-4 text-[12.5px]">
            <McpStatusCard status={mcpStatus()} />
            <SkillCard />
          </PageContent>
        </Match>

        <Match when={tab() === "compile"}>
          <PageContent class="max-w-2xl overflow-auto p-4 text-[12.5px]">
            <CompilePreviewCard
              query={query()}
              entityId={entityId()}
              budget={budget()}
              busy={compileBusy()}
              result={compileResult()}
              error={compileError()}
              onQueryChange={setQuery}
              onEntityIdChange={setEntityId}
              onBudgetChange={setBudget}
              onCompile={() => {
                const bridge = agentsBridge();
                const projectId = props.projectId.trim();
                if (!bridge) {
                  setCompileError("Context compiler is unavailable in this session.");
                  return;
                }
                if (!projectId) {
                  setCompileError("Select a project before compiling context.");
                  return;
                }
                setCompileBusy(true);
                setCompileError(undefined);
                const tokenBudget = Number.parseInt(budget(), 10);
                const trimmedEntityId = entityId().trim();
                void bridge.compileContext({
                  projectId,
                  query: query().trim(),
                  ...(trimmedEntityId ? { entityId: trimmedEntityId } : {}),
                  tokenBudget: Number.isFinite(tokenBudget) && tokenBudget > 0 ? tokenBudget : 1200,
                })
                  .then((response) => {
                    setCompileResult(mapCompileResult(response));
                  })
                  .catch((error: unknown) => {
                    setCompileResult(undefined);
                    setCompileError(error instanceof Error ? error.message : "Compile failed.");
                  })
                  .finally(() => setCompileBusy(false));
              }}
            />
          </PageContent>
        </Match>

        <Match when={tab() === "history"}>
          <PageContent class="flex min-h-0 flex-1 flex-col p-0">
            <AgentRunHistoryCard
              runs={runs()}
              selectedId={selectedRunId()}
              detail={runDetail()}
              busy={historyBusy()}
              onSelect={(id) => {
                setSelectedRunId(id);
                void loadRunDetail(id);
              }}
            />
          </PageContent>
        </Match>
      </Switch>
    </div>
  );
}
