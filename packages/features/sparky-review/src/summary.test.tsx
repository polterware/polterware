import { afterEach, describe, expect, it } from "vitest";
import { createSignal } from "solid-js";
import { render } from "solid-js/web";
import { DesignSystemProvider } from "@polterware/frontend-design-system";
import { SummaryCard } from "./summary";
import { saoPauloFixture } from "./fake-bridge";

let dispose: (() => void) | undefined;
afterEach(() => { dispose?.(); document.body.replaceChildren(); });

describe("served character art", () => {
  it("uses /characters assets and preserves a real image failure fallback", () => {
    const [report, setReport] = createSignal(saoPauloFixture());
    const host = document.createElement("div"); document.body.append(host);
    dispose = render(() => <DesignSystemProvider><SummaryCard report={report()} /></DesignSystemProvider>, host);
    const image = host.querySelector<HTMLImageElement>("img")!;
    expect(image.getAttribute("src")).toBe("/characters/me-streak-hopeful.png");
    expect(image.getAttribute("srcset")).toContain("/characters/me-streak-hopeful@2x.png");
    image.dispatchEvent(new Event("load"));
    expect(host.textContent).not.toContain("Character art unavailable");
    image.dispatchEvent(new Event("error"));
    expect(host.textContent).toContain("Character art unavailable");
    setReport(current => ({ ...current, streakMood: "happy" }));
    expect(host.querySelector<HTMLImageElement>("img")?.getAttribute("src")).toBe("/characters/me-streak-happy.png");
    expect(host.textContent).not.toContain("Character art unavailable");
  });
});
