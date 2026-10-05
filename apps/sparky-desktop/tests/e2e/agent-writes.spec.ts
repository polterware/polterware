import { test, expect } from "../fixtures";

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
