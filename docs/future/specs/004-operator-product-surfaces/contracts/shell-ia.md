# Contract: Shell & Information Architecture

**Owner**: `packages/features/shell` + `apps/desktop/src/renderer/App.tsx`  
**Consumers**: All feature views inside MainSurface  
**Supersedes for nav**: `003` shell-navigation DEFAULT_NAV multi-item list

## Normative navigation

```ts
type SectionId =
  | "architecture"
  | "resources"
  | "ops"
  | "knowledge"
  | "access"
  | "integrations"
  | "settings";

const DEFAULT_NAV: NavItem[] = [
  { id: "architecture", label: "Architecture", group: "CORE" },
  { id: "resources", label: "Resources", group: "CORE" },
  { id: "ops", label: "Operations", group: "CORE" },
  { id: "knowledge", label: "Knowledge", group: "KNOWLEDGE" },
  { id: "access", label: "Access", group: "KNOWLEDGE" },
  { id: "integrations", label: "Integrations", group: "CONFIGURATION" },
  { id: "settings", label: "Settings", group: "CONFIGURATION" },
];
```

Groups MUST render as labeled sections in the sidebar. Exactly **seven** destinations.

## Shell props

```ts
type ShellProps = {
  route: () => ClientRoute;
  onNavigateSection: (section: SectionId) => void;
  status: ShellStatus;
  navItems?: NavItem[]; // default DEFAULT_NAV
  canGoBack: boolean;
  canGoForward: boolean;
  onBack: () => void;
  onForward: () => void;
  onAvatarClick?: () => void;
  children: JSX.Element; // MainSurface body only
};
```

## Layout guarantees

1. Title bar: history controls + profile avatar affordance.  
2. Sidebar integrated with app background.  
3. Exactly one MainSurface for page content.  
4. Core online + nodes connected + actor identity in sidebar footer region.  
5. No permanent right inspector column in Shell.  
6. Toast portal + dialog/drawer layers above MainSurface.

## Legacy section map

| Legacy id | Redirect |
| --- | --- |
| processes | resources + tab processes |
| approvals | ops + tab approvals |
| workflows | ops + tab workflows |
| agents | access + tab agents |
| commander | open command palette (stay on current section) |
| logs | ops runs or last context; prefer quick action open logs pane |

## Accessibility

- Active nav: `aria-current="page"`  
- Focus-visible neutral rings  
- Group labels not focus stops (or correctly labeled)

## Out of contract

- Domain mutation  
- Electron BrowserWindow APIs inside feature views
