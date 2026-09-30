import { For, Show, createEffect, createSignal } from "solid-js";
import {
  Button,
  DataTable,
  Field,
  Input,
  PageEmpty,
  Skeleton,
  StatusChip,
  X,
  pushToast,
} from "@polterware/frontend-design-system";
import { PpmEvidenceFormDialog } from "./ppm-evidence-form-dialog";
import { PpmFactFormDialog } from "./ppm-fact-form-dialog";
import { PpmLinkEvidenceDialog } from "./ppm-link-evidence-dialog";
import type {
  PpmEvidenceDraft,
  PpmEvidenceRow,
  PpmFactDetail,
  PpmFactDraft,
  PpmFactRow,
  PpmFactTruthClass,
} from "./ppm-facts-types";

export type PpmChangeEnvelope = {
  summary: string;
  rationale: string;
  agentRunId?: string;
};

export type PpmJustifyChange = (input: {
  title: string;
  description?: string;
  defaultSummary?: string;
}) => Promise<PpmChangeEnvelope | null>;

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function asString(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function mapFact(value: unknown): PpmFactRow | undefined {
  const raw = asRecord(value);
  const id = asString(raw.id);
  if (!id) return undefined;
  return {
    id,
    projectId: asString(raw.projectId),
    statement: asString(raw.statement),
    truthClass: asString(raw.truthClass),
    source: asString(raw.source),
    actor: asString(raw.actor),
    entityId: asString(raw.entityId),
    observedAt: asString(raw.observedAt),
    createdAt: asString(raw.createdAt),
    updatedAt: asString(raw.updatedAt),
  };
}

function mapEvidence(value: unknown): PpmEvidenceRow | undefined {
  const raw = asRecord(value);
  const id = asString(raw.id);
  if (!id) return undefined;
  return {
    id,
    projectId: asString(raw.projectId),
    kind: asString(raw.kind),
    summary: asString(raw.summary),
    source: asString(raw.source),
    path: asString(raw.path),
    actor: asString(raw.actor),
    observedAt: asString(raw.observedAt),
    createdAt: asString(raw.createdAt),
    updatedAt: asString(raw.updatedAt),
  };
}

function truthTone(truthClass: string): "ok" | "warn" | "muted" | "syncing" {
  switch (truthClass) {
    case "verified":
      return "ok";
    case "declared":
      return "syncing";
    case "observed":
      return "ok";
    case "inferred":
      return "warn";
    default:
      return "muted";
  }
}

type PpmBridge = {
  listFacts: (payload: { projectId: string; entityId?: string }) => Promise<unknown>;
  getFact: (payload: { id: string }) => Promise<unknown>;
  upsertFact: (payload: Record<string, unknown>) => Promise<unknown>;
  listEvidence: (payload: { projectId: string }) => Promise<unknown>;
  upsertEvidence: (payload: Record<string, unknown>) => Promise<unknown>;
  linkEvidenceToFact: (payload: Record<string, unknown>) => Promise<unknown>;
};

function ppmBridge(): PpmBridge | undefined {
  const root = globalThis as {
    polterware?: {
      core?: {
        ppm?: {
          listFacts?: PpmBridge["listFacts"];
          getFact?: PpmBridge["getFact"];
          upsertFact?: PpmBridge["upsertFact"];
          listEvidence?: PpmBridge["listEvidence"];
          upsertEvidence?: PpmBridge["upsertEvidence"];
          linkEvidenceToFact?: PpmBridge["linkEvidenceToFact"];
        };
      };
    };
  };
  const ppm = root.polterware?.core?.ppm;
  if (
    !ppm?.listFacts
    || !ppm.getFact
    || !ppm.upsertFact
    || !ppm.listEvidence
    || !ppm.upsertEvidence
    || !ppm.linkEvidenceToFact
  ) {
    return undefined;
  }
  return {
    listFacts: ppm.listFacts,
    getFact: ppm.getFact,
    upsertFact: ppm.upsertFact,
    listEvidence: ppm.listEvidence,
    upsertEvidence: ppm.upsertEvidence,
    linkEvidenceToFact: ppm.linkEvidenceToFact,
  };
}

function changePayload(change: { summary: string; rationale: string; agentRunId?: string }) {
  return {
    summary: change.summary,
    rationale: change.rationale,
    agentRunId: change.agentRunId ?? "",
  };
}

/** Agents → Facts: list/inspect PPM facts and evidence; mutate with change envelope. */
export function PpmFactsPanel(props: {
  projectId: string;
  entityFilter: string;
  busy: boolean;
  refreshToken: number;
  justifyChange: PpmJustifyChange;
  onEntityFilterChange: (value: string) => void;
  onBusyChange: (busy: boolean) => void;
}) {
  const [facts, setFacts] = createSignal<PpmFactRow[]>([]);
  const [selectedId, setSelectedId] = createSignal("");
  const [detail, setDetail] = createSignal<PpmFactDetail | undefined>();
  const [projectEvidence, setProjectEvidence] = createSignal<PpmEvidenceRow[]>([]);
  const [factDialogOpen, setFactDialogOpen] = createSignal(false);
  const [factDialogMode, setFactDialogMode] = createSignal<"create" | "edit">("create");
  const [factDraft, setFactDraft] = createSignal<PpmFactDraft | undefined>();
  const [evidenceDialogOpen, setEvidenceDialogOpen] = createSignal(false);
  const [linkDialogOpen, setLinkDialogOpen] = createSignal(false);

  const selected = () => facts().find((fact) => fact.id === selectedId());

  const loadDetail = async (id: string) => {
    const bridge = ppmBridge();
    if (!bridge || !id) {
      setDetail(undefined);
      return;
    }
    try {
      const response = asRecord(await bridge.getFact({ id }));
      const fact = mapFact(response.fact);
      if (!fact) {
        setDetail(undefined);
        return;
      }
      const evidenceRaw = Array.isArray(response.evidence) ? response.evidence : [];
      setDetail({
        fact,
        evidence: evidenceRaw.flatMap((item) => {
          const row = mapEvidence(item);
          return row ? [row] : [];
        }),
      });
    } catch (error: unknown) {
      setDetail(undefined);
      pushToast(error instanceof Error ? error.message : "Could not load fact.", "danger");
    }
  };

  const refresh = async () => {
    const bridge = ppmBridge();
    const projectId = props.projectId.trim();
    if (!bridge || !projectId) {
      setFacts([]);
      setDetail(undefined);
      setSelectedId("");
      setProjectEvidence([]);
      return;
    }
    props.onBusyChange(true);
    try {
      const entityId = props.entityFilter.trim();
      const listResponse = asRecord(await bridge.listFacts({
        projectId,
        ...(entityId ? { entityId } : {}),
      }));
      const nextFacts = (Array.isArray(listResponse.facts) ? listResponse.facts : []).flatMap(
        (item) => {
          const row = mapFact(item);
          return row ? [row] : [];
        },
      );
      setFacts(nextFacts);
      const evidenceResponse = asRecord(await bridge.listEvidence({ projectId }));
      setProjectEvidence(
        (Array.isArray(evidenceResponse.evidence) ? evidenceResponse.evidence : []).flatMap(
          (item) => {
            const row = mapEvidence(item);
            return row ? [row] : [];
          },
        ),
      );
      // Keep the user's selection across refreshes; never auto-open the first fact.
      const current = selectedId();
      const selectedFact = nextFacts.find((fact) => fact.id === current);
      setSelectedId(selectedFact?.id ?? "");
      if (selectedFact) await loadDetail(selectedFact.id);
      else setDetail(undefined);
    } catch (error: unknown) {
      pushToast(error instanceof Error ? error.message : "Could not load facts.", "danger");
    } finally {
      props.onBusyChange(false);
    }
  };

  const saveFact = async (draft: PpmFactDraft) => {
    const bridge = ppmBridge();
    const projectId = props.projectId.trim();
    if (!bridge || !projectId) {
      pushToast("PPM facts are unavailable in this session.", "danger");
      return;
    }
    const envelope = await props.justifyChange({
      title: draft.id ? "Justify fact update" : "Justify new fact",
      description: "Facts become durable project truth only with summary and rationale.",
      defaultSummary: draft.statement.slice(0, 120),
    });
    if (!envelope) return;
    props.onBusyChange(true);
    try {
      await bridge.upsertFact({
        fact: {
          id: draft.id ?? "",
          projectId,
          statement: draft.statement,
          truthClass: draft.truthClass,
          entityId: draft.entityId,
          source: "",
          actor: "",
          observedAt: "",
          createdAt: "",
          updatedAt: "",
        },
        change: changePayload(envelope),
      });
      setFactDialogOpen(false);
      pushToast(draft.id ? "Fact updated." : "Fact created.", "success");
      await refresh();
    } catch (error: unknown) {
      pushToast(error instanceof Error ? error.message : "Could not save fact.", "danger");
      props.onBusyChange(false);
    }
  };

  const saveEvidence = async (draft: PpmEvidenceDraft) => {
    const bridge = ppmBridge();
    const projectId = props.projectId.trim();
    if (!bridge || !projectId) {
      pushToast("PPM evidence is unavailable in this session.", "danger");
      return;
    }
    const envelope = await props.justifyChange({
      title: "Justify new evidence",
      description: "Evidence supports facts and appears in change history.",
      defaultSummary: draft.summary.slice(0, 120),
    });
    if (!envelope) return;
    props.onBusyChange(true);
    try {
      const evidenceId = `ev-${crypto.randomUUID().slice(0, 8)}`;
      await bridge.upsertEvidence({
        evidence: {
          id: evidenceId,
          projectId,
          kind: draft.kind,
          summary: draft.summary,
          path: draft.path,
          source: "",
          actor: "",
          observedAt: "",
          createdAt: "",
          updatedAt: "",
        },
        change: changePayload(envelope),
      });
      const factId = selectedId();
      if (draft.linkToSelectedFact && factId) {
        await bridge.linkEvidenceToFact({
          projectId,
          factId,
          evidenceId,
          change: changePayload(envelope),
        });
      }
      setEvidenceDialogOpen(false);
      pushToast(
        draft.linkToSelectedFact && factId
          ? "Evidence created and linked."
          : "Evidence created.",
        "success",
      );
      await refresh();
    } catch (error: unknown) {
      pushToast(error instanceof Error ? error.message : "Could not save evidence.", "danger");
      props.onBusyChange(false);
    }
  };

  const linkEvidence = async (evidenceId: string) => {
    const bridge = ppmBridge();
    const projectId = props.projectId.trim();
    const factId = selectedId();
    if (!bridge || !projectId || !factId) return;
    const envelope = await props.justifyChange({
      title: "Justify evidence link",
      description: "Link existing evidence to the selected fact.",
      defaultSummary: `Link ${evidenceId} to ${factId}`,
    });
    if (!envelope) return;
    props.onBusyChange(true);
    try {
      await bridge.linkEvidenceToFact({
        projectId,
        factId,
        evidenceId,
        change: changePayload(envelope),
      });
      setLinkDialogOpen(false);
      pushToast("Evidence linked.", "success");
      await refresh();
    } catch (error: unknown) {
      pushToast(error instanceof Error ? error.message : "Could not link evidence.", "danger");
      props.onBusyChange(false);
    }
  };

  createEffect(() => {
    props.projectId;
    props.refreshToken;
    void refresh();
  });

  return (
    <div class="flex h-full min-h-0 flex-col">
      <div class="flex flex-wrap items-end gap-2 border-b border-[var(--color-border-subtle)] px-4 py-3">
        <Field label="Entity filter" class="min-w-[14rem] flex-1">
          <Input
            value={props.entityFilter}
            placeholder="optional PPM entity id"
            onInput={(event) => props.onEntityFilterChange(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") void refresh();
            }}
          />
        </Field>
        <Button size="sm" variant="ghost" disabled={props.busy} onClick={() => void refresh()}>
          Apply filter
        </Button>
        <Button
          size="sm"
          variant="primary"
          disabled={props.busy || !props.projectId.trim()}
          onClick={() => {
            setFactDialogMode("create");
            setFactDraft({
              statement: "",
              truthClass: "declared",
              entityId: props.entityFilter.trim(),
            });
            setFactDialogOpen(true);
          }}
        >
          New fact
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={props.busy || !props.projectId.trim()}
          onClick={() => setEvidenceDialogOpen(true)}
        >
          New evidence
        </Button>
      </div>

      <div
        class={`grid min-h-0 flex-1 grid-cols-1 ${
          detail() ? "lg:grid-cols-[minmax(0,1.1fr)_minmax(0,0.9fr)]" : ""
        }`}
      >
        <div
          class={`min-h-0 overflow-auto ${
            detail()
              ? "border-b border-[var(--color-border-subtle)] lg:border-b-0 lg:border-r"
              : ""
          }`}
        >
          <DataTable.Root rows={facts()} getRowId={(row) => row.id}>
            <DataTable.Table aria-label="PPM facts">
              <DataTable.Header>
                <DataTable.ColumnHeader id="statement" sortBy={(row) => row.statement}>
                  Statement
                </DataTable.ColumnHeader>
                <DataTable.ColumnHeader id="truth" sortBy={(row) => row.truthClass}>
                  Truth
                </DataTable.ColumnHeader>
                <DataTable.ColumnHeader id="source" sortBy={(row) => row.source}>
                  Source
                </DataTable.ColumnHeader>
              </DataTable.Header>
              <DataTable.Body>
                {(row) => (
                  <DataTable.Row
                    row={row}
                    rowId={row.id}
                    class={
                      row.id === selectedId() ? "pw-data-table-row--active" : undefined
                    }
                    onActivate={() => {
                      setSelectedId(row.id);
                      void loadDetail(row.id);
                    }}
                    aria-label={`Open fact ${row.statement || row.id}`}
                  >
                    <DataTable.Cell>
                      <div class="font-medium">{row.statement || "—"}</div>
                      <div class="mt-0.5 font-mono text-[11px] text-[var(--color-text-muted)]">
                        {row.id}
                        {row.entityId ? ` · ${row.entityId}` : ""}
                      </div>
                    </DataTable.Cell>
                    <DataTable.Cell>
                      <StatusChip tone={truthTone(row.truthClass)} label={row.truthClass || "—"} />
                    </DataTable.Cell>
                    <DataTable.Cell>
                      <span class="truncate text-[11px] text-[var(--color-text-muted)]">
                        {row.source || "—"}
                      </span>
                    </DataTable.Cell>
                  </DataTable.Row>
                )}
              </DataTable.Body>
              <DataTable.Empty>
                <Show
                  when={!props.busy}
                  fallback={
                    <div class="pw-page-skeleton__rows p-3" role="status" aria-label="Loading…">
                      <For each={[0, 1, 2, 3]}>
                        {() => (
                          <div class="pw-page-skeleton__row">
                            <Skeleton class="h-3 w-[30%]" />
                            <Skeleton class="h-3 w-[20%]" />
                            <Skeleton class="ml-auto h-3 w-[12%]" />
                          </div>
                        )}
                      </For>
                    </div>
                  }
                >
                  <PageEmpty
                    title="No facts yet"
                    description="Import project sources or create a declared fact. Agent MCP writes also appear here."
                  />
                </Show>
              </DataTable.Empty>
            </DataTable.Table>
          </DataTable.Root>
        </div>

        <Show when={detail()}>
        <div class="min-h-0 overflow-auto p-4 text-[12.5px]">
          <Show when={detail()}>
            {(current) => (
              <div class="space-y-4">
                <div class="flex flex-wrap items-start justify-between gap-2">
                  <div>
                    <div class="text-[11px] uppercase tracking-wide text-[var(--color-text-muted)]">
                      Fact
                    </div>
                    <div class="mt-1 font-medium">{current().fact.statement}</div>
                    <div class="mt-2 flex flex-wrap gap-1.5">
                      <StatusChip
                        tone={truthTone(current().fact.truthClass)}
                        label={current().fact.truthClass || "—"}
                      />
                      <StatusChip tone="muted" label={current().fact.actor || "actor"} />
                    </div>
                  </div>
                  <div class="flex flex-wrap gap-1.5">
                    <Button
                      size="sm"
                      variant="ghost"
                      disabled={props.busy}
                      onClick={() => {
                        const fact = current().fact;
                        setFactDialogMode("edit");
                        setFactDraft({
                          id: fact.id,
                          statement: fact.statement,
                          truthClass: (["declared", "observed", "inferred", "verified"].includes(
                            fact.truthClass,
                          )
                            ? fact.truthClass
                            : "declared") as PpmFactTruthClass,
                          entityId: fact.entityId,
                        });
                        setFactDialogOpen(true);
                      }}
                    >
                      Edit
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      disabled={props.busy}
                      onClick={() => setLinkDialogOpen(true)}
                    >
                      Link evidence
                    </Button>
                    <Button
                      size="icon"
                      variant="icon"
                      aria-label="Close fact details"
                      onClick={() => {
                        setSelectedId("");
                        setDetail(undefined);
                      }}
                    >
                      <X size={14} />
                    </Button>
                  </div>
                </div>

                <div class="rounded-[var(--radius-md)] border border-[var(--color-border)] p-3">
                  <div class="text-[11px] uppercase tracking-wide text-[var(--color-text-muted)]">
                    Provenance
                  </div>
                  <dl class="mt-2 grid gap-2 sm:grid-cols-2">
                    <div>
                      <dt class="text-[11px] text-[var(--color-text-muted)]">Id</dt>
                      <dd class="m-0 font-mono pw-selectable">{current().fact.id}</dd>
                    </div>
                    <div>
                      <dt class="text-[11px] text-[var(--color-text-muted)]">Entity</dt>
                      <dd class="m-0 font-mono pw-selectable">{current().fact.entityId || "—"}</dd>
                    </div>
                    <div>
                      <dt class="text-[11px] text-[var(--color-text-muted)]">Source</dt>
                      <dd class="m-0 font-mono pw-selectable">{current().fact.source || "—"}</dd>
                    </div>
                    <div>
                      <dt class="text-[11px] text-[var(--color-text-muted)]">Updated</dt>
                      <dd class="m-0">{current().fact.updatedAt || "—"}</dd>
                    </div>
                  </dl>
                </div>

                <div>
                  <div class="mb-2 text-[11px] uppercase tracking-wide text-[var(--color-text-muted)]">
                    Linked evidence
                  </div>
                  <Show
                    when={current().evidence.length > 0}
                    fallback={
                      <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
                        No evidence linked to this fact.
                      </p>
                    }
                  >
                    <ul class="m-0 list-none space-y-2 p-0">
                      <For each={current().evidence}>
                        {(item) => (
                          <li class="rounded-[var(--radius-md)] border border-[var(--color-border-subtle)] px-3 py-2">
                            <div class="flex flex-wrap items-center gap-1.5">
                              <StatusChip tone="muted" label={item.kind || "other"} />
                              <span class="font-mono text-[11px] text-[var(--color-text-muted)]">
                                {item.id}
                              </span>
                            </div>
                            <div class="mt-1">{item.summary || "—"}</div>
                            <Show when={item.path}>
                              <div class="mt-0.5 font-mono text-[11px] text-[var(--color-text-muted)]">
                                {item.path}
                              </div>
                            </Show>
                          </li>
                        )}
                      </For>
                    </ul>
                  </Show>
                </div>
              </div>
            )}
          </Show>
        </div>
        </Show>
      </div>

      <PpmFactFormDialog
        open={factDialogOpen()}
        mode={factDialogMode()}
        initial={factDraft()}
        onClose={() => setFactDialogOpen(false)}
        onSubmit={(draft) => void saveFact(draft)}
      />
      <PpmEvidenceFormDialog
        open={evidenceDialogOpen()}
        canLinkToFact={Boolean(selected())}
        onClose={() => setEvidenceDialogOpen(false)}
        onSubmit={(draft) => void saveEvidence(draft)}
      />
      <PpmLinkEvidenceDialog
        open={linkDialogOpen()}
        evidence={projectEvidence()}
        linkedIds={(detail()?.evidence ?? []).map((item) => item.id)}
        busy={props.busy}
        onClose={() => setLinkDialogOpen(false)}
        onLink={(evidenceId) => void linkEvidence(evidenceId)}
      />
    </div>
  );
}
