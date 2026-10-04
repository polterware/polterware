import { vi } from "vitest";
import type { MemoriesBridge, MemoryJSON } from "./model";
import { memoryFixture } from "./fixtures";

/** All behavior fixtures use the same JSON input/output boundary as the Electron preload. */
export function fakeBridge(seed: MemoryJSON[] = [memoryFixture()]) {
  const data = new Map(seed.map(memory => [memory.id!, structuredClone(memory)]));
  let listener: Parameters<MemoriesBridge["onChange"]>[0] | undefined;
  function find(id?: string) { const memory = data.get(id ?? ""); if (!memory) throw new Error("Memory not found"); return memory; }
  function update(id: string | undefined, expectedRev: string | undefined, patch: Partial<MemoryJSON>) {
    const old = find(id);
    if (expectedRev && expectedRev !== old.rev) throw Object.assign(new Error("memory rev conflict"), { code: "conflict" });
    const next = { ...old, ...patch, rev: String(BigInt(old.rev ?? "0") + 1n), updatedAt: new Date().toISOString() };
    data.set(id!, next); return { memory: { ...structuredClone(next), checklist: [], tags: [], attachments: [], completions: [] } };
  }
  const bridge: MemoriesBridge = {
    memories: {
      list: vi.fn<MemoriesBridge["memories"]["list"]>(async input => {
        const all = [...data.values()]; const offset = Number(input?.cursor ?? 0), limit = input?.limit ?? 500;
        return { memories: all.slice(offset, offset + limit).map(memory => ({ ...memory, snippet: memory.note, tagIds: memory.tags?.map(tag => tag.id ?? "") })), total: all.length, nextCursor: offset + limit < all.length ? String(offset + limit) : "" };
      }),
      get: vi.fn<MemoriesBridge["memories"]["get"]>(async input => ({ memory: structuredClone(find(input?.id)) })),
      create: vi.fn<MemoriesBridge["memories"]["create"]>(async input => { const memory = { ...input, id: `created-${data.size}`, status: "active", rev: "1" }; data.set(memory.id, memory); return { memory }; }),
      update: vi.fn<MemoriesBridge["memories"]["update"]>(async input => {
        const { id, expectedRev, clearDueAt, clearPriority, clearMindId, ...patch } = input!;
        if (clearDueAt && patch.dueAt !== undefined || clearPriority && patch.priority !== undefined || clearMindId && patch.mindId !== undefined) throw new Error("cannot set and clear the same field");
        if (clearDueAt) patch.dueAt = undefined;
        if (clearPriority) patch.priority = undefined;
        if (clearMindId) patch.mindId = "";
        return update(id, expectedRev, patch);
      }),
      delete: vi.fn<MemoriesBridge["memories"]["delete"]>(async input => { const memory = find(input?.id); if (input?.expectedRev !== memory.rev) throw Object.assign(new Error("memory rev conflict"), { code: "conflict" }); data.delete(input!.id!); return {}; }),
      setStatus: vi.fn<MemoriesBridge["memories"]["setStatus"]>(async input => update(input?.id, input?.expectedRev, { status: input?.status })),
      setPinned: vi.fn<MemoriesBridge["memories"]["setPinned"]>(async input => update(input?.id, input?.expectedRev, { pinned: input?.pinned })),
      reorder: vi.fn<MemoriesBridge["memories"]["reorder"]>(async () => ({})),
      toggleCheckItem: vi.fn<MemoriesBridge["memories"]["toggleCheckItem"]>(async input => {
        const old = find(input?.memoryId);
        const checklist = (old.checklist ?? []).map(item => item.id === input?.itemId ? { ...item, done: input?.done ?? !item.done } : item);
        const patch: MemoryJSON = { checklist };
        if (old.autoCompleteOnChecklist) patch.status = checklist.length && checklist.every(item => item.done) ? "completed" : "active";
        const result = update(input?.memoryId, input?.expectedRev, patch);
        return { ...result, memory: { ...result.memory, note: "" }, item: checklist.find(item => item.id === input?.itemId) };
      }),
      replaceChecklist: vi.fn<MemoriesBridge["memories"]["replaceChecklist"]>(async input => {
        const items = (input?.items ?? []).map((item, index) => ({ ...item, id: item.id || `new-check-${index}` }));
        const old = find(input?.memoryId);
        const patch: MemoryJSON = { checklist: items };
        if (old.autoCompleteOnChecklist) patch.status = items.length > 0 && items.every(item => item.done) ? "completed" : "active";
        const result = update(input?.memoryId, input?.expectedRev, patch);
        return { ...result, memory: { ...result.memory, note: "" }, items };
      }),
      setTags: vi.fn<MemoriesBridge["memories"]["setTags"]>(async input => { const tags = input?.tagIds?.map(id => ({ id, name: id })); const result = update(input?.memoryId, input?.expectedRev, { tags }); return { ...result, memory: { ...result.memory, note: "" }, tags }; }),
      bulkAction: vi.fn<MemoriesBridge["memories"]["bulkAction"]>(async () => ({ results: [] })),
    },
    minds: { list: vi.fn(async () => ({ minds: [{ id: "mind-1", name: "Home" }] })), get: vi.fn(async () => ({})), create: vi.fn(async () => ({})), update: vi.fn(async () => ({})), delete: vi.fn(async () => ({})), reorder: vi.fn(async () => ({})) },
    tags: { list: vi.fn(async () => ({ tags: [{ id: "tag-1", name: "Errands" }] })), create: vi.fn(async () => ({})), update: vi.fn(async () => ({})), delete: vi.fn(async () => ({})), merge: vi.fn(async () => ({})) },
    activity: { list: vi.fn(async () => ({ events: [{ id: "activity-1", entityId: "memory-1", actor: "agent:codex", summary: "Updated memory", at: "2026-10-04T08:00:00Z" }] })) },
    schedule: { set: vi.fn(async () => ({})), clear: vi.fn(async () => ({})), preview: vi.fn(async () => ({})), listOccurrences: vi.fn(async () => ({ occurrences: [] })), completeOccurrence: vi.fn(async () => ({})), reopenOccurrence: vi.fn(async () => ({})), setFocusRecipe: vi.fn(async () => ({})) },
    onChange: vi.fn(callback => { listener = callback; return () => { listener = undefined; }; }),
    app: { openDeepLink: vi.fn(async () => {}) },
  };
  return { bridge, data, emit: (id = "memory-1") => listener?.({ resync: false, changes: [{ entity: "memory", id, op: "update" }] }) };
}
