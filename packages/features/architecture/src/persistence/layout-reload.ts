/**
 * Decide whether ArchitectureView should replace the in-memory document from
 * workspace layout props. Separates project/fixture switches and external
 * refreshes from the echo of a local autosave (revision bump we just wrote).
 */
export function shouldApplyExternalLayout(input: {
  sourceKey: string;
  previousSourceKey: string;
  remoteRevision: number;
  localRevision: number;
  lastLocalSaveRevision: number | null;
  /** True from just before persist until the workspace echo is observed. */
  suppressLayoutEcho?: boolean;
}): boolean {
  if (input.previousSourceKey === "" || input.sourceKey !== input.previousSourceKey) {
    return true;
  }
  // Suppress while a local save is in flight / echoing — App updates workspace
  // before onRevision runs, so remote can briefly lead local.
  if (input.suppressLayoutEcho) {
    return false;
  }
  if (
    input.lastLocalSaveRevision !== null &&
    input.remoteRevision === input.lastLocalSaveRevision
  ) {
    return false;
  }
  if (input.remoteRevision === input.localRevision) {
    return false;
  }
  return true;
}
