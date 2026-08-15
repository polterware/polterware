# Contract: Overlays (Dialogs, Drawer, Command Palette)

**Owner**: design-system primitives + feature dialogs + `feature-commander`  
**Consumers**: App composition and feature pages

## Dialogs

Common rules:

1. Focus trap + Escape + restore focus  
2. Cancel leaves no residual permanent panel  
3. Validation keeps dialog open with errors  
4. Primary action disabled while submitting  

### Create architecture node

Fields: name, type, environment, owner, description, linked resource optional.

### Run command

Fields: target, command, working directory, environment.  
If approval required: primary CTA = “Request execution” (not “Run now” that bypasses policy).

### Credential create

Secret field write-only; never rehydrate into list after success.

## Process drawer

```ts
type ProcessDrawerProps = {
  process: ProcessRow | null;
  open: boolean;
  onClose: () => void;
  onViewLogs?: () => void;
  onOpenTerminal?: () => void;
  onRestart?: () => void;
  onStop?: () => void;
};
```

Rules:

1. Only process entities use drawer in this feature.  
2. Close clears `route.drawer`.  
3. Dead PID → terminated state, no crash.

## Command palette

```ts
type CommandPaletteProps = {
  open: boolean;
  onClose: () => void;
  onNavigate: (section: SectionId) => void;
  onAction: (actionId: string) => void;
  recent: { id: string; label: string; entityKind?: string }[];
};
```

Rules:

1. Overlay only—never a sidebar destination.  
2. Groups: Navigation (7), Actions, Recent.  
3. Open via platform shortcut (⌘K / Ctrl+K) from App.  
4. Does not permanently consume MainSurface layout.
