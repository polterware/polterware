import { describe, expect, it } from "vitest";
import { mapAgentSetup, setupChecks } from "./agent-setup-card";

describe("agent setup", () => {
  it("maps the setup response and treats an omitted enabled as off", () => {
    const setup = mapAgentSetup({
      setup: {
        hooks: [
          { name: "status", label: "Session-start status", enabled: true },
          { name: "stop", label: "Stop handoff reminder" },
        ],
        polterwarePath: "/usr/local/bin/polterware",
        claudePath: "/usr/local/bin/claude",
        pluginInstalled: true,
        pluginEnabled: true,
        pluginVersion: "0.1.0",
        embeddedVersion: "0.1.0",
      },
    });
    expect(setup.hooks.map((hook) => hook.enabled)).toEqual([true, false]);
    expect(setupChecks(setup).every((check) => check.tone === "ok" || check.label === "Codex skill")).toBe(true);
  });

  it("flags a plugin behind the binary and a missing claude", () => {
    const behind = mapAgentSetup({
      setup: { polterwarePath: "/p", claudePath: "/c", pluginInstalled: true, pluginVersion: "0.0.9", embeddedVersion: "0.1.0" },
    });
    expect(setupChecks(behind).find((check) => check.label === "Claude plugin")).toMatchObject({
      tone: "warn",
      hint: "polterware setup claude",
    });
    const none = mapAgentSetup({ setup: {} });
    expect(setupChecks(none).find((check) => check.label === "Claude Code")?.state).toBe("Not detected");
  });
});
