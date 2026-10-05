export type ShortcutRow = {
  id: string;
  group: string;
  action: string;
  keys: string;
};

/** Read-only keymap from 06-ui.md and the shell shortcut map. */
export const SHORTCUTS: ShortcutRow[] = [
  { id: "commander", group: "Shell", action: "Commander", keys: "⌘K" },
  { id: "quick-add", group: "Shell", action: "New memory", keys: "⌘N" },
  { id: "settings", group: "Shell", action: "Settings", keys: "⌘," },
  { id: "back", group: "Shell", action: "History back", keys: "⌘[" },
  { id: "forward", group: "Shell", action: "History forward", keys: "⌘]" },
  { id: "close-detail", group: "Shell", action: "Close detail", keys: "⌘J" },
  { id: "route-1", group: "Shell", action: "Inbox", keys: "⌘1" },
  { id: "route-2", group: "Shell", action: "Today", keys: "⌘2" },
  { id: "route-3", group: "Shell", action: "Calendar", keys: "⌘3" },
  { id: "route-4", group: "Shell", action: "Minds", keys: "⌘4" },
  { id: "route-5", group: "Shell", action: "Map", keys: "⌘5" },
  { id: "route-6", group: "Shell", action: "Focus", keys: "⌘6" },
  { id: "search", group: "Shell", action: "Focus search in lists", keys: "/" },
  { id: "open", group: "Memory", action: "Open selected", keys: "↵" },
  { id: "toggle-done", group: "Memory", action: "Toggle done", keys: "Space" },
  { id: "edit-title", group: "Memory", action: "Edit title", keys: "E" },
  { id: "pin", group: "Memory", action: "Pin", keys: "P" },
  { id: "delete", group: "Memory", action: "Delete (confirm)", keys: "⌘⌫" },
];
