// @vitest-environment jsdom

import { render } from "solid-js/web";
import { describe, expect, it, vi } from "vitest";

import { TemplateCard, TemplateCardGrid } from "./template-card";

describe("TemplateCard", () => {
  it("renders title, description, and invokes onSelect", () => {
    const onSelect = vi.fn();
    const host = document.createElement("div");
    document.body.append(host);
    const dispose = render(
      () => (
        <TemplateCard
          title="Architecture overview"
          description="System context and boundaries."
          icon={<span data-testid="icon">◆</span>}
          onSelect={onSelect}
        />
      ),
      host,
    );

    const button = host.querySelector("button.pw-template-card");
    expect(button).not.toBeNull();
    expect(button?.textContent).toContain("Architecture overview");
    expect(button?.textContent).toContain("System context and boundaries.");
    button?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(onSelect).toHaveBeenCalledTimes(1);

    dispose();
    host.remove();
  });
});

describe("TemplateCardGrid", () => {
  it("renders a labeled grid of cards", () => {
    const host = document.createElement("div");
    document.body.append(host);
    const dispose = render(
      () => (
        <TemplateCardGrid
          label="Start from a template"
          items={[
            {
              id: "a",
              title: "A",
              description: "First",
              icon: <span>A</span>,
              onSelect: () => undefined,
            },
            {
              id: "b",
              title: "B",
              description: "Second",
              icon: <span>B</span>,
              onSelect: () => undefined,
            },
          ]}
        />
      ),
      host,
    );

    expect(host.querySelector(".pw-template-card-grid__label")?.textContent).toBe(
      "Start from a template",
    );
    expect(host.querySelectorAll("button.pw-template-card")).toHaveLength(2);

    dispose();
    host.remove();
  });
});
