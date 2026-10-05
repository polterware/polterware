import { batch, createSignal } from "solid-js";
import { browserBridge } from "./bridge";
import { applyPatch, errorText, parseMemoryFailure, mergePatch, recordFromJSON, revisionString, rowFromJSON, type ActivityJSON, type ChecklistInput, type CreateMemoryJSON, type MemoriesBridge, type MemoryFailure, type MemoryJSON, type MemoryRecord, type MemoryRow, type MindJSON, type OccurrenceJSON, type Patch, type TagJSON } from "./model";

export type SaveState = "live" | "queued" | "saving" | "failed" | "conflict";
type Draft = { patch: Patch; checklist?: ChecklistInput[]; tagIds?: string[] };
type Undo = { id: string; kind: "delete" | "complete"; message: string; expiresAt: number; run: () => Promise<void>; timer: ReturnType<typeof setTimeout> };
const INCLUDE = { note: true, checklist: true, tags: true, schedule: true, place: true, attachments: true, completions: true };

/** One entity cache; edits survive detail unmounts and every write uses the newest acknowledged rev. */
export function createMemoriesStore(getBridge: () => MemoriesBridge = browserBridge, debounceMs = 350) {
  const [rows, setRows] = createSignal<MemoryRow[]>([]);
  const [records, setRecords] = createSignal<Record<string, MemoryRecord>>({});
  const [drafts, setDrafts] = createSignal<Record<string, Draft>>({});
  const [inflight, setInflight] = createSignal<Record<string, Draft>>({});
  const [optimistic, setOptimistic] = createSignal<Record<string, Partial<MemoryRecord>>>({});
  const [checkToggles, setCheckToggles] = createSignal<Record<string, Record<string, boolean>>>({});
  const [states, setStates] = createSignal<Record<string, SaveState>>({});
  const [errors, setErrors] = createSignal<Record<string, string>>({});
  const [minds, setMinds] = createSignal<MindJSON[]>([]);
  const [tags, setTags] = createSignal<TagJSON[]>([]);
  const [activity, setActivity] = createSignal<ActivityJSON[]>([]);
  const [occurrences, setOccurrences] = createSignal<OccurrenceJSON[]>([]);
  const [scheduleError, setScheduleError] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [loadError, setLoadError] = createSignal("");
  const [metadataError, setMetadataError] = createSignal("");
  const [undos, setUndos] = createSignal<Undo[]>([]);
  const [deleting, setDeleting] = createSignal<ReadonlySet<string>>(new Set());
  const timers = new Map<string, ReturnType<typeof setTimeout>>();
  const queues = new Map<string, Promise<void>>();
  const toggleVersions = new Map<string, number>();
  let generation = 0, unsubscribe: (() => void) | undefined, users = 0;
  let refreshTimer: ReturnType<typeof setTimeout> | undefined;
  const state = (id: string) => states()[id] ?? "live";
  const report = (id: string, error: MemoryFailure) => batch(() => {
    setStates(v => ({ ...v, [id]: error.conflict ? "conflict" : "failed" }));
    setErrors(v => ({ ...v, [id]: error.message }));
  });
  function accept(json: MemoryJSON, full = false, relations: Pick<MemoryJSON, "checklist" | "tags"> = {}) {
    if (!json.id) throw new Error("The core returned a memory without an ID.");
    const old = records()[json.id];
    if (old && BigInt(old.rev) > BigInt(revisionString(json.rev))) return old;
    const record = recordFromJSON(full ? json : { ...old, ...json,
      checklist: relations.checklist ?? old?.checklist, tags: relations.tags ?? old?.tags,
      schedule: old?.schedule, place: old?.place,
      attachments: old?.attachments, completions: old?.completions });
    batch(() => {
      setRecords(v => ({ ...v, [record.id]: record }));
      setRows(v => { const exists = v.some(row => row.id === record.id); return exists ? v.map(row => row.id === record.id ? record : row) : [...v, record]; });
    });
    return record;
  }
  async function load(id: string, force = false) {
    if (records()[id] && !force) return records()[id]!;
    const result = await getBridge().memories.get({ id, include: INCLUDE });
    if (!result.memory) throw new Error("Memory unavailable. It may have been deleted.");
    return accept(result.memory, true);
  }
  function overlay<T extends MemoryRow>(row: T): T {
    let next = { ...row, ...optimistic()[row.id] };
    for (const draft of [inflight()[row.id], drafts()[row.id]]) {
      if (!draft) continue;
      next = applyPatch(next, draft.patch);
      if (draft.tagIds) next = { ...next, tagIds: draft.tagIds };
      if (draft.checklist) next = { ...next, checklist: draft.checklist };
    }
    return next;
  }
  const record = (id: string) => {
    const value = records()[id]; if (!value) return undefined;
    const next = overlay(value), toggles = checkToggles()[id];
    return toggles ? { ...next, checklist: next.checklist.map(item => item.id && item.id in toggles ? { ...item, done: toggles[item.id] } : item) } : next;
  };
  const visibleRows = () => rows().filter(row => !deleting().has(row.id)).map(overlay);
  function enqueue(id: string, action: () => Promise<void>) {
    const previous = queues.get(id) ?? Promise.resolve();
    const next = previous.then(action);
    queues.set(id, next);
    void next.finally(() => { if (queues.get(id) === next) queues.delete(id); }).catch(() => {});
    return next;
  }
  function mergeDraft(id: string, value: Draft) {
    setDrafts(v => ({ ...v, [id]: { ...value, ...v[id], patch: mergePatch(value.patch, v[id]?.patch ?? {}) } }));
  }
  async function saveDraft(id: string) {
    if (state(id) === "conflict" || state(id) === "failed") return;
    const draft = drafts()[id];
    if (!draft) return;
    const remaining = { ...draft, patch: { ...draft.patch } };
    batch(() => {
      setDrafts(v => { const next = { ...v }; delete next[id]; return next; });
      setInflight(v => ({ ...v, [id]: draft }));
      setStates(v => ({ ...v, [id]: "saving" }));
    });
    try {
      await load(id);
      if (Object.keys(remaining.patch).length) {
        const response = await getBridge().memories.update({ id, expectedRev: revisionString(records()[id]!.rev), ...remaining.patch });
        if (!response.memory) throw new Error("The core did not acknowledge the edit.");
        accept({ ...applyPatch(records()[id]!, remaining.patch), ...response.memory, note: remaining.patch.note ?? response.memory.note ?? records()[id]!.note }); remaining.patch = {};
      }
      if (remaining.checklist) {
        const response = await getBridge().memories.replaceChecklist({ memoryId: id, expectedRev: revisionString(records()[id]!.rev), items: remaining.checklist });
        if (!response.memory) throw new Error("The core did not acknowledge the checklist.");
        accept({ ...response.memory, note: records()[id]!.note }, false, { checklist: response.items ?? [] }); delete remaining.checklist;
      }
      if (remaining.tagIds) {
        const response = await getBridge().memories.setTags({ memoryId: id, expectedRev: revisionString(records()[id]!.rev), tagIds: remaining.tagIds });
        if (!response.memory) throw new Error("The core did not acknowledge the tags.");
        accept({ ...response.memory, note: records()[id]!.note }, false, { tags: response.tags ?? [] }); delete remaining.tagIds;
      }
      setStates(v => ({ ...v, [id]: drafts()[id] ? "queued" : "live" }));
      setErrors(v => ({ ...v, [id]: "" }));
    } catch (error) {
      mergeDraft(id, remaining); report(id, parseMemoryFailure(error));
    } finally {
      setInflight(v => { const next = { ...v }; delete next[id]; return next; });
    }
  }
  function edit(id: string, value: Partial<Draft>) {
    if (deleting().has(id)) return;
    setDrafts(v => ({ ...v, [id]: { ...v[id], ...value, patch: mergePatch(v[id]?.patch ?? {}, value.patch ?? {}) } }));
    if (state(id) === "conflict" || state(id) === "failed") return;
    setStates(v => ({ ...v, [id]: "queued" }));
    const old = timers.get(id); if (old) clearTimeout(old);
    timers.set(id, setTimeout(() => { timers.delete(id); void flush(id); }, debounceMs));
  }
  async function flush(id: string) {
    const timer = timers.get(id); if (timer) clearTimeout(timer); timers.delete(id);
    await enqueue(id, () => saveDraft(id));
  }
  async function resolve(id: string, mode: "reload" | "overwrite") {
    await enqueue(id, async () => {
      try {
        await load(id, true);
        if (mode === "reload") setDrafts(v => { const next = { ...v }; delete next[id]; return next; });
        setStates(v => ({ ...v, [id]: "live" })); setErrors(v => ({ ...v, [id]: "" }));
        if (mode === "overwrite") await saveDraft(id);
      } catch (error) { report(id, parseMemoryFailure(error)); }
    });
  }
  function addUndo(id: string, kind: "delete" | "complete", message: string, run: () => Promise<void>, expire?: () => Promise<void>) {
    const timer = setTimeout(() => {
      setUndos(v => v.filter(undo => undo.timer !== timer));
      void expire?.();
    }, 8000);
    setUndos(v => [...v, { id, kind, message, run, timer, expiresAt: Date.now() + 8000 }]);
  }
  function cancelQueuedDeletes() {
    const pending = undos().filter(item => item.kind === "delete");
    if (!pending.length) return;
    pending.forEach(item => clearTimeout(item.timer));
    batch(() => {
      setUndos(v => v.filter(item => item.kind !== "delete"));
      setDeleting(new Set<string>());
    });
  }
  async function undo(item: Undo) {
    if (Date.now() >= item.expiresAt || !undos().includes(item)) return;
    clearTimeout(item.timer); setUndos(v => v.filter(value => value !== item));
    try { await item.run(); } catch (error) { report(item.id, parseMemoryFailure(error)); }
  }
  async function toggle(id: string, kind: "status" | "pinned", explicit?: boolean) {
    const row = visibleRows().find(row => row.id === id);
    if (!row || deleting().has(id)) return;
    const value = explicit ?? (kind === "pinned" ? !row.pinned : row.status !== "completed");
    const toggleKey = `${id}:${kind}`;
    const version = (toggleVersions.get(toggleKey) ?? 0) + 1; toggleVersions.set(toggleKey, version);
    const patch = kind === "pinned" ? { pinned: value } : { status: value ? "completed" : "active" };
    setOptimistic(v => ({ ...v, [id]: { ...v[id], ...patch } }));
    await enqueue(id, async () => {
      try {
        await saveDraft(id);
        if (state(id) === "conflict" || state(id) === "failed") throw new Error("Resolve the unsaved edit before changing this memory.");
        await load(id); setStates(v => ({ ...v, [id]: "saving" }));
        const expectedRev = revisionString(records()[id]!.rev);
        const response = kind === "pinned"
          ? await getBridge().memories.setPinned({ id, pinned: value, expectedRev })
          : await getBridge().memories.setStatus({ id, status: value ? "completed" : "active", expectedRev });
        if (!response.memory) throw new Error("The core did not acknowledge the action.");
        accept({ ...response.memory, ...patch });
        if (kind === "status") await load(id, true);
        setStates(v => ({ ...v, [id]: drafts()[id] ? "queued" : "live" }));
        if (kind === "status" && value) addUndo(id, "complete", `Completed “${row.title}”`, () => toggle(id, "status", false));
      } catch (error) { if (state(id) !== "conflict") report(id, parseMemoryFailure(error)); }
      finally {
        // Later queued toggles still carry their own value into the serialized write.
        if (toggleVersions.get(toggleKey) === version) setOptimistic(v => { const next = { ...v, [id]: { ...v[id] } }; delete next[id]![kind === "pinned" ? "pinned" : "status"]; return next; });
      }
    });
  }
  async function toggleChecklist(id: string, itemId: string, done: boolean) {
    const key = `${id}:check:${itemId}`;
    const version = (toggleVersions.get(key) ?? 0) + 1; toggleVersions.set(key, version);
    setCheckToggles(v => ({ ...v, [id]: { ...v[id], [itemId]: done } }));
    await enqueue(id, async () => {
      try {
        await saveDraft(id);
        if (state(id) === "conflict" || state(id) === "failed") throw new Error("Resolve the unsaved edit before toggling a checklist item.");
        await load(id); setStates(v => ({ ...v, [id]: "saving" }));
        const response = await getBridge().memories.toggleCheckItem({ memoryId: id, itemId, done, expectedRev: revisionString(records()[id]!.rev) });
        if (!response.memory) throw new Error("The core did not acknowledge the checklist action.");
        accept({ ...response.memory, note: records()[id]!.note });
        await load(id, true);
        setStates(v => ({ ...v, [id]: drafts()[id] ? "queued" : "live" }));
      } catch (error) { if (state(id) !== "conflict") report(id, parseMemoryFailure(error)); }
      finally {
        if (toggleVersions.get(key) === version) setCheckToggles(v => {
          const next = { ...v, [id]: { ...v[id] } }; delete next[id]![itemId]; return next;
        });
      }
    });
  }
  async function queueDelete(id: string) {
    await flush(id);
    if (state(id) === "conflict" || state(id) === "failed") throw new Error("Resolve unsaved edits before deleting.");
    const snapshot = await load(id);
    if (deleting().has(id)) return;
    setDeleting(v => new Set([...v, id]));
    addUndo(id, "delete", `Deletion queued: “${snapshot.title}”`, async () => { setDeleting(v => { const next = new Set(v); next.delete(id); return next; }); }, async () => {
      await enqueue(id, async () => {
        try {
          await getBridge().memories.delete({ id, expectedRev: revisionString(snapshot.rev) });
          batch(() => {
            setRows(v => v.filter(row => row.id !== id));
            setRecords(v => { const next = { ...v }; delete next[id]; return next; });
          });
        } catch (error) { report(id, parseMemoryFailure(error)); }
        finally { setDeleting(v => { const next = new Set(v); next.delete(id); return next; }); }
      });
    });
  }
  async function create(input: CreateMemoryJSON) {
    const response = await getBridge().memories.create(input);
    if (!response.memory) throw new Error("The core did not acknowledge the new memory.");
    return accept(response.memory, true);
  }
  async function refresh() {
    const current = ++generation; setLoading(true); setLoadError("");
    try {
      const bridge = getBridge(), all: MemoryRow[] = [], seen = new Set<string>();
      let cursor = "";
      do {
        const result = await bridge.memories.list({ limit: 500, cursor, order: "updated" });
        if (current !== generation) return;
        all.push(...(result.memories ?? []).map(rowFromJSON)); cursor = result.nextCursor ?? "";
        if (cursor && seen.has(cursor)) throw new Error("The core returned a repeated list cursor.");
        seen.add(cursor);
      } while (cursor);
      if (current === generation) {
        setRows(all.map(row => { const cached = records()[row.id]; return cached && BigInt(cached.rev) >= BigInt(revisionString(row.rev)) ? cached : row; }));
        // Do not overwrite unsaved drafts; clean details are invalidated by the same event.
        await Promise.all(Object.keys(records()).filter(id => !drafts()[id] && !inflight()[id] && !queues.has(id) && !deleting().has(id)).map(id => load(id, true).catch(error => report(id, parseMemoryFailure(error)))));
      }
    } catch (error) { if (current === generation) setLoadError(errorText(error)); }
    finally { if (current === generation) setLoading(false); }
  }
  async function refreshMetadata() {
    const now = new Date(), from = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const to = new Date(from); to.setDate(to.getDate() + 8);
    void getBridge().schedule.listOccurrences({ from: from.toISOString(), to: to.toISOString(), limit: 10000 }).then(result => { setOccurrences(result.occurrences ?? []); setScheduleError(""); }).catch(error => setScheduleError(errorText(error)));
    const results = await Promise.allSettled([getBridge().minds.list({}), getBridge().tags.list({}), getBridge().activity.list({ limit: 100 })]);
    batch(() => {
      if (results[0].status === "fulfilled") setMinds(results[0].value.minds ?? []);
      if (results[1].status === "fulfilled") setTags(results[1].value.tags ?? []);
      if (results[2].status === "fulfilled") setActivity(results[2].value.events ?? []);
      setMetadataError(results.filter(value => value.status === "rejected").map(value => errorText(value.reason)).join("; "));
    });
  }
  const onPageHide = () => cancelQueuedDeletes();
  function connect() {
    users++;
    if (!unsubscribe) {
      try {
        unsubscribe = getBridge().onChange(event => {
          if (!event.resync && !event.changes.some(change => ["memory", "memories", "check_item", "checklist", "tag", "mind", "activity", "schedule"].includes(change.entity))) return;
          if (refreshTimer) clearTimeout(refreshTimer);
          refreshTimer = setTimeout(() => { void refresh(); void refreshMetadata().catch(error => setMetadataError(errorText(error))); }, 100);
        });
        window.addEventListener("pagehide", onPageHide);
        void refresh(); void refreshMetadata().catch(error => setMetadataError(errorText(error)));
      } catch (error) { setLoadError(errorText(error)); }
    }
    return () => {
      users--;
      if (users === 0) {
        unsubscribe?.(); unsubscribe = undefined; if (refreshTimer) clearTimeout(refreshTimer);
        window.removeEventListener("pagehide", onPageHide);
      }
      // Draft saves and undo timers intentionally survive route/detail unmounts.
    };
  }
  function dispose() {
    generation++; unsubscribe?.(); unsubscribe = undefined;
    window.removeEventListener("pagehide", onPageHide);
    if (refreshTimer) clearTimeout(refreshTimer);
    timers.forEach(clearTimeout); timers.clear();
    cancelQueuedDeletes();
    undos().forEach(item => clearTimeout(item.timer));
    setUndos([]);
  }
  return { rows: visibleRows, record, state, errors, drafts, minds, tags, activity, loading, loadError, metadataError, occurrences, scheduleError, undos, deleting,
    load, edit: (id: string, patch: Patch) => edit(id, { patch }), editChecklist: (id: string, checklist: ChecklistInput[]) => edit(id, { checklist }),
    editTags: (id: string, tagIds: string[]) => edit(id, { tagIds }), flush, resolve, toggle, toggleChecklist, queueDelete, create, refresh, refreshMetadata, connect, undo, cancelQueuedDeletes, dispose };
}
export type MemoriesStore = ReturnType<typeof createMemoriesStore>;
export const memoriesStore = createMemoriesStore();
