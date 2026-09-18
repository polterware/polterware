import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { CATALOG_TEST_PROVIDERS } from "./test-fixtures";

describe("visual CLI integrations", () => {
  it("contains exactly the supported providers and commands", () => {
    expect(CATALOG_TEST_PROVIDERS).toHaveLength(9);
    expect(CATALOG_TEST_PROVIDERS.every((provider) => provider.binary && provider.authCommands.length > 0)).toBe(true);
    expect(CATALOG_TEST_PROVIDERS.map((provider) => provider.name)).toEqual([
      "GitHub", "GitLab", "AWS", "Google Cloud", "Azure", "Vercel", "Supabase", "Terraform", "Docker",
    ]);
  });

  it("uses the shared transcript without starting external work", () => {
    const source = readFileSync(join(import.meta.dirname, "setup-wizard.tsx"), "utf8");
    expect(source).toContain("TerminalTranscript");
    expect(source).toContain("No command, browser, PTY, or authentication request will be started");
    expect(source).not.toContain('type="password"');
  });
});
