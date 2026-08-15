export const GLOBAL_COMMANDER_SHORTCUT = "CommandOrControl+Shift+K";

export interface GlobalShortcutRegistry {
  register(accelerator: string, callback: () => void): boolean;
  unregister(accelerator: string): void;
}

export interface ShortcutLogger {
  info(message: string): void;
  warn(message: string): void;
}

export function registerGlobalCommanderShortcut({
  log,
  onToggleCommander,
  shortcuts,
}: {
  log: ShortcutLogger;
  onToggleCommander: () => void;
  shortcuts: GlobalShortcutRegistry;
}): () => void {
  const registered = shortcuts.register(GLOBAL_COMMANDER_SHORTCUT, onToggleCommander);

  if (!registered) {
    log.warn(
      `Could not register global Commander shortcut ${GLOBAL_COMMANDER_SHORTCUT}. Another app may already own it.`,
    );
    return () => {};
  }

  log.info(`Registered global Commander shortcut ${GLOBAL_COMMANDER_SHORTCUT}.`);
  return () => shortcuts.unregister(GLOBAL_COMMANDER_SHORTCUT);
}
