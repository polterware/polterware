# Contract: Architecture UI

**Owner**: `packages/features/architecture`  
**Consumers**: `App.tsx` composition  
**Extends**: `003` architecture-ui (canvas full-width, no permanent inspector)

## Tabs

`canvas` | `components` | `dependencies` | `views` | `changes`

## Props (normative)

```ts
type ArchitectureViewProps = {
  route: ClientRoute;
  onRoutePatch: (patch: Partial<ClientRoute>) => void;
  components: ArchitectureComponent[];
  dependencies: DependencyEdge[];
  views: ArchitectureView[];
  changes: ArchitectureChangeEvent[];
  mapModel?: /* existing canvas model */;
  loadState: LoadState;
  dataSource: "core" | "fixture" | "empty" | "cached";
  onOpenNode: (id: string) => void;
  onSelectNode: (id: string | null) => void;
  onCreateNode?: () => void;
  onSaveLayout?: () => void;
  onExportDependencies?: () => void;
};
```

## Canvas rules

1. Full-width content; no permanent right inspector.  
2. Selection = neutral border only.  
3. Enter / Open → detail page (`entityKind: "arch-node"`).  
4. Zoom/fit/lock controls present when map has content.  
5. Shell remains interactive during pan/zoom on prototype-scale graphs.

## Detail page

Must expose: identity, status chips, overview, relationship mini-view, connected list, activity, notes/metadata, quick actions (open resource, logs, run).

## Dialogs

- Create node → `dialog.type === "arch-node-create"`
