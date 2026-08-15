import type { ArchitectureDocument } from "@polterware/frontend-canvas";
import { validateImportJson, serializeDocument } from "@polterware/frontend-canvas";

export type ImportResult =
  | { ok: true; doc: ArchitectureDocument; backup: string }
  | { ok: false; error: string };

/**
 * Validate and prepare import. Does not replace current until caller commits.
 */
export function prepareImport(
  current: ArchitectureDocument,
  json: string,
): ImportResult {
  const check = validateImportJson(json);
  if (!check.ok || !check.result) {
    return { ok: false, error: check.error ?? "Invalid import" };
  }
  const hard = check.result.warnings.filter((w) => w.level === "error");
  if (hard.some((h) => /Unknown format|Malformed/.test(h.message))) {
    return { ok: false, error: hard[0]!.message };
  }
  return {
    ok: true,
    doc: check.result.doc,
    backup: serializeDocument(current),
  };
}
