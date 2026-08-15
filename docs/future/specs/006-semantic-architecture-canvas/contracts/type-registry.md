# Contract: Type Registries & Suggestions

**Package**: `@polterware/feature-architecture` domain modules  
**Consumers**: catalog dialog, host chrome, connection flow, import validation  
**Must not import**: Electron, Connect clients, Solid runtime (pure TS preferred)

---

## ComponentTypeDefinition

```ts
type ComponentCategory =
  | "applications"
  | "data"
  | "infrastructure"
  | "development"
  | "external"
  | "organization"
  | "connected"; // action rows, not placeable types

type ComponentTypeDefinition = {
  id: string; // NodeType id
  label: string;
  category: ComponentCategory;
  description: string;
  icon: string; // Hugeicons icon name key
  defaultWidth: number;
  defaultHeight: number;
  defaultPorts: Port[];
  searchTerms: string[];
  synonyms: string[];
  order: number;
  placeable: boolean; // false for connected-resource actions
  appearance?: { compactSubtitle?: string };
};
```

### Required APIs

```ts
listComponentTypes(): ComponentTypeDefinition[]
getComponentType(id: string): ComponentTypeDefinition | undefined
searchComponentTypes(query: string): ComponentTypeDefinition[]
// matches label, category, searchTerms, synonyms (case-insensitive)
```

### Search examples (must pass tests)

| Query | Expect includes |
| --- | --- |
| postgres | database |
| rabbitmq | queue |
| next | web_application |
| cron | scheduled_job (when type exists) |

### Favorites / recents (prefs port)

```ts
type TypePrefsPort = {
  getRecents(): string[];
  pushRecent(id: string): void;
  getFavorites(): string[];
  toggleFavorite(id: string): void;
};
```

Not part of ArchitectureDocument. Domain module defines the port + in-memory fake only. Browser `localStorage` adapter (`type-prefs-local.ts`) implements the port — domain MUST NOT import DOM/storage APIs (Constitution II).

---

## RelationshipTypeDefinition

```ts
type RelationshipTypeDefinition = {
  id: string; // RelationshipType id
  label: string;
  description?: string;
  order: number;
};
```

### Required APIs

```ts
listRelationshipTypes(): RelationshipTypeDefinition[]
getRelationshipType(id: string): RelationshipTypeDefinition | undefined
labelForRelType(id: string): string
```

---

## Connection rules

```ts
canConnect(
  source: ArchNode,
  target: ArchNode,
  relType: string,
): { ok: boolean; reason?: string }

suggestRelationshipTypes(
  source: ArchNode,
  target: ArchNode,
): string[] // ordered; first is default focus
```

### Invariants

- Notes never connect.
- Self-connect: ok only for explicitly allowed types (default false).
- `suggestRelationshipTypes` never auto-commits; host must confirm unless quick mode has preselected type.
- Quick mode type must still pass `canConnect`.

---

## Extensibility

Adding a type:

1. Add definition to registry data.
2. Add id to document `NodeType` / `RelationshipType` union if serialized.
3. Add synonyms/search terms.
4. Optionally add suggestion rows.
5. **No** host `if (type === "api")` branches for appearance — read registry.

Custom types: `custom_resource` / `custom` plus metadata for future user-defined types.
