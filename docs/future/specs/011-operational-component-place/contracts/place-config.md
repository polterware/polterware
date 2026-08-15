# Contract: PlaceConfig & Project Environments

**Owner**: `packages/features/architecture/src/domain/place-config.ts` (+ project env helpers)

---

## PlaceConfig (normalized)

```ts
type PlaceConfig = {
  name: string;                 // trimmed, non-empty
  environmentId: string;        // must be in project env list
  description?: string;         // purpose
  role?: string;                // allowlisted id
  owner?: string;
  resourceId?: string;          // optional link
};
```

### normalizePlaceConfig(input, envs)

| Input | Result |
| --- | --- |
| empty name | `{ error: "Name is required" }` |
| env not in `envs` | `{ error: "Environment is required" }` |
| `envs.length === 0` | `{ error: "Add a project environment first" }` |
| invalid role | `{ error: "Role is invalid" }` |
| ok | `PlaceConfig` with optionals omitted when empty |

### metadataFromPlaceConfig

- If `role` set → `{ role }`  
- Else `undefined`

### environmentIdsFromConfig

- `[config.environmentId]`

---

## ProjectEnvironmentPort

```ts
type ProjectEnvironment = {
  id: string;
  label: string;
  kind?: string;
  protected?: boolean;
};

interface ProjectEnvironmentPort {
  listEnvironments(projectId?: string): ProjectEnvironment[];
}
```

**Fixture adapter**: returns at least Production, Staging, Development, Local with stable ids.

**UI**: Configure `Select` options = `listEnvironments()`; default = production-like if present else first.

---

## Role allowlist (v1)

`"" | primary | replica | edge | internal | cache | stream | other`

---

## Compatibility

- Prior commits may have used `environment: "production"` field name on PlaceConfig — migrate call sites to `environmentId`.  
- Document still uses `environmentIds: string[]` on ArchNode.  
