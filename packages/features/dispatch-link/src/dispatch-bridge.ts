import type { StatusTone } from "@polterware/frontend-design-system";
import type {
  DispatchAgentView,
  DispatchHistoryEventKind,
  DispatchModelView,
  DispatchRunEventView,
  DispatchTierView,
} from "./dispatch-types";

/** The part of Polterware's `window.polterware.dispatch` a run's Route pill reads. */
export type RouteSourcesBridge = {
  listTiers: (payload?: Record<string, never>) => Promise<unknown>;
  listAgents: (payload?: Record<string, never>) => Promise<unknown>;
  /** Models the installed agent CLIs list; absent on cores that predate it. */
  listModels?: (payload?: Record<string, never>) => Promise<unknown>;
  /** Reads the routing policy (`get`); absent on cores that predate it. */
  config?: (payload: { action: string; dir?: string; project?: boolean; key?: string; values?: string[] }) => Promise<unknown>;
};

/** The sources Commander and a card's run read, while Polterware Dispatch is installed. */
export function routeSourcesBridge(): RouteSourcesBridge | undefined {
  const root = globalThis as { polterware?: { dispatch?: Partial<RouteSourcesBridge> } };
  const d = root.polterware?.dispatch;
  if (!d?.listTiers || !d.listAgents) return undefined;
  return { listTiers: d.listTiers, listAgents: d.listAgents, listModels: d.listModels, config: d.config };
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

function asOptionalNumber(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

const HISTORY_KINDS = new Set<DispatchHistoryEventKind>(["started", "merged", "failed"]);

export function isDispatchHistoryKind(value: string): value is DispatchHistoryEventKind {
  return HISTORY_KINDS.has(value as DispatchHistoryEventKind);
}

export function mapDispatchHistory(raw: unknown): DispatchRunEventView[] {
  const record = asRecord(raw);
  const events = Array.isArray(record.events) ? record.events : [];
  return events.flatMap((item) => {
    const row = asRecord(item);
    const id = asString(row.id);
    const kind = asString(row.kind);
    if (!id || !isDispatchHistoryKind(kind)) return [];
    return [{
      id,
      runId: asString(row.runId),
      projectId: asString(row.projectId),
      kind,
      agent: asString(row.agent),
      tier: asString(row.tier),
      text: asString(row.text),
      seq: asNumber(row.seq),
      at: asString(row.at),
    }];
  });
}

export function mapDispatchTiers(raw: unknown): DispatchTierView[] {
  const record = asRecord(raw);
  const tiers = Array.isArray(record.tiers) ? record.tiers : [];
  return tiers.flatMap((item) => {
    const row = asRecord(item);
    const name = asString(row.name);
    if (!name) return [];
    return [{
      name,
      provider: asString(row.provider),
      agent: asString(row.agent),
      model: asString(row.model),
      speed: asNumber(row.speed),
      strength: asNumber(row.strength),
      position: asNumber(row.position),
    }];
  });
}

export function mapDispatchModels(raw: unknown): DispatchModelView[] {
  const record = asRecord(raw);
  const models = Array.isArray(record.models) ? record.models : [];
  return models.flatMap((item) => {
    const row = asRecord(item);
    const agent = asString(row.agent);
    const model = asString(row.model);
    if (!agent || !model) return [];
    return [{ agent, model, provider: asString(row.provider), tier: asString(row.tier) }];
  });
}

export function mapDispatchAgents(raw: unknown): DispatchAgentView[] {
  const record = asRecord(raw);
  const agents = Array.isArray(record.agents) ? record.agents : [];
  return agents.flatMap((item) => {
    const row = asRecord(item);
    const name = asString(row.name);
    if (!name) return [];
    const usedPercent = asOptionalNumber(row.usedPercent);
    return [{
      name,
      bin: asString(row.bin),
      provider: asString(row.provider),
      ...(usedPercent !== undefined ? { usedPercent } : {}),
      cooldownUntil: asString(row.cooldownUntil),
      installed: row.installed === true,
    }];
  });
}

/** The status chip's tone for a run status or a history event, as Polterware Dispatch shows it. */
export function dispatchStatusTone(status: string): StatusTone {
  const normalized = status.toLowerCase();
  if (normalized === "merged" || normalized === "completed" || normalized === "passed" || normalized === "ok" || normalized === "ready") {
    return "ok";
  }
  if (
    normalized === "running"
    || normalized === "started"
    || normalized === "starting"
    || normalized === "queued"
    || normalized === "pending"
    || normalized === "active"
  ) {
    return "syncing";
  }
  if (normalized === "failed" || normalized === "error" || normalized === "blocked" || normalized === "orphaned") {
    return "danger";
  }
  if (normalized === "cancelled" || normalized === "canceled" || normalized === "timed_out" || normalized === "abandoned") {
    return "warn";
  }
  return "muted";
}
