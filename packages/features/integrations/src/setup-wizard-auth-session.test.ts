import { describe, expect, it, vi } from "vitest";
import {
  authTargetId,
  startAuthProbeLoop,
  type AuthIntegrationsBridge,
} from "./setup-wizard-auth-session";

describe("setup-wizard-auth-session", () => {
  it("builds a synthetic auth target id", () => {
    expect(authTargetId("docker")).toBe("auth-docker");
  });

  it("stops the probe loop when connected", async () => {
    const probeProvider = vi
      .fn()
      .mockResolvedValueOnce({ status: "ready", accountLabel: "", message: "ready", binaryPath: "/bin/gh" })
      .mockResolvedValueOnce({
        status: "connected",
        accountLabel: "octocat",
        message: "ok",
        binaryPath: "/bin/gh",
      });

    const integrations: AuthIntegrationsBridge = {
      probeProvider,
      beginAuth: vi.fn(),
    };

    const onConnected = vi.fn();
    const stop = startAuthProbeLoop({
      integrations,
      providerId: "github",
      intervalMs: 10,
      timeoutMs: 5_000,
      handlers: {
        onSnapshot: vi.fn(),
        onConnected,
        onUnavailable: vi.fn(),
        onTimeout: vi.fn(),
      },
    });

    await vi.waitFor(() => {
      expect(onConnected).toHaveBeenCalledTimes(1);
    });
    stop();
    expect(probeProvider.mock.calls.length).toBeGreaterThanOrEqual(2);
  });
});
