import { describe, expect, it, vi } from "vitest";
import { deepLinkPath, findCommands, matchPath, pushRoute, resolveRoute, sidebarItems } from "./route";
import type { SparkyFeature } from "./feature";
const component = () => null;
describe("shell registry and routes", () => {
  it("uses registered routes before shell stubs", () => {
    const stub = { path: "/today", title: "Stub", component };
    const live = { ...stub, title: "Live" };
    expect(resolveRoute([{ id: "shell", routes: [stub] }, { id: "memories", routes: [live] }], "/today")?.route).toBe(live);
  });
  it.each([
    ["/mind/:id", "/mind/Home%20Office", { id: "Home Office" }],
    ["/calendar", "/calendar/?date=2026-10-04", {}],
    ["/mind/:id", "/mind", null], ["/mind/:id", "/mind/%", null], ["/today", "/today/extra", null],
  ])("matches %s to %s", (pattern, path, expected) => expect(matchPath(pattern, path)).toEqual(expected));
  it("preserves the six primary navigation destinations and contributions", () => {
    const feature: SparkyFeature = { id: "minds", routes: [], sidebar: [{ id: "minds", label: "My Minds", path: "/minds", badge: 3 }, { id: "custom", label: "Custom", path: "/custom" }] };
    const items = sidebarItems([feature]);
    expect(items.find(item => item.path === "/minds")?.label).toBe("My Minds");
    expect(items.filter(item => item.path === "/minds")).toHaveLength(1);
    expect(items.at(-1)?.path).toBe("/custom");
  });
  it("finds a registered command by keywords and executes its callback", async () => {
    const run = vi.fn();
    const commands = [{ id: "pin", label: "Pin selected memory", keywords: ["favorite"], run }];
    await findCommands(commands, "favorite memory")[0]?.run();
    expect(run).toHaveBeenCalledOnce();
    expect(findCommands(commands, "missing")).toHaveLength(0);
  });
  it("branches history after going back and ignores repeated routes", () => {
    const history = { entries: ["/today", "/inbox", "/calendar"], index: 1 };
    expect(pushRoute(history, "/map")).toEqual({ entries: ["/today", "/inbox", "/map"], index: 2 });
    expect(pushRoute(history, "/inbox")).toBe(history);
  });
  it.each([
    ["sparky://today", "/today"], ["sparky://memory/ABC?action=focus", "/memory/ABC"],
    ["sparky://calendar?date=2026-10-04", "/calendar"], ["https://example.com/today", null], ["sparky://unknown", null], ["bad url", null],
  ])("routes deep link %s", (url, expected) => expect(deepLinkPath(url)).toBe(expected));
});

it("records query changes without mutating the current history", () => {
  const history = { entries: ["/calendar"], index: 0 };
  expect(pushRoute(history, "/calendar?date=2026-10-04")).toEqual({ entries: ["/calendar", "/calendar?date=2026-10-04"], index: 1 });
  expect(history.entries).toEqual(["/calendar"]);
});
