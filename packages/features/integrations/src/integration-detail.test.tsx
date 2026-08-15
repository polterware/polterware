import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { CATALOG_PROVIDER_MOCKS } from "@polterware/feature-shell";

describe("visual CLI integrations", () => {
  it("contains exactly the supported providers and commands", () => {
    expect(CATALOG_PROVIDER_MOCKS).toHaveLength(9);
    expect(CATALOG_PROVIDER_MOCKS.every((provider) => provider.binary && provider.authCommands.length > 0)).toBe(true);
  });

  it("uses the shared transcript without starting external work", () => {
    const source = readFileSync(join(import.meta.dirname, "setup-wizard.tsx"), "utf8");
    expect(source).toContain("TerminalTranscript");
    expect(source).toContain("No command, browser, PTY, or authentication request will be started");
    expect(source).not.toContain('type="password"');
  });
});
