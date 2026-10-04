import { test, expect } from "../fixtures";

// Product gap: plans/sparky-next/requests/w3-e2e-3.md
test.fixme(process.env.SPARKY_E2E_RECHECK_GAPS !== "1", "w3-e2e-3: SDK writes succeed but virtualized rows never render. Set SPARKY_E2E_RECHECK_GAPS=1 after the fix.");
import { driver } from "../helpers/driver";

for (const transport of ["stdio", "http"] as const) {
  test(`agent ${transport} SDK write invalidates the open window`, async ({ sparky, page }) => {
    await sparky.skipOnboarding();
    await page.getByRole("button", { name: "Inbox", exact: true }).click();
    await page.evaluate(() => window.sparky.settings.set({ keys: ["agents.mcpEnabled"], settings: { agentsMcpEnabled: true } }));
    const title = `Agent ${transport} memory`;
    const result = await driver(sparky, transport, title);
    expect(result.isError ?? false).toBe(false);
    await expect(page.getByRole("main")).toContainText(title);
    const persisted = await page.evaluate(async query => (await window.sparky.memories.list({ query })).memories, title);
    expect(persisted).toHaveLength(1);
  });
}
