/** Debounced per-id document/path persist for Knowledge editors. */

export type DocumentAutosave = {
  schedule: (id: string, value: string) => void;
  flush: (id?: string) => void;
  dispose: () => void;
};

export function createDocumentAutosave(opts: {
  save: (id: string, value: string) => void | Promise<void>;
  debounceMs?: number;
}): DocumentAutosave {
  const debounceMs = opts.debounceMs ?? 800;
  const timers = new Map<string, ReturnType<typeof setTimeout>>();
  const pending = new Map<string, string>();

  function run(id: string) {
    timers.delete(id);
    const value = pending.get(id);
    pending.delete(id);
    if (value === undefined) return;
    void opts.save(id, value);
  }

  return {
    schedule(id, value) {
      pending.set(id, value);
      const existing = timers.get(id);
      if (existing) clearTimeout(existing);
      timers.set(
        id,
        setTimeout(() => run(id), debounceMs),
      );
    },
    flush(id) {
      if (id !== undefined) {
        const existing = timers.get(id);
        if (existing) clearTimeout(existing);
        run(id);
        return;
      }
      for (const key of [...pending.keys()]) {
        const existing = timers.get(key);
        if (existing) clearTimeout(existing);
        run(key);
      }
    },
    dispose() {
      for (const timer of timers.values()) clearTimeout(timer);
      timers.clear();
      pending.clear();
    },
  };
}
