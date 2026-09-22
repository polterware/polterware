/** Architecture commands for the app command palette — same ids as canvas registry. */

import { formatShortcut, type ShortcutCommandId } from "@polterware/frontend-state";

export type PaletteCommand = {
  id: string;
  label: string;
  group: string;
  shortcutId?: ShortcutCommandId;
};

export const ARCHITECTURE_PALETTE_COMMANDS: PaletteCommand[] = [
  { id: "tool.add_node", label: "Add to architecture", group: "Architecture", shortcutId: "architecture.add_node" },
  {
    id: "tool.connect",
    label: "Connect from selected component",
    group: "Architecture",
    shortcutId: "architecture.tool_connect",
  },
  {
    id: "view.fit_document",
    label: "Fit architecture to view",
    group: "Architecture",
    shortcutId: "architecture.fit_document",
  },
  {
    id: "view.fit_selection",
    label: "Fit selection",
    group: "Architecture",
    shortcutId: "architecture.fit_selection",
  },
  { id: "history.undo", label: "Undo canvas change", group: "Architecture", shortcutId: "architecture.undo" },
  { id: "history.redo", label: "Redo canvas change", group: "Architecture", shortcutId: "architecture.redo" },
  { id: "edit.auto_layout", label: "Auto layout selection", group: "Architecture" },
  { id: "edit.highlight_deps", label: "Highlight dependencies", group: "Architecture" },
  { id: "export.json", label: "Export architecture JSON", group: "Architecture" },
];

export function paletteCommandShortcut(command: PaletteCommand): string | undefined {
  if (!command.shortcutId) return undefined;
  const hint = formatShortcut(command.shortcutId);
  return hint || undefined;
}
