# Contract: Architecture Domain & UI

**Owner**: `packages/features/architecture`  
**Consumes**: `@polterware/frontend-canvas`, `@polterware/frontend-design-system`  
**Composed by**: `apps/desktop/src/renderer/App.tsx`

## Domain registry

Provides:

- Node type catalog (labels, icons, default ports, default size)
- Relationship type catalog
- Connection validation rules
- State → StatusChip tone mapping
- Evidence display policy (dashed vs solid, etc.)
- Environment filter application

Pure helpers must be unit-testable without Electron.

## Views

### ArchitectureCanvasHost (replaces static ArchitectureCanvas)

| Input | Role |
| --- | --- |
| document / model | ArchitectureDocument v1 |
| revision | RPC revision |
| loading / error / dataSource | honesty |
| environmentId | filter |
| search | query string |
| selectedNodeId | controlled selection bridge to route |
| savePort | `(bodyJson, expectedRevision) => Promise<newRevision>` |
| onOpenNode | detail navigation |
| onSelect | route silent update |
| sessionKey | systemId for viewport session cache |

**UI duties**

- Full-width canvas in MainSurface; **no permanent inspector**
- Tools toolbar, minimap, zoom indicator, save status
- Context menus, floating selection chrome
- Empty / loading / error / fixture banner
- Double-click / Enter → `onOpenNode`
- DESIGN.md compliance (quiet, not Excalidraw chrome)

### ArchitectureNodeDetailView

Unchanged product role: page-based depth. May enrich from document relationships.

### Supporting tabs

Components / Dependencies / Views / Changes remain; should read from same document model when live, fixtures when not.

## Create flows

| Entry | Behavior |
| --- | --- |
| Dialog `arch-node-create` | May remain; submits into document command |
| Tool Add node | Click-to-place or place at viewport center |
| Context menu / double-click empty / palette | Same create command |

## Forbidden

- Electron imports in feature package  
- Permanent right inspector  
- Claiming non-evidence edges as live wiring  
- Brand-green selection glow / Excalidraw-like colorful toolbar  
