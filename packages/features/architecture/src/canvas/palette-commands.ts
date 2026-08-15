/** Architecture commands for the app command palette — same ids as canvas registry. */

export type PaletteCommand = {
  id: string;
  label: string;
  group: string;
  shortcut?: string;
};

export const ARCHITECTURE_PALETTE_COMMANDS: PaletteCommand[] = [
  { id: "tool.add_node", label: "Add to architecture", group: "Architecture", shortcut: "N" },
  {
    id: "tool.connect",
    label: "Connect from selected component",
    group: "Architecture",
    shortcut: "C",
  },
  { id: "view.fit_document", label: "Fit architecture to view", group: "Architecture", shortcut: "⇧1" },
  { id: "view.fit_selection", label: "Fit selection", group: "Architecture", shortcut: "⇧2" },
  { id: "history.undo", label: "Undo canvas change", group: "Architecture", shortcut: "⌘Z" },
  { id: "history.redo", label: "Redo canvas change", group: "Architecture", shortcut: "⇧⌘Z" },
  { id: "edit.auto_layout", label: "Auto layout selection", group: "Architecture" },
  { id: "edit.highlight_deps", label: "Highlight dependencies", group: "Architecture" },
  { id: "export.json", label: "Export architecture JSON", group: "Architecture" },
];
