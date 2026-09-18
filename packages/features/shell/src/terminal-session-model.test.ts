import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { TerminalTargetItem } from "./types";
import { UNUSABLE_TERMINAL_TEST_TARGETS } from "./test-fixtures";
import {
  buildSessionCorrelationId,
  canOpenTerminalSessionOnTarget,
  closeMockTerminalSession,
  createMockTerminalSession,
  findSessionForTarget,
  formatTerminalStartedAt,
  removeTerminalSession,
  terminalSessionDataSourceLabel,
  terminalSessionStatusLabel,
  terminalSessionStatusTone,
  unusableTerminalTargetFeedback,
} from "./terminal-session-model";
import type { TerminalSessionMock } from "./workspace-mocks";

const here = dirname(fileURLToPath(import.meta.url));

const onlineTarget: TerminalTargetItem = {
  id: "node-a",
  name: "alpha",
  shell: "bash",
  status: "online",
};

const offlineTarget: TerminalTargetItem = {
  id: "node-b",
  name: "bravo",
  shell: "zsh",
  status: "offline",
};

describe("terminalSessionStatusTone / Label", () => {
  it("maps active/idle/closed to StatusChip tones and labels", () => {
    expect(terminalSessionStatusTone("active")).toBe("healthy");
    expect(terminalSessionStatusTone("idle")).toBe("warn");
    expect(terminalSessionStatusTone("closed")).toBe("muted");
    expect(terminalSessionStatusLabel("active")).toBe("Active");
    expect(terminalSessionStatusLabel("idle")).toBe("Idle");
    expect(terminalSessionStatusLabel("closed")).toBe("Closed");
  });
});

describe("canOpenTerminalSessionOnTarget", () => {
  it("allows online and degraded only", () => {
    expect(canOpenTerminalSessionOnTarget("online")).toBe(true);
    expect(canOpenTerminalSessionOnTarget("degraded")).toBe(true);
    expect(canOpenTerminalSessionOnTarget("offline")).toBe(false);
    expect(canOpenTerminalSessionOnTarget("revoked")).toBe(false);
    expect(UNUSABLE_TERMINAL_TEST_TARGETS.map((target) => target.name)).toEqual(["offline-edge-01", "revoked-node-01"]);
  });
});

describe("unusableTerminalTargetFeedback", () => {
  it("names target and reason without silent wording", () => {
    const msg = unusableTerminalTargetFeedback(offlineTarget);
    expect(msg).toContain("bravo");
    expect(msg.toLowerCase()).toContain("offline");
    expect(msg.toLowerCase()).toContain("cannot open");
  });
});

describe("createMockTerminalSession", () => {
  it("builds active session with shell, startedAt, correlation, mock honesty", () => {
    const session = createMockTerminalSession({
      target: onlineTarget,
      id: "term-node-a-test",
      startedAt: "2026-08-12T12:00:00.000Z",
      dataSource: "mock",
    });
    expect(session.status).toBe("active");
    expect(session.shell).toBe("bash");
    expect(session.nodeId).toBe("node-a");
    expect(session.startedAt).toBe("2026-08-12T12:00:00.000Z");
    expect(session.correlationId).toContain("corr-term");
    expect(session.dataSource).toBe("mock");
    expect(session.lines.join("\n").toLowerCase()).not.toContain("connected to real host");
    expect(session.lines.some((l) => l.includes(session.correlationId))).toBe(true);
  });
});

describe("closeMockTerminalSession / removeTerminalSession", () => {
  it("marks closed and removes from list with next active", () => {
    const a = createMockTerminalSession({ target: onlineTarget, id: "term-a" });
    const closed = closeMockTerminalSession(a);
    expect(closed.status).toBe("closed");

    const b: TerminalSessionMock = {
      ...createMockTerminalSession({
        target: { ...onlineTarget, id: "node-c", name: "charlie" },
        id: "term-c",
      }),
      status: "idle",
    };
    const list = [a, b];
    const result = removeTerminalSession(list, "term-a");
    expect(result.sessions).toHaveLength(1);
    expect(result.nextActiveId).toBe("term-c");
  });
});

describe("findSessionForTarget / formatTerminalStartedAt / dataSource", () => {
  it("finds non-closed session for target", () => {
    const open = createMockTerminalSession({ target: onlineTarget, id: "term-open" });
    const closed = closeMockTerminalSession(
      createMockTerminalSession({ target: onlineTarget, id: "term-old" }),
    );
    expect(findSessionForTarget([closed, open], "node-a")?.id).toBe("term-open");
  });

  it("formats ISO startedAt compactly", () => {
    expect(formatTerminalStartedAt("2026-08-12T12:30:00.000Z")).toMatch(/2026-08-12/);
    expect(formatTerminalStartedAt("")).toBe("—");
  });

  it("labels fixture and mock data sources honestly", () => {
    expect(terminalSessionDataSourceLabel("fixture")).toBe("Fixture session");
    expect(terminalSessionDataSourceLabel("mock")).toBe("Mock session");
    expect(terminalSessionDataSourceLabel(undefined)).toBe("Fixture session");
  });

  it("builds stable correlation ids", () => {
    expect(buildSessionCorrelationId("term-local", "node-local")).toContain("node-local");
  });
});

describe("UI contract source", () => {
  it("terminal page uses StatusChip session chrome", () => {
    const page = readFileSync(join(here, "terminal-page.tsx"), "utf8");
    expect(page).toContain("StatusChip");
    expect(page).toContain("terminalSessionStatusTone");
    expect(page).toContain("TerminalEmulator");
    expect(page).toContain("initialSession");
    expect(page).toContain("unusableTerminalTargetFeedback");
    expect(page).toContain("No terminal sessions");
    expect(page).toContain("No targets available");
    expect(page.toLowerCase()).not.toContain("connected to real host");
  });

  it("target picker uses StatusChip and blocks unusable without silent select", () => {
    const picker = readFileSync(join(here, "target-picker.tsx"), "utf8");
    expect(picker).toContain("StatusChip");
    expect(picker).toContain("terminalTargetStatusTone");
    expect(picker).toContain("isTerminalTargetUsable");
    expect(picker).toContain("onUnusableSelect");
    expect(picker).toContain("No targets available");
  });

  it("side panel hosts live TerminalEmulator without fixture transcript", () => {
    const panel = readFileSync(join(here, "terminal-side-panel.tsx"), "utf8");
    expect(panel).toContain("StatusChip");
    expect(panel).toContain("TerminalEmulator");
    expect(panel).toContain("closeOnInteractOutside={false}");
    expect(panel).toContain("flush");
    expect(panel).not.toContain("TERMINAL_SESSION_MOCKS");
    expect(panel).not.toContain("TERMINAL_OUTPUT_FIXTURES");
    expect(panel.toLowerCase()).not.toMatch(/connected to real host/);
    // Old dishonest "Connected" for usable targets must not return
    expect(panel).not.toMatch(/return "Connected"/);
  });
});