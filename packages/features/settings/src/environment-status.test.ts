import { describe, expect, it } from "vitest";
import {
  addEnvironmentToast,
  archiveProjectToast,
  defaultFlagLabel,
  defaultFlagTone,
  defaultProtectionForKind,
  deleteEnvironmentToast,
  environmentKindLabel,
  environmentKindTone,
  protectionLabel,
  protectionTone,
  saveGeneralSettingsToast,
  setDefaultEnvironmentToast,
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

  it("maps default protection by kind", () => {
    expect(defaultProtectionForKind("production")).toBe("strict");
    expect(defaultProtectionForKind("staging")).toBe("standard");
    expect(defaultProtectionForKind("development")).toBe("relaxed");
    expect(defaultProtectionForKind("local")).toBe("relaxed");
  });

  it("maps default flag chips", () => {
    expect(defaultFlagLabel(true)).toBe("Default");
    expect(defaultFlagLabel(false)).toBe("Not default");
    expect(defaultFlagTone(true)).toBe("healthy");
    expect(defaultFlagTone(false)).toBe("muted");
  });

  it("keeps mutation toasts honest about local SQLite persistence", () => {
    expect(addEnvironmentToast("staging")).toMatch(/saved locally/i);
    expect(addEnvironmentToast("staging")).not.toMatch(/fixture|preview session|mock/i);
    expect(setDefaultEnvironmentToast("Production")).toMatch(/default environment/i);
    expect(setDefaultEnvironmentToast("Production")).not.toMatch(/fixture|mock/i);
    expect(saveGeneralSettingsToast()).toBe("Project settings saved.");
    expect(saveGeneralSettingsToast()).not.toMatch(/preview session|mock/i);
    expect(archiveProjectToast("Demo")).toMatch(/archived/i);
    expect(archiveProjectToast("Demo")).not.toMatch(/preview|wiped|deleted from core/i);
    expect(deleteEnvironmentToast("staging")).toMatch(/deleted/i);
  });
});
