# Contract: Shell & Information Architecture v2

**Owner**: `packages/features/shell` + `apps/desktop/src/renderer/App.tsx`  
**Consumers**: All feature views inside MainSurface  
**Supersedes**: `004` shell-ia seven-destination map for product nav going forward

## Normative navigation

```ts
type NavSectionId =
  | "architecture"
  | "resources"
  | "ops"
  | "observability"
  | "knowledge"
  | "access"
  | "integrations"
  | "settings";

type ChromeSectionId = "overview" | "search" | "sync" | "onboarding";

type SectionId = NavSectionId | ChromeSectionId;

const DEFAULT_NAV: NavItem[] = [
  { id: "architecture", label: "Architecture", group: "CORE" },
  { id: "resources", label: "Resources", group: "CORE" },
  { id: "ops", label: "Operations", group: "CORE" },
  { id: "observability", label: "Observability", group: "INSIGHTS" },
  { id: "knowledge", label: "Knowledge", group: "INSIGHTS" },
  { id: "access", label: "Access", group: "INSIGHTS" },
  { id: "integrations", label: "Integrations", group: "CONFIGURATION" },
  { id: "settings", label: "Settings", group: "CONFIGURATION" },
];
```

Rules:

1. Exactly **eight** primary destinations in `DEFAULT_NAV`.  
2. Groups: **CORE**, **INSIGHTS**, **CONFIGURATION** (not KNOWLEDGE).  
3. Chrome sections are routable but **must not** appear in `DEFAULT_NAV`.  
4. Subdivisions remain in-page tabs.

## Shell entry points

| Control | Behavior |
| --- | --- |
| Logo / project name | `goSection(overview)` or dedicated `openOverview()` |
| Chevron beside name | Open project switcher popover |
| Bell | `drawer = { type: "notifications" }` |
| Footer core status | `goSection(sync)` |
| ⌘K / Ctrl+K | Command palette (unchanged ownership) |
| Palette “Show all results” | `section=search`, `search=query` |

## Shell props extensions

```ts
type ShellProps = {
  route: () => ClientRoute;
  onNavigateSection: (section: SectionId) => void;
  onOpenOverview?: () => void;
  onOpenProjectSwitcher?: () => void;
  onOpenNotifications?: () => void;
  onOpenSync?: () => void;
  status: ShellStatus;
  navItems?: NavItem[];
  // history + identity as in 004
  children: JSX.Element;
};
```

## Active nav rules

- When `section` is a `NavSectionId`, that item is `aria-current="page"`.  
- When `section` is chrome (`overview` | `search` | `sync` | `onboarding`), **no** primary nav item is active (or overview may leave previous selection cleared).  
- Do not highlight Observability when user is on Logs via a non-observability path.

## Legacy map (additive)

| Input | Redirect |
| --- | --- |
| seven-destination deep links | still resolve; knowledge/access remain |
| `logs` (legacy) | prefer `observability` + tab `logs` (004 mapped to ops runs — **update** to observability when 007 lands) |
| missing project | `onboarding` |

## Accessibility

- Project switcher: focus trap in popover, Escape closes, restore focus.  
- Logo control: keyboard activatable button/link.  
- Group labels not focus stops.

## Out of contract

- Domain mutation  
- Electron BrowserWindow APIs inside feature views
