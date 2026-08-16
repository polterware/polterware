import type { ArchitectureDocument, SelectionState } from "../document/types";

export type CaptureMode = "immediately" | "eventually" | "never";

export type DocPatch = {
  label?: string;
  /** Full document snapshots for reliable undo (bounded stack). */
  before: ArchitectureDocument;
  after: ArchitectureDocument;
  selectionBefore?: SelectionState;
  selectionAfter?: SelectionState;
};

export type HistoryState = {
  undoStack: DocPatch[];
  redoStack: DocPatch[];
  limit: number;
  /** Pending eventually snapshot before first mutation in gesture. */
  pendingBefore: ArchitectureDocument | null;
  pendingSelection: SelectionState | null;
};

export function createHistory(limit = 100): HistoryState {
  return {
    undoStack: [],
    redoStack: [],
    limit,
    pendingBefore: null,
    pendingSelection: null,
  };
}

function pushUndo(h: HistoryState, patch: DocPatch): HistoryState {
  const undoStack = [...h.undoStack, patch];
  while (undoStack.length > h.limit) undoStack.shift();
  return {
    ...h,
    undoStack,
    redoStack: [],
    pendingBefore: null,
    pendingSelection: null,
  };
}

/**
 * Commit a document change under a capture mode.
 * - never: return doc as-is, no history
 * - eventually: record before on first call; do not push until immediately
 * - immediately: push patch (using pending before if set)
 */
export function commit(
  history: HistoryState,
  beforeDoc: ArchitectureDocument,
  afterDoc: ArchitectureDocument,
  mode: CaptureMode,
  opts?: {
    label?: string;
    selectionBefore?: SelectionState;
    selectionAfter?: SelectionState;
  },
) {
  if (mode === "never") {
    return { history, doc: afterDoc };
  }

  if (mode === "eventually") {
    if (!history.pendingBefore) {
      return {
        history: {
          ...history,
          pendingBefore: structuredClone(beforeDoc),
          pendingSelection: opts?.selectionBefore ?? null,
        },
        doc: afterDoc,
      };
    }
    return { history, doc: afterDoc };
  }

  // immediately
  const before = history.pendingBefore ?? structuredClone(beforeDoc);
  const selectionBefore = history.pendingSelection ?? opts?.selectionBefore;
  const patch: DocPatch = {
    label: opts?.label,
    before,
    after: structuredClone(afterDoc),
    selectionBefore: selectionBefore ?? undefined,
    selectionAfter: opts?.selectionAfter,
  };
  // skip empty
  if (JSON.stringify(patch.before) === JSON.stringify(patch.after)) {
    return {
      history: { ...history, pendingBefore: null, pendingSelection: null },
      doc: afterDoc,
    };
  }
  return { history: pushUndo(history, patch), doc: afterDoc };
}

export function undo(
  history: HistoryState,
  current: ArchitectureDocument,
) {
  if (history.undoStack.length === 0) {
    return { history, doc: current };
  }
  const patch = history.undoStack[history.undoStack.length - 1]!;
  const undoStack = history.undoStack.slice(0, -1);
  const redoStack = [
    ...history.redoStack,
    {
      ...patch,
      before: structuredClone(patch.before),
      after: structuredClone(current),
    },
  ];
  return {
    history: {
      ...history,
      undoStack,
      redoStack,
      pendingBefore: null,
      pendingSelection: null,
    },
    doc: structuredClone(patch.before),
    selection: patch.selectionBefore,
  };
}

export function redo(
  history: HistoryState,
  current: ArchitectureDocument,
) {
  if (history.redoStack.length === 0) {
    return { history, doc: current };
  }
  const patch = history.redoStack[history.redoStack.length - 1]!;
  const redoStack = history.redoStack.slice(0, -1);
  const undoStack = [
    ...history.undoStack,
    {
      ...patch,
      before: structuredClone(current),
      after: structuredClone(patch.after),
    },
  ];
  return {
    history: {
      ...history,
      undoStack,
      redoStack,
      pendingBefore: null,
      pendingSelection: null,
    },
    doc: structuredClone(patch.after),
    selection: patch.selectionAfter,
  };
}

export function canUndo(h: HistoryState): boolean {
  return h.undoStack.length > 0;
}

export function canRedo(h: HistoryState): boolean {
  return h.redoStack.length > 0;
}
