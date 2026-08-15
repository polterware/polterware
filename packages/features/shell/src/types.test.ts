import { describe, expect, it } from "vitest";
import {
  isTerminalTargetUsable,
  terminalTargetStatusLabel,
  terminalTargetStatusTone,
  type TerminalTargetStatus,
} from "./types";

const ALL_STATUSES: TerminalTargetStatus[] = [
  "online",
  "offline",
  "degraded",
  "registering",
  "draining",
  "upgrading",
  "revoked",
  "unknown",
];

describe("isTerminalTargetUsable", () => {
  it("allows online and degraded", () => {
    expect(isTerminalTargetUsable("online")).toBe(true);
    expect(isTerminalTargetUsable("degraded")).toBe(true);
    expect(isTerminalTargetUsable("ONLINE")).toBe(true);
  });

  it("rejects lifecycle and offline statuses", () => {
    for (const status of [
      "offline",
      "revoked",
      "draining",
      "upgrading",
      "registering",
      "unknown",
    ] as const) {
      expect(isTerminalTargetUsable(status)).toBe(false);
    }
  });
});

describe("terminalTargetStatusTone", () => {
  it("maps health statuses to StatusDot tones", () => {
    expect(terminalTargetStatusTone("online")).toBe("healthy");
    expect(terminalTargetStatusTone("degraded")).toBe("warn");
    expect(terminalTargetStatusTone("registering")).toBe("warn");
    expect(terminalTargetStatusTone("draining")).toBe("warn");
    expect(terminalTargetStatusTone("upgrading")).toBe("warn");
    expect(terminalTargetStatusTone("offline")).toBe("down");
    expect(terminalTargetStatusTone("revoked")).toBe("down");
    expect(terminalTargetStatusTone("unknown")).toBe("unknown");
  });
});

describe("terminalTargetStatusLabel", () => {
  it("title-cases known statuses", () => {
    expect(terminalTargetStatusLabel("online")).toBe("Online");
    expect(terminalTargetStatusLabel("offline")).toBe("Offline");
    expect(terminalTargetStatusLabel("degraded")).toBe("Degraded");
    expect(terminalTargetStatusLabel("registering")).toBe("Registering");
    expect(terminalTargetStatusLabel("draining")).toBe("Draining");
    expect(terminalTargetStatusLabel("upgrading")).toBe("Upgrading");
    expect(terminalTargetStatusLabel("revoked")).toBe("Revoked");
    expect(terminalTargetStatusLabel("unknown")).toBe("Unknown");
  });

  it("returns a non-empty label for every TerminalTargetStatus", () => {
    for (const status of ALL_STATUSES) {
      expect(terminalTargetStatusLabel(status).length).toBeGreaterThan(0);
    }
  });
});

describe("target usability UI contract", () => {
  it("every TerminalTargetStatus has tone + label + usability decision", () => {
    for (const status of ALL_STATUSES) {
      expect(terminalTargetStatusTone(status)).toBeTruthy();
      expect(terminalTargetStatusLabel(status).length).toBeGreaterThan(0);
      const usable = isTerminalTargetUsable(status);
      if (status === "online" || status === "degraded") {
        expect(usable).toBe(true);
      } else {
        expect(usable).toBe(false);
      }
    }
  });
});
