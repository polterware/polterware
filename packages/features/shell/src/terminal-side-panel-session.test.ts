import { describe, expect, it } from "vitest";
import {
  hasLiveTerminalBridge,
  resolveDrawerTarget,
} from "./terminal-side-panel-session";
import type { TerminalTargetItem } from "./types";

describe("terminal-side-panel-session", () => {
  it("requires project and session callbacks for a live bridge", () => {
    expect(hasLiveTerminalBridge({})).toBe(false);
    expect(
      hasLiveTerminalBridge({
        projectId: "proj-1",
        onCreateSession: async () => ({
          id: "term-1",
          name: "node",
          targetId: "node-1",
          nodeName: "node",
          shell: "zsh",
          status: "active",
          startedAt: new Date().toISOString(),
        }),
        onWriteSession: async () => undefined,
        onReadSession: async () => ({ data: "", nextOffset: 0, closed: false }),
      }),
    ).toBe(true);
  });

  it("resolves drawer targets by id", () => {
    const targets: TerminalTargetItem[] = [
      {
        id: "node-1",
        name: "api",
        environmentId: "env-1",
        shell: "zsh",
        status: "online",
      },
    ];
    expect(resolveDrawerTarget(targets, "node-1")?.name).toBe("api");
    expect(resolveDrawerTarget(targets, "missing")).toBeNull();
  });
});