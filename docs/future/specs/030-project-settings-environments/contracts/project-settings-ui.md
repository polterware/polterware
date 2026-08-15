# Contract: Project Settings UI

**Owner**: `packages/features/settings`  
**Consumers**: `apps/desktop` (`SettingsView`)

## Tabs

Exactly:

| id | label |
| --- | --- |
| `general` | General |
| `environments` | Environments |
| `advanced` | Advanced |

**Forbidden tab ids in TABS array**: `integrations`, `access`, `knowledge`, `people`, `teams`, `agents`, `policies`, `audit`, `connected`, `catalog`.

Legacy route tab values normalize to `general` or `advanced` without rendering forbidden tabs.

## General

```ts
type GeneralSettingsForm = {
  name: string;
  description: string;
  defaultEnvironmentId: string;
  projectId: string;
  onSave: () => void; // mock toast only
};
```

- Controls: design-system `Field` + `Input` / `Textarea` / `Select`
- Save toast copy MUST include mock/session honesty (e.g. “saved (mock)”)

## Environments table

```ts
type EnvironmentRow = {
  id: string;
  name: string;
  kind: "production" | "staging" | "development" | "local";
  protection: "strict" | "standard" | "relaxed";
  resourceCount: number;
  nodeCount: number;
  isDefault: boolean;
};
```

- Compound `DataTable` only
- `PrimaryCell` + domain icon (Server or equivalent Hugeicon from design-system)
- Kind and protection: `StatusChip` via helpers
- Row activate → `detailPanelStore.activateTarget({ scopeId: "settings:environments", entityId, entityKind: "environment" })`

## Environment detail

```ts
type EnvironmentDetailProps = {
  environment: ProjectEnvironment | null | undefined;
  loading?: boolean;
  error?: string | null;
  notFound?: boolean;
  onPromote?: (id: string) => void;
  onDemote?: (id: string) => void;
};
```

- `DetailPanel.Content` + `Body layout="sections"` + `Grid`
- Overview metrics/metadata: kind, protection, default, resources, nodes
- Section: protection rules summary
- Actions: Promote / Demote (mock toast honesty)
- Loading / error / not-found honest states

## Advanced

- Retention selects via Field + Select
- Danger zone: Archive project → `A11yDialog` confirm
- Confirm toast MUST NOT claim core wipe/delete completed against real storage
- Delete project / wipe core remains unavailable or clearly non-operative — no false success

## Honesty

- Fixtures: `dataSource: "fixture"`
- Toasts for save, promote, demote, add environment, archive: mock wording
