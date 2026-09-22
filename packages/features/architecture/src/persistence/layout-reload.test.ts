import { describe, expect, it } from "vitest";
import { shouldApplyExternalLayout } from "./layout-reload";

describe("shouldApplyExternalLayout", () => {
  it("applies on first load", () => {
    expect(
      shouldApplyExternalLayout({
        sourceKey: "sys::fallback",
        previousSourceKey: "",
        remoteRevision: 1,
        localRevision: 0,
        lastLocalSaveRevision: null,
      }),
    ).toBe(true);
  });

  it("applies when system or fixture mode changes", () => {
    expect(
      shouldApplyExternalLayout({
        sourceKey: "other::fallback",
        previousSourceKey: "sys::fallback",
        remoteRevision: 3,
        localRevision: 3,
        lastLocalSaveRevision: 3,
      }),
    ).toBe(true);
  });

  it("ignores echo of a local autosave revision bump", () => {
    expect(
      shouldApplyExternalLayout({
        sourceKey: "sys::fallback",
        previousSourceKey: "sys::fallback",
        remoteRevision: 4,
        localRevision: 4,
        lastLocalSaveRevision: 4,
      }),
    ).toBe(false);
  });

  it("suppresses reload while a local save echo is in flight", () => {
    expect(
      shouldApplyExternalLayout({
        sourceKey: "sys::fallback",
        previousSourceKey: "sys::fallback",
        remoteRevision: 4,
        localRevision: 3,
        lastLocalSaveRevision: null,
        suppressLayoutEcho: true,
      }),
    ).toBe(false);
  });

  it("ignores when remote revision already matches local", () => {
    expect(
      shouldApplyExternalLayout({
        sourceKey: "sys::fallback",
        previousSourceKey: "sys::fallback",
        remoteRevision: 2,
        localRevision: 2,
        lastLocalSaveRevision: null,
      }),
    ).toBe(false);
  });

  it("applies external refresh with a newer remote revision", () => {
    expect(
      shouldApplyExternalLayout({
        sourceKey: "sys::fallback",
        previousSourceKey: "sys::fallback",
        remoteRevision: 7,
        localRevision: 5,
        lastLocalSaveRevision: 5,
      }),
    ).toBe(true);
  });
});
