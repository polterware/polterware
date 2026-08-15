import { describe, expect, it } from "vitest";
import {
  addEnvironmentToast,
  archiveProjectToast,
  defaultFlagLabel,
  defaultFlagTone,
  demoteEnvironmentToast,
  environmentKindLabel,
  environmentKindTone,
  promoteEnvironmentToast,
  protectionLabel,
  protectionTone,
  saveGeneralSettingsToast,
} from "./environment-status";

describe("environment status helpers", () => {
  it("maps kind labels and tones for StatusChip", () => {
    expect(environmentKindLabel("production")).toBe("Production");
    expect(environmentKindLabel("staging")).toBe("Staging");
    expect(environmentKindLabel("development")).toBe("Development");
    expect(environmentKindLabel("local")).toBe("Local");
    expect(environmentKindTone("production")).toBe("danger");
    expect(environmentKindTone("staging")).toBe("warn");
    expect(environmentKindTone("development")).toBe("healthy");
    expect(environmentKindTone("local")).toBe("muted");
  });

  it("maps protection labels and tones for StatusChip", () => {
    expect(protectionLabel("strict")).toBe("Strict");
    expect(protectionLabel("standard")).toBe("Standard");
    expect(protectionLabel("relaxed")).toBe("Relaxed");
    expect(protectionTone("strict")).toBe("danger");
    expect(protectionTone("standard")).toBe("warn");
    expect(protectionTone("relaxed")).toBe("muted");
  });

  it("maps default flag chips", () => {
    expect(defaultFlagLabel(true)).toBe("Default");
    expect(defaultFlagLabel(false)).toBe("Not default");
    expect(defaultFlagTone(true)).toBe("healthy");
    expect(defaultFlagTone(false)).toBe("muted");
  });

  it("keeps mutation toasts honest about mock stage", () => {
    expect(promoteEnvironmentToast("Production")).toMatch(/fixture|mock/i);
    expect(demoteEnvironmentToast("Staging")).toMatch(/fixture|mock/i);
    expect(addEnvironmentToast()).toMatch(/mock/i);
    expect(saveGeneralSettingsToast()).toMatch(/mock/i);
    expect(archiveProjectToast()).toMatch(/mock/i);
    expect(archiveProjectToast()).not.toMatch(/wiped|deleted from core/i);
  });
});
