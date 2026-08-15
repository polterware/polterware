export type CommandContext = {
  metaKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  key: string;
  code: string;
  targetIsEditable: boolean;
  canvasFocused: boolean;
};

export type Command = {
  id: string;
  label: string;
  run: (ctx: CommandContext) => void;
  isEnabled?: (ctx: CommandContext) => boolean;
  keyTest?: (ctx: CommandContext) => boolean;
  shortcutHint?: string;
};

export type CommandRegistry = {
  commands: Map<string, Command>;
};

export function createCommandRegistry(): CommandRegistry {
  return { commands: new Map() };
}

export function registerCommand(reg: CommandRegistry, cmd: Command): void {
  reg.commands.set(cmd.id, cmd);
}

export function dispatch(reg: CommandRegistry, id: string, ctx: CommandContext): boolean {
  const cmd = reg.commands.get(id);
  if (!cmd) return false;
  if (cmd.isEnabled && !cmd.isEnabled(ctx)) return false;
  cmd.run(ctx);
  return true;
}

export function matchShortcut(
  reg: CommandRegistry,
  ctx: CommandContext,
): Command | null {
  if (ctx.targetIsEditable) return null;
  if (!ctx.canvasFocused) return null;
  for (const cmd of reg.commands.values()) {
    if (cmd.keyTest?.(ctx) && (!cmd.isEnabled || cmd.isEnabled(ctx))) {
      return cmd;
    }
  }
  return null;
}

export function listCommands(reg: CommandRegistry): Command[] {
  return [...reg.commands.values()];
}
