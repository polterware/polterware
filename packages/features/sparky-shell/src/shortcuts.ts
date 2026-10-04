export type ShellShortcut = "commander" | "quick-add" | "settings" | "back" | "forward" | "close-detail" | "route-1" | "route-2" | "route-3" | "route-4" | "route-5" | "route-6" | "search";
export function shellShortcut(event: Pick<KeyboardEvent, "key" | "code" | "metaKey" | "ctrlKey" | "altKey" | "shiftKey" | "isComposing">, editing: boolean): ShellShortcut | undefined {
  if (event.isComposing || event.altKey) return;
  const key = event.key.toLowerCase();
  if (event.metaKey || event.ctrlKey) {
    if (event.shiftKey) return;
    if (/^[1-6]$/.test(key)) return `route-${key}` as ShellShortcut;
    return ({ k: "commander", n: "quick-add", ",": "settings", "[": "back", "]": "forward", j: "close-detail" } as const)[key as "k"];
  }
  if (key === "/" && !editing) return "search";
}
export function isEditing(target: EventTarget | null): boolean {
  return target instanceof Element && !!target.closest('input, textarea, select, [contenteditable="true"], [role="textbox"]');
}
