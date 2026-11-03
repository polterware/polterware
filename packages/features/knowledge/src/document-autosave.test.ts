import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createDocumentAutosave } from "./document-autosave";

describe("createDocumentAutosave", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("does not save after cancel", () => {
    const save = vi.fn();
    const autosave = createDocumentAutosave({ save, debounceMs: 100 });

    autosave.schedule("doc-1", "changed");
    autosave.cancel("doc-1");
    vi.advanceTimersByTime(200);

    expect(save).not.toHaveBeenCalled();
    autosave.dispose();
  });

  it("saves after debounce when not cancelled", () => {
    const save = vi.fn();
    const autosave = createDocumentAutosave({ save, debounceMs: 100 });

    autosave.schedule("doc-1", "changed");
    vi.advanceTimersByTime(100);

    expect(save).toHaveBeenCalledWith("doc-1", "changed");
    autosave.dispose();
  });
});
