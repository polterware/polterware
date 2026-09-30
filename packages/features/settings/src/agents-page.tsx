import { Match, Show, Switch, createEffect, createMemo, createSignal, on, onCleanup, onMount } from "solid-js";
import {
  Button,
  Boxes,
  History,
  Layers,
  PageContent,
  PageToolbar,
  Plug,
  RefreshCw,
  Tabs,
  pushToast,
  type TabItem,
} from "@polterware/frontend-design-system";
import type {
  AgentRunDetail,
  AgentSessionsView,
  AgentSetupView,
  CompilePreviewResult,
  DriftPreviewResult,
  McpStatusView,
  ProjectChangeRow,
} from "./agents-types";
import { AgentSessionsCard, mapAgentSessions } from "./agent-sessions-card";
import { AgentRunHistoryCard, type ChangeActorFilter } from "./agent-run-history-card";
import { CompilePreviewCard } from "./compile-preview-card";
import { DriftPreviewCard } from "./drift-preview-card";
import { AgentSetupCard, mapAgentSetup, mapHookSwitches } from "./agent-setup-card";
import { McpStatusCard } from "./mcp-status-card";
import { PpmFactsPanel, type PpmJustifyChange } from "./ppm-facts-panel";
import { OraclePreviewCard } from "./oracle-preview-card";
import { SkillCard } from "./skill-card";
import { SettingsSection } from "./settings-section";

/** Centered settings-style column shared by the Access, Sessions, and Context tabs. */
const COLUMN = "mx-auto flex w-full max-w-2xl flex-col gap-6 overflow-auto px-4 py-6 text-[12.5px]";

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
    id: "facts",
    label: "Facts",
    tooltip: {
      title: "PPM facts",
      description:
        "Inspect and justify PPM facts and evidence for the current project. Mutations require a change envelope and appear in History.",
      icon: Boxes,
    },
  },
  {
    id: "sessions",
    label: "Sessions",
    tooltip: {
      title: "Live sessions",
      description:
        "Agents working on this project now, the tasks they claimed, and the most recent handoffs.",
      icon: Plug,
    },
  },
  {
    id: "history",
    label: "History",
    tooltip: {
      title: "Unified Change History",
      description:
        "Justified project-model changes from humans and agents: summary, rationale, and the audit chain for each mutation.",
      icon: History,
    },
  },
  {
    id: "context",
    label: "Context",
    tooltip: {
      title: "Context and drift",
      description:
        "Preview the Oracle text an agent would receive, CompileContext, and the Narrow Drift Checker. Findings are advisory.",
      icon: Layers,
    },
  },
];

type AgentsTabId = "access" | "context" | "facts" | "history" | "sessions";

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
  checkDrift: (payload: {
    projectId: string;
    entityId?: string;
  }) => Promise<unknown>;
  listChanges: (payload: {
    projectId: string;
    actorKind?: string;
    limit?: number;
  }) => Promise<unknown>;
  getAgentRun: (payload: { id: string }) => Promise<unknown>;
};

function workQuery():
  | ((payload: { projectId: string; query: string }) => Promise<unknown>)
  | undefined {
  const root = globalThis as {
    polterware?: { core?: { work?: { query?: (payload: { projectId: string; query: string }) => Promise<unknown> } } };
  };
  return root.polterware?.core?.work?.query;
}

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
          checkDrift?: (payload: {
            projectId: string;
            entityId?: string;
          }) => Promise<unknown>;
          listChanges?: (payload: {
            projectId: string;
            actorKind?: string;
            limit?: number;
          }) => Promise<unknown>;
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
    || !context.checkDrift
    || !context.listChanges
    || !context.getAgentRun
  ) {
    return undefined;
  }
  return {
    getMcpStatus: app.getMcpStatus,
    startMcp: app.startMcp,
    stopMcp: app.stopMcp,
    compileContext: context.compile,
    checkDrift: context.checkDrift,
    listChanges: context.listChanges,
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
      actorKind: asString(runRaw.actorKind) || "agent",
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

function mapProjectChanges(raw: unknown): ProjectChangeRow[] {
  const record = asRecord(raw);
  const changes = Array.isArray(record.changes) ? record.changes : [];
  return changes.flatMap((item) => {
    const row = asRecord(item);
    const stepId = asString(row.stepId);
    if (!stepId) return [];
    return [{
      stepId,
      agentRunId: asString(row.agentRunId),
      projectId: asString(row.projectId),
      actorKind: asString(row.actorKind) || "agent",
      agentLabel: asString(row.agentLabel),
      runStatus: asString(row.runStatus),
      runTask: asString(row.runTask),
      summary: asString(row.summary),
      payloadJson: asString(row.payloadJson),
      createdAt: asString(row.createdAt),
    }];
  });
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

function mapDriftResult(value: unknown): DriftPreviewResult {
  const raw = asRecord(value);
  const findingsRaw = Array.isArray(raw.findings) ? raw.findings : [];
  return {
    projectId: asString(raw.projectId),
    entityId: asString(raw.entityId),
    status: asString(raw.status),
    checker: asString(raw.checker) || "narrow",
    findings: findingsRaw.flatMap((item) => {
      const row = asRecord(item);
      const subjectIds = Array.isArray(row.subjectIds)
        ? row.subjectIds.flatMap((value) => {
            const id = asString(value);
            return id ? [id] : [];
          })
        : [];
      const evidenceIds = Array.isArray(row.evidenceIds)
        ? row.evidenceIds.flatMap((value) => {
            const id = asString(value);
            return id ? [id] : [];
          })
        : [];
      return [{
        type: asString(row.type),
        severity: asString(row.severity),
        summary: asString(row.summary),
        subjectIds,
        evidenceIds,
      }];
    }),
  };
}

function normalizeAgentsTab(raw?: string): AgentsTabId {
  if (raw === "compile" || raw === "drift") return "context";
  if (raw === "context" || raw === "facts" || raw === "history" || raw === "access" || raw === "sessions") {
    return raw;
  }
  return "access";
}

/** Agents: MCP access, context/drift preview, PPM facts, and Unified Change History. */
export function AgentsPage(props: {
  projectId: string;
  tab?: string;
  /** Tab rail supplied by the host section; defaults to the Agents tabs. */
  tabs?: TabItem[];
  /** Accessible name for a host-supplied rail. */
  tabsLabel?: string;
  onTabChange?: (tab: string) => void;
  justifyChange: PpmJustifyChange;
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
  const [driftBusy, setDriftBusy] = createSignal(false);
  const [driftResult, setDriftResult] = createSignal<DriftPreviewResult | undefined>();
  const [driftError, setDriftError] = createSignal<string | undefined>();
  const [factsBusy, setFactsBusy] = createSignal(false);
  const [factsRefreshToken, setFactsRefreshToken] = createSignal(0);
  const [historyBusy, setHistoryBusy] = createSignal(false);
  const [changes, setChanges] = createSignal<ProjectChangeRow[]>([]);
  const [actorFilter, setActorFilter] = createSignal<ChangeActorFilter>("all");
  const [selectedStepId, setSelectedStepId] = createSignal("");
  const [runDetail, setRunDetail] = createSignal<AgentRunDetail | undefined>();

  const [sessionsView, setSessionsView] = createSignal<AgentSessionsView | undefined>();
  const [sessionsLoaded, setSessionsLoaded] = createSignal(false);

  const [setupView, setSetupView] = createSignal<AgentSetupView | undefined>();
  const [setupLoaded, setSetupLoaded] = createSignal(false);

  // The page stays mounted across project switches: drop the previous project's
  // views so the cards show their loading state, then reload history. The setup
  // and sessions effects below refetch on their own because they read projectId.
  createEffect(
    on(
      () => props.projectId,
      () => {
        setSetupView(undefined);
        setSetupLoaded(false);
        setSessionsView(undefined);
        setSessionsLoaded(false);
        setChanges([]);
        setSelectedStepId("");
        setRunDetail(undefined);
        setCompileResult(undefined);
        setDriftResult(undefined);
        void refreshHistory();
      },
      { defer: true },
    ),
  );

  type SetupBridge = {
    getAgentSetup?: (payload: { projectId: string }) => Promise<unknown>;
    setHookSwitch?: (payload: { name: string; enabled: boolean }) => Promise<unknown>;
  };
  const setupBridge = () =>
    (globalThis as { polterware?: { core?: { context?: SetupBridge } } }).polterware?.core?.context;

  const refreshSetup = async () => {
    const get = setupBridge()?.getAgentSetup;
    if (!get) {
      setSetupLoaded(true);
      return;
    }
    try {
      setSetupView(mapAgentSetup(await get({ projectId: props.projectId.trim() })));
    } catch (error: unknown) {
      pushToast(error instanceof Error ? error.message : "Could not read agent setup.", "danger");
    } finally {
      setSetupLoaded(true);
    }
  };

  const toggleHook = async (name: string, enabled: boolean) => {
    const set = setupBridge()?.setHookSwitch;
    if (!set) return;
    try {
      const response = (await set({ name, enabled })) as { hooks?: unknown } | undefined;
      const hooks = mapHookSwitches(response?.hooks);
      setSetupView((current) => (current ? { ...current, hooks } : current));
    } catch (error: unknown) {
      pushToast(error instanceof Error ? error.message : "Could not save the hook switch.", "danger");
    }
  };

  createEffect(() => {
    if (tab() === "access") void refreshSetup();
  });

  const refreshSessions = async () => {
    const projectId = props.projectId.trim();
    const root = globalThis as {
      polterware?: { core?: { work?: { listAgentSessions?: (payload: { projectId: string }) => Promise<unknown> } } };
    };
    const list = root.polterware?.core?.work?.listAgentSessions;
    if (!list || !projectId) {
      setSessionsLoaded(true);
      return;
    }
    try {
      setSessionsView(mapAgentSessions(await list({ projectId })));
    } catch (error: unknown) {
      pushToast(error instanceof Error ? error.message : "Could not read sessions.", "danger");
    } finally {
      setSessionsLoaded(true);
    }
  };

  // Sessions start and end outside the renderer, so poll while the tab is open.
  createEffect(() => {
    if (tab() !== "sessions") return;
    void refreshSessions();
    const timer = setInterval(() => void refreshSessions(), 5000);
    onCleanup(() => clearInterval(timer));
  });

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

  const refreshHistory = async (filterOverride?: ChangeActorFilter) => {
    const bridge = agentsBridge();
    const projectId = props.projectId.trim();
    if (!bridge || !projectId) {
      setChanges([]);
      setRunDetail(undefined);
      setSelectedStepId("");
      return;
    }
    setHistoryBusy(true);
    try {
      const filter = filterOverride ?? actorFilter();
      const nextChanges = mapProjectChanges(await bridge.listChanges({
        projectId,
        limit: 50,
        ...(filter === "all" ? {} : { actorKind: filter }),
      }));
      setChanges(nextChanges);
      // Keep the user's selection across refreshes; never auto-open the first change.
      const current = selectedStepId();
      const selected = nextChanges.find((change) => change.stepId === current);
      setSelectedStepId(selected?.stepId ?? "");
      if (selected?.agentRunId) await loadRunDetail(selected.agentRunId);
      else setRunDetail(undefined);
    } catch (error: unknown) {
      pushToast(error instanceof Error ? error.message : "Could not load change history.", "danger");
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

  const runCompile = () => {
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
  };

  const runDrift = () => {
    const bridge = agentsBridge();
    const projectId = props.projectId.trim();
    if (!bridge) {
      setDriftError("Drift checker is unavailable in this session.");
      return;
    }
    if (!projectId) {
      setDriftError("Select a project before checking drift.");
      return;
    }
    setDriftBusy(true);
    setDriftError(undefined);
    const trimmedEntityId = entityId().trim();
    void bridge.checkDrift({
      projectId,
      ...(trimmedEntityId ? { entityId: trimmedEntityId } : {}),
    })
      .then((response) => {
        setDriftResult(mapDriftResult(response));
      })
      .catch((error: unknown) => {
        setDriftResult(undefined);
        setDriftError(error instanceof Error ? error.message : "Drift check failed.");
      })
      .finally(() => setDriftBusy(false));
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
            items={[...(props.tabs ?? AGENTS_TABS)]}
            value={tab()}
            onChange={(value) => props.onTabChange?.(value)}
            aria-label={props.tabsLabel ?? "Agents"}
          />
        </PageToolbar.Tabs>
        <PageToolbar.Actions>
          <Show when={tab() === "access"}>
            <Button
              size="sm"
              variant="ghost"
              disabled={mcpBusy()}
              onClick={() => {
                void refreshMcp();
                void refreshSetup();
              }}
            >
              <RefreshCw size={14} />
              Refresh
            </Button>
          </Show>
          <Show when={tab() === "context"}>
            <Button
              size="sm"
              variant="ghost"
              disabled={compileBusy() || driftBusy()}
              onClick={() => {
                runCompile();
                runDrift();
              }}
            >
              <RefreshCw size={14} />
              Refresh
            </Button>
          </Show>
          <Show when={tab() === "facts"}>
            <Button
              size="sm"
              variant="ghost"
              disabled={factsBusy()}
              onClick={() => setFactsRefreshToken((value) => value + 1)}
            >
              <RefreshCw size={14} />
              Refresh
            </Button>
          </Show>
          <Show when={tab() === "sessions"}>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => void refreshSessions()}
            >
              <RefreshCw size={14} />
              Refresh
            </Button>
          </Show>
          <Show when={tab() === "history"}>
            <Button
              size="sm"
              variant="ghost"
              disabled={historyBusy()}
              onClick={() => void refreshHistory()}
            >
              <RefreshCw size={14} />
              Refresh
            </Button>
          </Show>
        </PageToolbar.Actions>
      </PageToolbar.Root>

      <Switch>
        <Match when={tab() === "access"}>
          <PageContent class={COLUMN}>
            <McpStatusCard
              status={mcpStatus()}
              busy={mcpBusy()}
              onStart={() => startMcp()}
              onStop={() => stopMcp()}
            />
            <AgentSetupCard setup={setupView()} loaded={setupLoaded()} onToggle={(name, enabled) => void toggleHook(name, enabled)} />
            <SkillCard />
          </PageContent>
        </Match>

        <Match when={tab() === "context"}>
          <PageContent class={COLUMN}>
            <SettingsSection
              title="Oracle"
              description="The exact text an agent receives for one node, card, or title."
            >
              <OraclePreviewCard projectId={props.projectId} query={workQuery()} />
            </SettingsSection>
            <SettingsSection
              title="Compile context"
              description="The ranked context package an agent gets for a query, within a token budget."
            >
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
                onCompile={() => runCompile()}
              />
            </SettingsSection>
            <SettingsSection
              title="Drift check"
              description="The narrow drift checker over the project model. Findings are advisory."
            >
              <DriftPreviewCard
                entityId={entityId()}
                busy={driftBusy()}
                result={driftResult()}
                error={driftError()}
                onEntityIdChange={setEntityId}
                onCheck={() => runDrift()}
              />
            </SettingsSection>
          </PageContent>
        </Match>

        <Match when={tab() === "facts"}>
          <PageContent class="flex min-h-0 flex-1 flex-col p-0">
            <PpmFactsPanel
              projectId={props.projectId}
              entityFilter={entityId()}
              busy={factsBusy()}
              refreshToken={factsRefreshToken()}
              justifyChange={props.justifyChange}
              onEntityFilterChange={setEntityId}
              onBusyChange={setFactsBusy}
            />
          </PageContent>
        </Match>

        <Match when={tab() === "sessions"}>
          <PageContent class={COLUMN}>
            <AgentSessionsCard view={sessionsView()} loaded={sessionsLoaded()} />
          </PageContent>
        </Match>

        <Match when={tab() === "history"}>
          <PageContent class="flex min-h-0 flex-1 flex-col p-0">
            <AgentRunHistoryCard
              changes={changes()}
              selectedStepId={selectedStepId()}
              detail={runDetail()}
              actorFilter={actorFilter()}
              busy={historyBusy()}
              onActorFilter={(filter) => {
                setActorFilter(filter);
                void refreshHistory(filter);
              }}
              onSelect={(stepId, agentRunId) => {
                setSelectedStepId(stepId);
                void loadRunDetail(agentRunId);
              }}
              onClear={() => {
                setSelectedStepId("");
                setRunDetail(undefined);
              }}
            />
          </PageContent>
        </Match>
      </Switch>
    </div>
  );
}
