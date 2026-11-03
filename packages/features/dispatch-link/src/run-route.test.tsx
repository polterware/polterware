import { createRoot } from "solid-js";
import { describe, expect, it } from "vitest";
import { createRunRoute } from "./run-route";

const CARDS = [
  { slug: "board-view", title: "Board view", milestone: "Beta", stage: "In progress" },
  { slug: "split-api", title: "Split the API", milestone: "Backlog", stage: "Planned" },
];

function route(autoAvailable = true) {
  return createRoot(() => createRunRoute({
    autoAvailable: () => autoAvailable,
    tiers: () => [],
    agents: () => [],
    models: () => [{ agent: "codex", model: "gpt-5-codex-mini", provider: "openai", tier: "easy" }] as never,
    cards: () => CARDS,
  }));
}
const pill = (r: ReturnType<typeof route>, id: string) => r.properties().find((property) => property.id === id)!;

describe("createRunRoute", () => {
  it("starts on Auto with no card, listed first", () => {
    const r = route();
    expect(r.selection()).toEqual({ tier: "auto", agent: "", model: "", cardSlug: "" });
    expect(pill(r, "route").options[0]).toMatchObject({ value: "tier:auto", label: "Auto", hint: "The classifier judges the task, then the cheapest model likely to pass" });
    expect(pill(r, "route").options.slice(1, 4).map((option) => option.value)).toEqual(["tier:easy", "tier:normal", "tier:hard"]);
  });

  it("routes by one model, a tier, or back to Auto", () => {
    const r = route();
    pill(r, "route").onChange("codex\u0000gpt-5-codex-mini");
    expect(r.selection()).toMatchObject({ tier: "", agent: "codex", model: "gpt-5-codex-mini" });
    pill(r, "route").onChange("tier:hard");
    expect(r.selection()).toMatchObject({ tier: "hard", agent: "", model: "" });
    pill(r, "route").onChange("codex\u0000gpt-5-codex-mini");
    expect(r.selection()).toMatchObject({ tier: "hard", agent: "codex" });
    pill(r, "route").onChange("tier:auto");
    expect(r.selection()).toMatchObject({ tier: "auto", agent: "", model: "" });
  });

  it("offers no Auto without a classifier and starts on Normal", () => {
    const r = route(false);
    expect(r.selection()).toMatchObject({ tier: "normal" });
    expect(pill(r, "route").value).toBe("tier:normal");
    expect(pill(r, "route").options.map((option) => option.value)).not.toContain("tier:auto");
    pill(r, "route").onChange("codex\u0000gpt-5-codex-mini");
    expect(r.selection()).toMatchObject({ tier: "", agent: "codex" });
  });

  it("shows Card under a milestone and keeps them consistent", () => {
    const r = route();
    expect(pill(r, "card").hidden).toBe(true);
    r.setCard("board-view");
    expect(pill(r, "milestone").value).toBe("Beta");
    expect(pill(r, "card").hidden).toBe(false);
    expect(pill(r, "card").options.map((option) => option.value)).toEqual(["", "board-view"]);
    pill(r, "milestone").onChange("Backlog");
    expect(r.selection().cardSlug).toBe("");
    r.clearCard();
    expect(pill(r, "milestone").value).toBe("");
  });
});
