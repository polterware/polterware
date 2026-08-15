# Contract: Architecture Deep Flows UI

**Owner**: `packages/features/architecture`  
**Extends**: 004 canvas/components/dependencies/views/changes

## Import resources

```ts
type ArchitectureImportProps = {
  sourceLabel: string;
  rows: ArchitectureImportRow[];
  suggestedDependencies: { from: string; to: string }[];
  step: number;
  onChangeRow: (discoveredId: string, patch: Partial<ArchitectureImportRow>) => void;
  onBack: () => void;
  onApply: () => void;
};
```

Actions per row: link-existing | create-and-link | ignore.

## Snapshot comparison

```ts
type SnapshotCompareProps = {
  leftId: string;
  rightId: string;
  onLeft: (id: string) => void;
  onRight: (id: string) => void;
  tab: "summary" | "components" | "dependencies" | "configuration";
  summary: SnapshotCompareSummary;
  onOpenVisualDiff?: () => void;
  onRestore?: () => void;
};
```

## View editor

```ts
type ViewEditorProps = {
  view: ArchitectureViewEdit;
  allComponents: { id: string; name: string }[];
  onChange: (patch: Partial<ArchitectureViewEdit>) => void;
  onSave: () => void;
  onCancel: () => void;
  dirty: boolean;
};
```

Cancel with dirty state → confirm discard.
