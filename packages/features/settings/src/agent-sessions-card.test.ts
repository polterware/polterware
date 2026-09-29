import { describe, expect, it } from "vitest";
import { mapAgentSessions, sessionAge } from "./agent-sessions-card";

describe("agent sessions", () => {
  it("maps sessions, claims, and handoffs from a core response", () => {
    const view = mapAgentSessions({
      branch: "feat/x",
      sessions: [
        { runId: "r1", agent: "codex", task: "wire import", startedAt: "2026-09-29T10:00:00Z", claims: [{ slug: "git-import", number: 2, scope: "import/*" }] },
      ],
      handoffs: [{ slug: "git-import", branch: "feat/x", goal: "import HEAD", agent: "codex", at: "2026-09-29T11:00:00Z" }],
    });
    expect(view.branch).toBe("feat/x");
    expect(view.sessions[0]).toMatchObject({ agent: "codex", claims: [{ slug: "git-import", number: 2, scope: "import/*" }] });
    expect(view.handoffs[0]?.goal).toBe("import HEAD");
  });

  it("returns an empty view for an empty response", () => {
    expect(mapAgentSessions({})).toEqual({ branch: "", sessions: [], handoffs: [] });
  });

  it("formats compact ages", () => {
    const now = Date.parse("2026-09-29T12:00:00Z");
    expect(sessionAge("2026-09-29T11:46:00Z", now)).toBe("14m");
    expect(sessionAge("2026-09-29T09:00:00Z", now)).toBe("3h");
    expect(sessionAge("", now)).toBe("");
  });
});
