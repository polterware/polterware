# Contract: Workspace Breadcrumb Controls

**Owner**: `packages/features/shell/src/context-switchers.tsx` (+ `title-bar.tsx` props)  
**Consumers**: Desktop Shell title bar  
**Related**: [hierarchy-shell.md](./hierarchy-shell.md)

---

## Organization segment

### Structure

```text
[ Org name button ]  /  [ Project name button ]  /  [ Environment kind badge ]
```

Each visible context has one ghost trigger. Direct navigation lives inside the corresponding switcher menu.

### Behavior

| Control | Action | Navigates? |
| --- | --- | --- |
| Org **name** | Toggle org switcher popover | No (until menu item chosen) |
| **Organization home** menu item | `onOpenOrganizationHome?.(organizationId)` | Yes → org home |

### Switcher menu contents

1. Search field (“Find organization…”)
2. List of accessible organizations (current marked)
3. Footer actions:
   - **Organization home** → current organization home
   - **All organizations** → `onManageOrganizations` / `onOpenOrganizationsRoot` (hierarchy root)
   - **New organization** → create/onboarding entry

### A11y

- Name button: accessible name includes organization name (e.g. “Switch organization: {name}”); `aria-expanded` reflects the popover.
- Popover: existing dismiss/focus patterns via design-system Popover.

---

## Project segment

### Structure (aligned)

```text
[ Project name button ]
```

| Control | Action | Navigates? |
| --- | --- | --- |
| Project **name** | Toggle project switcher | No until selection |
| **Project overview** menu item | `onOpenOverview` | Yes → project Overview |

### Switcher rules

- Lists projects filtered to **current organization only**.
- Selecting a project switches project workspace (stays in project layout mode).
- Footer includes Project overview and New project.
- MUST NOT list other organizations’ projects as peers without org grouping that implies leaving hierarchy incorrectly; preferred: only current org.

---

## Environment segment

- The breadcrumb shows only the current environment kind as a filled semantic badge: `PRODUCTION`, `STAGING`, `DEVELOPMENT`, or `LOCAL`.
- The full-height ghost button around the badge opens the project-scoped environment switcher; the environment name remains available in its accessible label and tooltip.
- Dropdown rows display only filled kind badges; each row keeps the environment name and kind in its accessible label.
- The switcher contains only environment choices and has no footer navigation action.

---

## Hierarchy layout breadcrumb

When Shell is in **hierarchy** mode:

| Surface | Org segment | Project segment | Env segment |
| --- | --- | --- | --- |
| Organizations root | “Organizations” opens the organization switcher | Hidden | Hidden |
| Organization home | Current org name opens the organization switcher | Hidden | Hidden |

Exact visual polish is implement-time; **must not** show a real project name as selected when no project workspace is active.

---

## Callback surface (Shell / TitleBar)

```ts
type HierarchyChromeHandlers = {
  onOpenOrganizationsRoot?: () => void; // All organizations
  onOpenOrganizationHome?: (organizationId: string) => void; // Organization home menu action
  onSelectOrganization?: (organizationId: string) => void; // alias → home (R4)
  onOpenOverview?: () => void; // Project overview menu action
  onSelectProject?: (projectId: string) => void;
  onCreateOrganization?: () => void;
  onCreateProject?: () => void;
};
```

**Breaking change vs 007 wiring**: `onManageOrganizations` MUST open organizations root, not Settings.

---

## Regression checklist (contract tests / manual)

1. Organization and project names open their switchers without navigating.
2. Environment breadcrumb shows only its filled semantic kind badge.
3. Organization home and Project overview actions navigate to their documented destinations; environment selection only updates context.
4. All organizations ≠ Settings.
5. Menu select org → organization home, not first project auto-enter.
