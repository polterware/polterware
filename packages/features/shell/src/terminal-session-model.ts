import type { StatusTone } from "@polterware/frontend-design-system";
import type { TerminalTargetItem } from "./types";
import { isTerminalTargetUsable, terminalTargetStatusLabel } from "./types";
import type { TerminalSessionMock, TerminalSessionStatus } from "./workspace-mocks";

export type { TerminalSessionStatus };

export type TerminalSessionDataSource = "fixture" | "mock";

/** StatusChip tone for session lifecycle (active / idle / closed). */
export function terminalSessionStatusTone(status: string): StatusTone {
  switch (status.toLowerCase()) {
    case "active":
      return "healthy";
    case "idle":
      return "warn";
    case "closed":
      return "muted";
    default:
      return "unknown";
  }
}

export function terminalSessionStatusLabel(status: string): string {
  switch (status.toLowerCase()) {
    case "active":
      return "Active";
    case "idle":
      return "Idle";
    case "closed":
      return "Closed";
    default:
      return status;
  }
}

/** Operator-facing fixture/mock honesty label — never “connected to real host”. */
export function terminalSessionDataSourceLabel(
  dataSource: TerminalSessionDataSource | string | undefined,
): string {
  const src = (dataSource ?? "fixture").toLowerCase();
  if (src === "mock") return "Mock session";
  return "Fixture session";
}

/** Explicit feedback when a target cannot open a session (never silent). */
export function unusableTerminalTargetFeedback(target: {
  name: string;
  status: string;
}): string {
  const reason = terminalTargetStatusLabel(target.status);
  return `Cannot open session on ${target.name}: target is ${reason.toLowerCase()}.`;
}

export function canOpenTerminalSessionOnTarget(status: string): boolean {
  return isTerminalTargetUsable(status);
}

export function buildSessionCorrelationId(sessionId: string, targetId: string): string {
  return `corr-term-${targetId}-${sessionId.replace(/^term-/, "").slice(0, 12)}`;
}

export type CreateMockTerminalSessionInput = {
  target: TerminalTargetItem;
  /** Optional fixed id for tests; otherwise generated. */
  id?: string;
  promptName?: string;
  startedAt?: string;
  dataSource?: TerminalSessionDataSource;
};

/** Build a new mock session for a usable target. Caller MUST gate usability first. */
export function createMockTerminalSession(
  input: CreateMockTerminalSessionInput,
): TerminalSessionMock {
  const { target } = input;
  const id = input.id ?? `term-${target.id}-${Date.now()}`;
  const startedAt = input.startedAt ?? new Date().toISOString();
  const dataSource = input.dataSource ?? "mock";
  const promptName = input.promptName ?? "polter";
  const correlationId = buildSessionCorrelationId(id, target.id);

  return {
    id,
    name: target.name,
    nodeId: target.id,
    nodeName: target.name,
    cwd: "~",
    shell: target.shell,
    status: "active",
    startedAt,
    duration: "0m",
    environment: "Target",
    correlationId,
    dataSource,
    lines: [
      `${terminalSessionDataSourceLabel(dataSource)} · ${target.name} · ${target.shell}`,
      `session ${id}`,
      `correlation ${correlationId}`,
      `${promptName}@${target.name}:~$ _`,
    ],
  };
}

/** Mark a session closed without removing it from a list (optional soft-close). */
export function closeMockTerminalSession(session: TerminalSessionMock): TerminalSessionMock {
  return {
    ...session,
    status: "closed",
    duration: session.duration === "—" ? "—" : session.duration,
    lines: [...session.lines, "", `[${terminalSessionDataSourceLabel(session.dataSource)}] session closed.`],
  };
}

/** Remove session id from list; return next active id preference. */
export function removeTerminalSession(
  sessions: TerminalSessionMock[],
  id: string,
) {
  const remaining = sessions.filter((s) => s.id !== id);
  const nextActiveId = remaining.find((s) => s.status !== "closed")?.id ?? remaining[0]?.id ?? "";
  return { sessions: remaining, nextActiveId };
}

export function formatTerminalStartedAt(startedAt: string): string {
  if (!startedAt) return "—";
  // Keep ISO compact if already human; avoid locale surprises in tests.
  if (/^\d{4}-\d{2}-\d{2}T/.test(startedAt)) {
    const d = new Date(startedAt);
    if (!Number.isNaN(d.getTime())) {
      const pad = (n: number) => String(n).padStart(2, "0");
      return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
    }
  }
  return startedAt;
}

export function findSessionForTarget(
  sessions: TerminalSessionMock[],
  targetId: string,
): TerminalSessionMock | undefined {
  return sessions.find((s) => s.nodeId === targetId && s.status !== "closed");
}
