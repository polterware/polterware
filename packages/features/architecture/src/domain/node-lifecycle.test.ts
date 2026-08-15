import { describe, expect, it } from "vitest";
import {
  linkStatusLabel,
  linkStatusTone,
  nodeLifecycleTone,
  nodeStatusLabel,
} from "./node-lifecycle";

describe("nodeStatusLabel", () => {
  it("maps lifecycle states honestly (linked is not healthy)", () => {
    expect(nodeStatusLabel("observed")).toBe("Observed");
    expect(nodeStatusLabel("linked")).toBe("Linked");
    expect(nodeStatusLabel("drifted")).toBe("Drifted");
    expect(nodeStatusLabel("unreachable")).toBe("Unreachable");
    expect(nodeStatusLabel("planned")).toBe("Planned");
    expect(nodeStatusLabel("unknown")).toBe("Unknown");
  });
});

describe("nodeLifecycleTone", () => {
  it("does not paint planned/linked as healthy service health", () => {
    expect(nodeLifecycleTone("observed")).toBe("healthy");
    expect(nodeLifecycleTone("planned")).toBe("muted");
    expect(nodeLifecycleTone("linked")).toBe("ok");
    expect(nodeLifecycleTone("unreachable")).toBe("down");
    expect(nodeLifecycleTone("drifted")).toBe("degraded");
  });
});

describe("linkStatus helpers", () => {
  it("labels and tones link posture", () => {
    expect(linkStatusLabel("linked")).toBe("Linked");
    expect(linkStatusLabel("unlinked")).toBe("Unlinked");
    expect(linkStatusTone("partial")).toBe("warn");
    expect(linkStatusTone("unlinked")).toBe("muted");
  });
});
