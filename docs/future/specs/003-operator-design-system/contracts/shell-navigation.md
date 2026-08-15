# Contract: Shell & Navigation

**Owner**: `packages/features/shell` + `apps/desktop/src/renderer/App.tsx`  
**Consumers**: All feature views rendered inside MainSurface

## Shell props (normative shape)

```ts
type ShellProps = {
  route: ClientRoute; // or accessors
  onNavigate: (next: Partial<ClientRoute> | ClientRoute) => void;
  status: ShellStatus;
  navItems: NavItem[];
  children: JSX.Element; // MainSurface page body only
};

type NavItem = {
  id: string;           // section id
  label: string;
  group?: string;       // CORE | INSIGHTS | CONFIGURATION
  icon?: string;        // Hugeicons icon name key
  active?: boolean;
};
```

## Layout guarantees

1. Sidebar background matches shell/app family (integrated).  
2. Exactly one MainSurface hosts page header + content.  
3. No permanent right inspector slot in Shell for Resources/Architecture.  
4. Toast region is a fixed portal/layer, not a layout column.  
5. Dialog layer is modal above MainSurface.

## Navigation rules

| Intent | Mechanism |
| --- | --- |
| Change primary section | `onNavigate({ section, page: "index", entityId: undefined })` |
| Open resource/node detail | `onNavigate({ page: "detail", entityId })` |
| Back to index | `onNavigate({ page: "index", entityId: undefined })` |
| Create/edit | Set dialog state; do not change section unless required |
| Command palette | Overlay; does not replace MainSurface |

## Accessibility

- Nav items are buttons or links with `aria-current="page"` when active  
- Focus-visible rings use border-strong / text-primary outline, not brand glow  
- Sidebar collapse behavior (if any) remains keyboard reachable  

## Out of contract

- Domain mutation  
- Electron BrowserWindow APIs inside feature views (chrome only via shell CSS / Main)
)
