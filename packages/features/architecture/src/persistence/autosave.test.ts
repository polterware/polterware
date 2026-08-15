import { describe, expect, it, vi } from "vitest";
import { createAutosaveController } from "./autosave";

describe("autosave", () => {
  it("debounces and saves", async () => {
    vi.useFakeTimers();
    const save = vi.fn().mockResolvedValue({ revision: 2 });
    const ctrl = createAutosaveController({
      systemId: "sys",
      port: { save },
      docDebounceMs: 800,
    });
    ctrl.schedule(
      () => '{"formatVersion":1}',
      () => 1,
    );
    expect(ctrl.status()).toBe("dirty");
    expect(save).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(800);
    expect(save).toHaveBeenCalledTimes(1);
    expect(ctrl.status()).toBe("saved");
    vi.useRealTimers();
  });

  it("surfaces conflict errors", async () => {
    vi.useFakeTimers();
    const onConflict = vi.fn();
    const save = vi.fn().mockRejectedValue(new Error("layout revision conflict"));
    const ctrl = createAutosaveController({
      systemId: "sys",
      port: { save },
      docDebounceMs: 100,
      onConflict,
    });
    ctrl.schedule(
      () => "{}",
      () => 1,
    );
    await vi.advanceTimersByTimeAsync(100);
    expect(ctrl.status()).toBe("error");
    expect(onConflict).toHaveBeenCalled();
    vi.useRealTimers();
  });
});
