import { describe, expect, it, vi } from "vitest";
import { createShellBridge, type ShellWireBridge } from "@polterware/feature-sparky-shell";
import { loadDeletePreview, toMindNode, wireRev } from "./bridge";

function wire(mindsJson: unknown, memories = { total: 0 }): ShellWireBridge {
  return {
    minds: { list: async () => ({ minds: mindsJson }) },
    memories: { list: async () => memories },
    tags: {},
    reminders: {},
    search: {},
    onChange: () => () => {},
    app: { onDeepLink: () => () => {}, getDeepLink: async () => null },
  } as unknown as ShellWireBridge;
}

describe("minds wire bridge", () => {
  it("parses protobuf JSON and sends expectedRev as a decimal string", async () => {
    const update = vi.fn(async () => ({ mind: { id: "a", name: "Renamed", rev: "2" } }));
    const api = createShellBridge({
      ...wire([{ id: "a", name: "Alpha", parentId: "", color: "#7EA8F0", icon: "brain", sortOrder: 0, rev: "1" }]),
      minds: {
        list: async () => ({ minds: [{ id: "a", name: "Alpha", parentId: "", color: "#7EA8F0", icon: "brain", sortOrder: 0, rev: "1" }] }),
        update,
      },
    } as unknown as ShellWireBridge);
    const listed = await api.minds.list({});
    expect(listed.minds[0]?.rev).toBe(1n);
    await api.minds.update({ id: "a", expectedRev: wireRev(listed.minds[0]?.rev), name: "Renamed" });
    expect(update).toHaveBeenCalledWith(expect.objectContaining({ expectedRev: "1" }));
  });

  it("builds a delete preview with the same shape as the delete response", async () => {
    const api = createShellBridge(wire(
      [
        { id: "a", name: "Alpha", parentId: "", rev: "1" },
        { id: "b", name: "Beta", parentId: "a", rev: "1" },
      ],
      { total: 3 },
    ));
    const listed = await api.minds.list({});
    const preview = await loadDeletePreview(api, listed.minds.map(toMindNode), "a");
    expect(preview).toEqual({ deletedMindIds: ["a", "b"], movedMemoryCount: 3 });
  });
});
