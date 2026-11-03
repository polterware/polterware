// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import { TYPE_ICON_KEYS } from "./type-icon-keys";
import { TypeIcon } from "./type-icons";

describe("TypeIcon", () => {
  it("renders every persisted type icon key", () => {
    for (const key of TYPE_ICON_KEYS) {
      const host = document.createElement("div");
      const dispose = render(() => <TypeIcon name={key} />, host);

      expect(host.querySelector("svg"), key).not.toBeNull();
      dispose();
    }
  });

  it("renders the box fallback for an unknown key", () => {
    const host = document.createElement("div");
    const dispose = render(() => <TypeIcon name="unknown-icon-key" />, host);

    expect(host.querySelector("svg")).not.toBeNull();
    dispose();
  });
});
