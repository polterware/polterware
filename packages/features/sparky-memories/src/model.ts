// oxlint-disable-next-line anti-slop/no-shape-in-symbol-names -- MessageShape is the protobuf library export.
import type { DescMessage, MessageShape as ContractMessage } from "@bufbuild/protobuf";
import type { MemorySchema, MemorySummarySchema, MindSchema, TagSchema, ActivityEventSchema, OccurrenceSchema, UpdateMemoryRequestSchema, CheckItemInputSchema, CreateMemoryRequestSchema, MemoryService, MindService, TagService, ActivityService, ScheduleService } from "@polterware/sparky-contracts";
import type { DescMethodUnary } from "@bufbuild/protobuf";

/** Protobuf JSON: int64 is a decimal string, timestamps are RFC 3339. */
export type Wire<T> = T extends { $typeName: "google.protobuf.Timestamp" } ? string
  : T extends bigint ? string
  : T extends readonly (infer V)[] ? Wire<V>[]
  : T extends object ? { [K in keyof T as K extends "$typeName" ? never : K]?: Wire<T[K]> }
  : T;
export type JsonOf<S extends DescMessage> = Wire<ContractMessage<S>>;
type Namespace<S extends { method: object }> = {
  [K in keyof S["method"]]: S["method"][K] extends DescMethodUnary
    ? (input?: Wire<ContractMessage<S["method"][K]["input"]>>) => Promise<Wire<ContractMessage<S["method"][K]["output"]>>> : never;
};
export type MemoriesBridge = {
  memories: Namespace<typeof MemoryService>;
  minds: Namespace<typeof MindService>;
  tags: Namespace<typeof TagService>;
  activity: Namespace<typeof ActivityService>;
  schedule: Namespace<typeof ScheduleService>;
  onChange(callback: (event: { resync: boolean; changes: { entity: string; id: string; op: string }[] }) => void): () => void;
  app: { openDeepLink(url: string): Promise<void>; onDeepLink?(callback: (link: { url: string; route: string }) => void): () => void; getDeepLink?(): Promise<{ url: string; route: string } | null> };
};
export type MemoryJSON = JsonOf<typeof MemorySchema>;
export type SummaryJSON = JsonOf<typeof MemorySummarySchema>;
export type Patch = Omit<JsonOf<typeof UpdateMemoryRequestSchema>, "id" | "expectedRev">;
export type ChecklistInput = JsonOf<typeof CheckItemInputSchema>;
export type OccurrenceJSON = JsonOf<typeof OccurrenceSchema>;
export type CreateMemoryJSON = JsonOf<typeof CreateMemoryRequestSchema>;
export type MindJSON = JsonOf<typeof MindSchema>;
export type TagJSON = JsonOf<typeof TagSchema>;
export type ActivityJSON = JsonOf<typeof ActivityEventSchema>;

export function revisionString(rev: string | number | bigint | undefined): string {
  if (typeof rev === "string" && rev !== "") return rev;
  if (typeof rev === "bigint") return rev.toString();
  if (typeof rev === "number" && Number.isFinite(rev)) return Math.trunc(rev).toString();
  return "0";
}

export type MemoryRow = {
  id: string; title: string; note: string; status: string; mindId: string; pinned: boolean;
  priority?: number; dueAt?: string; dueAllDay: boolean; rev: string; tagIds: string[];
  createdAt?: string; updatedAt?: string; sortOrder: number; scheduleLabel: string;
};
export type MemoryRecord = MemoryRow & {
  checklist: NonNullable<MemoryJSON["checklist"]>; tags: NonNullable<MemoryJSON["tags"]>;
  autoCompleteOnChecklist: boolean; schedule?: MemoryJSON["schedule"]; place?: MemoryJSON["place"];
  attachments: NonNullable<MemoryJSON["attachments"]>; completions: NonNullable<MemoryJSON["completions"]>;
};
export function rowFromJSON(row: SummaryJSON | MemoryJSON): MemoryRow {
  const summary = row as SummaryJSON;
  const memory = row as MemoryJSON;
  return {
    id: row.id ?? "", title: row.title ?? "", note: summary.snippet ?? memory.note ?? "", status: row.status ?? "active",
    mindId: row.mindId ?? "", pinned: row.pinned ?? false, priority: row.priority, dueAt: row.dueAt,
    dueAllDay: row.dueAllDay ?? false, rev: revisionString(row.rev), tagIds: summary.tagIds ?? (memory.tags ?? []).map(tag => tag.id ?? ""),
    createdAt: row.createdAt, updatedAt: row.updatedAt, sortOrder: row.sortOrder ?? 0, scheduleLabel: summary.scheduleLabel ?? "",
  };
}
export function recordFromJSON(row: MemoryJSON): MemoryRecord {
  return { ...rowFromJSON(row), note: row.note ?? "", checklist: row.checklist ?? [], tags: row.tags ?? [],
    tagIds: (row.tags ?? []).map(tag => tag.id ?? ""), autoCompleteOnChecklist: row.autoCompleteOnChecklist ?? false,
    schedule: row.schedule, place: row.place, attachments: row.attachments ?? [], completions: row.completions ?? [] };
}
export function mergePatch(previous: Patch, incoming: Patch): Patch {
  const merged = { ...previous, ...incoming };
  for (const [field, clear] of [["mindId", "clearMindId"], ["priority", "clearPriority"], ["dueAt", "clearDueAt"]] as const) {
    if (incoming[clear] === true) delete merged[field];
    else if (incoming[field] !== undefined) merged[clear] = false;
  }
  return merged;
}
export function applyPatch<T extends MemoryRow>(row: T, patch: Patch): T {
  return { ...row, ...patch, mindId: patch.clearMindId ? "" : patch.mindId ?? row.mindId,
    priority: patch.clearPriority ? undefined : patch.priority ?? row.priority,
    dueAt: patch.clearDueAt ? undefined : patch.dueAt ?? row.dueAt };
}
export type ListFilter = { query: string; status: string; pinned: boolean; tagId: string; order: string; mindId?: string; inbox?: boolean };
export const DEFAULT_FILTER: ListFilter = { query: "", status: "active", pinned: false, tagId: "", order: "due" };
export function filterRows(rows: readonly MemoryRow[], filter: ListFilter): MemoryRow[] {
  const query = filter.query.trim().toLocaleLowerCase();
  const result = rows.filter(row => (!filter.status || row.status === filter.status) && (!filter.inbox || !row.mindId)
    && (filter.mindId === undefined || row.mindId === filter.mindId) && (!filter.pinned || row.pinned)
    && (!filter.tagId || row.tagIds.includes(filter.tagId))
    && (!query || row.title.toLocaleLowerCase().includes(query) || row.note.toLocaleLowerCase().includes(query)));
  const compare = filter.order === "title" ? (a: MemoryRow, b: MemoryRow) => a.title.localeCompare(b.title)
    : filter.order === "updated" ? (a: MemoryRow, b: MemoryRow) => (b.updatedAt ?? "").localeCompare(a.updatedAt ?? "")
    : (a: MemoryRow, b: MemoryRow) => (a.dueAt ?? "9999").localeCompare(b.dueAt ?? "9999") || a.sortOrder - b.sortOrder;
  return result.sort((a, b) => Number(b.pinned) - Number(a.pinned) || compare(a, b));
}
export function todayGroup(row: Pick<MemoryRow, "dueAt" | "dueAllDay">, now: Date): "Overdue" | "Today" | "Upcoming 7 days" | undefined {
  if (!row.dueAt) return undefined;
  const due = new Date(row.dueAt);
  const date = row.dueAllDay ? row.dueAt.slice(0, 10) : localDate(due);
  const today = localDate(now);
  const end = new Date(now); end.setDate(end.getDate() + 7);
  if (date < today) return "Overdue";
  if (date === today) return "Today";
  if (date <= localDate(end)) return "Upcoming 7 days";
  return undefined;
}
export function localDate(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}
export function checklistInputs(items: MemoryRecord["checklist"]): ChecklistInput[] {
  return items.map((item, index) => ({ id: item.id, title: item.title ?? "", detail: item.detail ?? "", done: item.done ?? false, sortOrder: index }));
}
export function reorderChecklist(items: ChecklistInput[], index: number, direction: -1 | 1): ChecklistInput[] {
  const target = index + direction;
  if (target < 0 || target >= items.length) return items;
  const next = [...items]; [next[index], next[target]] = [next[target]!, next[index]!];
  return next.map((item, sortOrder) => ({ ...item, sortOrder }));
}
export function errorText(error: unknown): string { return error instanceof Error ? error.message : String(error); }
export function isConflict(error: unknown): boolean {
  const code = error && typeof error === "object" && "code" in error ? error.code : undefined;
  return code === 9 || code === 10 || code === "aborted" || code === "conflict" || code === "failed_precondition"
    || /\b(conflict|aborted|revision)\b/i.test(errorText(error));
}
export type MemoryFailure = { message: string; conflict: boolean };
export function parseMemoryFailure(error: unknown): MemoryFailure { return { message: errorText(error), conflict: isConflict(error) }; }
