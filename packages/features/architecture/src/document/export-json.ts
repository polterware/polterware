import type { ArchitectureDocument, SelectionState } from "@polterware/frontend-canvas";
import { buildFragment, serializeDocument, serializeFragment } from "@polterware/frontend-canvas";

export function exportDocumentJson(doc: ArchitectureDocument): string {
  return serializeDocument(doc);
}

export function exportSelectionJson(
  doc: ArchitectureDocument,
  sel: SelectionState,
): string {
  return serializeFragment(buildFragment(doc, sel));
}

export function downloadText(filename: string, text: string, mime = "application/json"): void {
  const blob = new Blob([text], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}
