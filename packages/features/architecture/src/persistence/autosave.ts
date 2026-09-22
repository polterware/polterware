export type SaveStatus = "clean" | "dirty" | "saving" | "saved" | "error";

export type LayoutSavePort = {
  save: (
    systemId: string,
    expectedRevision: number,
    bodyJson: string,
  ) => Promise<{ revision: number }>;
};

export type AutosaveFlushOptions = {
  /** Persist even when status is clean (e.g. Cmd+S including viewport-only changes). */
  force?: boolean;
};

export type AutosaveController = {
  status: () => SaveStatus;
  markDirty: () => void;
  schedule: (getBody: () => string, getRevision: () => number) => void;
  flush: (
    getBody: () => string,
    getRevision: () => number,
    opts?: AutosaveFlushOptions,
  ) => Promise<void>;
  setStatusListener: (fn: (s: SaveStatus) => void) => void;
  lastError: () => string | null;
};

export function createAutosaveController(opts: {
  systemId: string;
  port: LayoutSavePort;
  docDebounceMs?: number;
  onRevision?: (rev: number) => void;
  onConflict?: (err: string) => void;
}): AutosaveController {
  const docDebounceMs = opts.docDebounceMs ?? 800;
  let status: SaveStatus = "clean";
  let timer: ReturnType<typeof setTimeout> | null = null;
  let lastError: string | null = null;
  let listener: ((s: SaveStatus) => void) | null = null;

  const setStatus = (s: SaveStatus) => {
    status = s;
    listener?.(s);
  };

  const runSave = async (getBody: () => string, getRevision: () => number) => {
    setStatus("saving");
    lastError = null;
    try {
      const res = await opts.port.save(opts.systemId, getRevision(), getBody());
      opts.onRevision?.(res.revision);
      setStatus("saved");
      // return to clean after brief saved
      setTimeout(() => {
        if (status === "saved") setStatus("clean");
      }, 1200);
    } catch (e) {
      lastError = String(e);
      setStatus("error");
      if (/revision|conflict/i.test(lastError)) {
        opts.onConflict?.(lastError);
      }
    }
  };

  return {
    status: () => status,
    markDirty: () => setStatus("dirty"),
    schedule: (getBody, getRevision) => {
      setStatus("dirty");
      if (timer) clearTimeout(timer);
      timer = setTimeout(() => {
        timer = null;
        void runSave(getBody, getRevision);
      }, docDebounceMs);
    },
    flush: async (getBody, getRevision, opts) => {
      if (timer) {
        clearTimeout(timer);
        timer = null;
      }
      if (
        opts?.force ||
        status === "dirty" ||
        status === "error" ||
        status === "saved"
      ) {
        await runSave(getBody, getRevision);
      }
    },
    setStatusListener: (fn) => {
      listener = fn;
    },
    lastError: () => lastError,
  };
}
