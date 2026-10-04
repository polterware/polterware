import { afterEach, describe, expect, it, vi } from "vitest";
import { createMemoriesStore, type MemoriesStore } from "./store";
import { fakeBridge } from "./fake-bridge";
import { memoryFixture } from "./fixtures";
import type { MemoriesBridge } from "./model";

const stores: MemoriesStore[] = [];
function setup(count = 1) {
  const fake = fakeBridge(Array.from({ length: count }, (_, index) => memoryFixture(index + 1)));
  const store = createMemoriesStore(() => fake.bridge); stores.push(store);
  return { ...fake, store };
}
afterEach(() => { stores.splice(0).forEach(store => store.dispose()); vi.useRealTimers(); });
function deferred<T>() { let resolve!: (value: T) => void; const promise = new Promise<T>(done => { resolve = done; }); return { promise, resolve }; }

describe("revision-safe memory store", () => {
  it("reads every cursor page of 10k records", async () => {
    const { store, bridge } = setup(10000); await store.refresh();
    expect(store.rows()).toHaveLength(10000); expect(bridge.memories.list).toHaveBeenCalledTimes(20);
  });
  it("serializes rapid edits and preserves the newer draft while a save is in flight", async () => {
    const { store, bridge, data } = setup(); await store.load("memory-1");
    const first = deferred<Awaited<ReturnType<MemoriesBridge["memories"]["update"]>>>();
    const update = bridge.memories.update;
    vi.mocked(update).mockImplementationOnce(async input => {
      const result = { ...data.get(input!.id!)!, title: input!.title, note: input!.note, rev: "2" };
      await first.promise; data.set("memory-1", result); return { memory: result };
    });
    store.edit("memory-1", { title: "First", note: "First note" });
    const one = store.flush("memory-1"); await vi.waitFor(() => expect(update).toHaveBeenCalledTimes(1));
    store.edit("memory-1", { title: "Final", note: "Final note" });
    const two = store.flush("memory-1");
    expect(store.record("memory-1")!.note).toBe("Final note");
    first.resolve({}); await Promise.all([one, two]);
    expect(data.get("memory-1")!.title).toBe("Final"); expect(data.get("memory-1")!.note).toBe("Final note");
    expect(vi.mocked(update).mock.calls[1]![0]!.expectedRev).toBe("2"); expect(store.state("memory-1")).toBe("live");
  });
  it("retains the latest local draft on conflict, then overwrites against the reloaded rev", async () => {
    const { store, data, bridge } = setup(); await store.load("memory-1");
    data.set("memory-1", { ...data.get("memory-1")!, note: "Remote edit", rev: "2" });
    store.edit("memory-1", { note: "Local edit" }); await store.flush("memory-1");
    expect(store.state("memory-1")).toBe("conflict"); expect(store.record("memory-1")!.note).toBe("Local edit");
    store.edit("memory-1", { note: "Newest local edit" }); await store.resolve("memory-1", "overwrite");
    expect(data.get("memory-1")!.note).toBe("Newest local edit");
    expect(vi.mocked(bridge.memories.update).mock.calls.at(-1)![0]!.expectedRev).toBe("2");
  });
  it("reload discards only the conflicted entity's draft", async () => {
    const { store, data } = setup(2); await store.load("memory-1"); await store.load("memory-2");
    data.set("memory-1", { ...data.get("memory-1")!, note: "Remote", rev: "2" });
    store.edit("memory-1", { note: "Local" }); store.edit("memory-2", { note: "Other local" });
    await store.flush("memory-1"); await store.resolve("memory-1", "reload");
    expect(store.record("memory-1")!.note).toBe("Remote"); expect(store.record("memory-2")!.note).toBe("Other local");
  });
  it("keeps rapid checkbox/detail/tag edits and saves each against the preceding revision", async () => {
    const { store, data, bridge } = setup(); await store.load("memory-1");
    store.edit("memory-1", { autoCompleteOnChecklist: true });
    store.editChecklist("memory-1", [{ id: "check-1", title: "Final", detail: "Retained detail", done: true }]);
    store.editTags("memory-1", ["tag-2"]); await store.flush("memory-1");
    expect(store.record("memory-1")!.note).toBe("Note 1"); expect(data.get("memory-1")!.status).toBe("completed"); expect(data.get("memory-1")!.checklist![0]!.detail).toBe("Retained detail");
    expect(vi.mocked(bridge.memories.replaceChecklist).mock.calls[0]![0]!.expectedRev).toBe("2");
    expect(vi.mocked(bridge.memories.setTags).mock.calls[0]![0]!.expectedRev).toBe("3");
  });
  it("preserves note and tags when real mutation responses emit empty unloaded relations", async () => {
    const { store, bridge, data } = setup(); await store.load("memory-1");
    vi.mocked(bridge.memories.setPinned).mockImplementationOnce(async () => ({ memory: { id: "memory-1", pinned: true, rev: "2", checklist: [], tags: [], attachments: [], completions: [] } }));
    await store.toggle("memory-1", "pinned");
    expect(store.record("memory-1")!.note).toBe("Note 1"); expect(store.record("memory-1")!.tagIds).toEqual(["tag-1"]);
    data.set("memory-1", { ...data.get("memory-1")!, rev: "2", pinned: true });
    vi.mocked(bridge.memories.setPinned).mockImplementationOnce(async () => ({ memory: { id: "memory-1", rev: "3" } }));
    await store.toggle("memory-1", "pinned", false); expect(store.record("memory-1")!.pinned).toBe(false);
  });
  it("rolls back a failed optimistic pin without losing unrelated draft text", async () => {
    const { store, bridge } = setup(); await store.load("memory-1"); await store.refresh();
    vi.mocked(bridge.memories.setPinned).mockRejectedValueOnce(new Error("Core unavailable"));
    const task = store.toggle("memory-1", "pinned"); expect(store.rows()[0]!.pinned).toBe(true); await task;
    expect(store.rows()[0]!.pinned).toBe(false); expect(store.state("memory-1")).toBe("failed");
  });
  it("undo reopens a completion with expectedRev", async () => {
    const { store, data, bridge } = setup(); await store.refresh(); await store.toggle("memory-1", "status");
    expect(data.get("memory-1")!.status).toBe("completed");
    await store.undo(store.undos()[0]!); expect(data.get("memory-1")!.status).toBe("active");
    expect(vi.mocked(bridge.memories.setStatus).mock.calls.at(-1)![0]!.expectedRev).toBe("2");
  });
  it("undo cancels queued deletion and preserves the complete aggregate", async () => {
    vi.useFakeTimers(); const { store, data, bridge } = setup();
    const memory = data.get("memory-1")!; memory.attachments = [{ id: "attachment-1", memoryId: "memory-1" }];
    await store.refresh(); await store.queueDelete("memory-1"); expect(store.rows()).toHaveLength(0);
    await store.undo(store.undos()[0]!); await vi.advanceTimersByTimeAsync(8001);
    expect(bridge.memories.delete).not.toHaveBeenCalled(); expect(data.get("memory-1")!.attachments).toHaveLength(1); expect(store.rows()).toHaveLength(1);
  });
  it("runs deletion after 8 seconds, with a checked revision, and restores a failed row", async () => {
    vi.useFakeTimers(); const { store, bridge } = setup(); await store.refresh();
    vi.mocked(bridge.memories.delete).mockRejectedValueOnce(new Error("Core unavailable"));
    await store.queueDelete("memory-1"); await vi.advanceTimersByTimeAsync(7999); expect(bridge.memories.delete).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1); expect(bridge.memories.delete).toHaveBeenCalledWith({ id: "memory-1", expectedRev: "1" });
    expect(store.rows()).toHaveLength(1); expect(store.state("memory-1")).toBe("failed");
  });
  it("refreshes live changes while keeping an unsaved editor draft", async () => {
    vi.useFakeTimers(); const { store, data, emit } = setup(); const disconnect = store.connect();
    await store.refresh(); await store.load("memory-1"); store.edit("memory-1", { note: "Unsaved" });
    data.set("memory-1", { ...data.get("memory-1")!, title: "Agent title", rev: "2" }); emit();
    await vi.advanceTimersByTimeAsync(100); expect(store.record("memory-1")!.note).toBe("Unsaved");
    disconnect(); await store.flush("memory-1"); expect(store.state("memory-1")).toBe("conflict");
  });
});
