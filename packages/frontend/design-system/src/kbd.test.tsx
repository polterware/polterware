// @vitest-environment jsdom

import { render } from "solid-js/web";
import { describe, expect, it } from "vitest";

import { Kbd } from "./kbd";

describe("Kbd", () => {
  it("renders a keyboard chip", () => {
    const host = document.createElement("div");
    document.body.append(host);
    const dispose = render(() => <Kbd>⌘J</Kbd>, host);

    const chip = host.querySelector("kbd.pw-kbd");
    expect(chip).not.toBeNull();
    expect(chip?.textContent).toBe("⌘J");

    dispose();
    host.remove();
  });
});
