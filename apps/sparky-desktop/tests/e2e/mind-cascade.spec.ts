import { test, expect } from "../fixtures";

test("deleting a parent Mind cascades children and moves their memories to Inbox", async ({ sparky, page }) => {
  await sparky.skipOnboarding();
  const fixture = await page.evaluate(async () => {
    const parent = (await window.sparky.minds.create({ name: "Cascade parent" })).mind!;
    const child = (await window.sparky.minds.create({ name: "Cascade child", parentId: parent.id })).mind!;
    const memory = (await window.sparky.memories.create({ title: "Cascade memory", mindId: child.id })).memory!;
    return { parent, child, memory };
  });
  await page.getByRole("button", { name: "Inbox", exact: true }).click();
  const result = await page.evaluate(async parent => window.sparky.minds.delete({ id: parent.id, expectedRev: parent.rev }), fixture.parent);
  expect(result.deletedMindIds?.sort()).toEqual([fixture.parent.id, fixture.child.id].sort());
  expect(result.movedMemoryCount).toBe(1);
  await expect.poll(() => page.evaluate(async () => ((await window.sparky.minds.list({})).minds ?? []).length)).toBe(0);
  await expect(page.getByText("Live · 1 memories", { exact: true })).toBeVisible();
  const memory = await page.evaluate(async id => (await window.sparky.memories.get({ id })).memory, fixture.memory.id);
  expect(memory?.mindId ?? "").toBe("");
});
