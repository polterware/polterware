# Contract: Hierarchy Shell (Org → Project)

**Owner**: `packages/features/shell` + `apps/desktop/src/renderer/App.tsx`  
**Consumers**: Title bar breadcrumb, hierarchy pages, Desktop App route Match tree  
**Supersedes**: `007` chrome mapping of “Manage / All organizations” → `settings` + tab `organization` for **discovery** purposes

---

## Chrome sections (additive)

```ts
type ChromeSectionId =
  | "overview"
  | "search"
  | "sync"
  | "onboarding"
  | "organizations" // NEW — organizations root
  | "organization"; // NEW — organization home (projects)

// NavSectionId unchanged (eight project destinations)
```

Rules:

1. `organizations` and `organization` MUST NOT appear in `DEFAULT_NAV`.
2. When `section` is `organizations` or `organization`, Shell MUST NOT present the eight project destinations as primary sidebar navigation.
3. When `section` is a `NavSectionId` (or project-scoped chrome such as `overview`), project sidebar remains available as today.

---

## ClientRoute fields (additive)

```ts
type ClientRoute = {
  section: string;
  tab: string;
  page: "index" | "detail";
  // existing fields…
  environmentId?: string;
  organizationId?: string; // NEW
  projectId?: string; // NEW — optional on hierarchy chrome
};
```

### Validity

| section | organizationId | projectId |
| --- | --- | --- |
| `organizations` | optional (last-known OK) | optional / ignored for chrome |
| `organization` | **required** | optional / ignored for chrome |
| `NavSectionId` | should be set | should be set |

### Helpers (normative names — implement as pure functions)

```ts
goOrganizationsRoot(route: ClientRoute): ClientRoute;
goOrganizationHome(route: ClientRoute, organizationId: string): ClientRoute;
enterProject(
  route: ClientRoute,
  projectId: string,
  opts?: { organizationId?: string; environmentId?: string; section?: string },
): ClientRoute;
```

- `goOrganizationsRoot` → `section: "organizations"`, clear detail/dialog noise.
- `goOrganizationHome` → `section: "organization"`, set `organizationId`.
- `enterProject` → set project (+ org if provided), default environment if needed, land on default project section (e.g. `overview` or `architecture` — product default must be single and documented in implement tasks).

`historySnapshot` MUST include `organizationId` and `projectId`.

---

## Shell layout modes

> **Update (settings-scope / org workspace):** `organization` is no longer “no sidebar”.
> It uses an **organization** sidebar. `organizations` root remains hierarchy (no rail).
> Neutral routes: `account`, `create-organization`, `create-project`, `onboarding`.

```ts
type ShellLayoutMode = "project" | "organization" | "hierarchy" | "neutral";

function shellLayoutMode(section: string): ShellLayoutMode {
  if (section === "organization") return "organization";
  if (section === "organizations") return "hierarchy";
  if (
    section === "account" ||
    section === "create-organization" ||
    section === "create-project" ||
    section === "onboarding"
  ) {
    return "neutral";
  }
  return "project";
}
```

| Mode | Sidebar | Main |
| --- | --- | --- |
| `project` | Project `SidebarNav` | Feature views |
| `organization` | Org nav: Projects · Team · Billing · Settings | Org workspace tabs |
| `hierarchy` | Hidden | Organizations root |
| `neutral` | Hidden (account page may have local rail) | Account / create forms |

---

## App composition (Match)

Desktop App MUST render:

| `route.section` | View |
| --- | --- |
| `organizations` | Organizations index page |
| `organization` | Organization home page (projects for `route.organizationId`) |

### Entry point retargets

| Callback / action | Target |
| --- | --- |
| `onManageOrganizations` / “All organizations” | `goOrganizationsRoot` |
| **Organization home** switcher action | `goOrganizationHome(currentOrgId)` |
| Select org in switcher menu | `goOrganizationHome(selectedOrgId)` |
| Select project on org home | `enterProject(projectId)` |
| Settings → Organization tab | Still settings admin (not catalog) |
| Account menu “account” | Unchanged product choice; MUST NOT be the only path to org catalog |

---

## Page UI contracts (behavioral)

### Organizations root

- Title region: “Your Organizations” (or product-equivalent).
- Search filters organizations by name.
- List rows/cards: name, plan label, project count (derived OK).
- Primary action: New organization.
- Row activate → organization home.
- Fixture honesty when data is fixture.

### Organization home

- Title region: organization name (+ plan if useful).
- Search filters projects of **this** org only.
- List rows/cards: project name, environment summary.
- Primary action: New project (scoped to org).
- Optional chrome action: All organizations → root.
- Row activate → enter project workspace.
- Empty state when zero projects.
- Fixture honesty when data is fixture.

---

## Non-goals in this contract

- Live Core org RPC shapes (future).
- Billing/SSO admin depth.
- Changing the eight `NavSectionId` labels or groups.
