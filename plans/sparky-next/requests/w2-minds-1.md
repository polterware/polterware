---
summary: Extend the shell registry for the Minds sidebar tree
rationale: W2-MINDS needs to contribute its live nested Mind tree without importing or coupling to the shell implementation.
---

W2-MINDS requests W2-SHELL to extend `packages/features/sparky-shell/src/feature.ts` with a typed sidebar tree contribution, preserving the existing flat `sidebar` API for other features. Proposed additions:

```ts
export type SidebarTreeDef = {
  id: string;
  component: Component;
};

export type SparkyFeature = {
  // existing fields
  sidebarTree?: SidebarTreeDef;
};
```

W2-MINDS will register a component that renders Minds using the design-system `TreeView`; the shell will render `sidebarTree.component` at the Minds navigation slot. Keep tree data and interactions owned by W2-MINDS.

The W0-SCAFFOLD owner should regenerate the root `pnpm-lock.yaml` after accepting the new W2-MINDS runtime dependencies on `@polterware/frontend-design-system` and `@polterware/sparky-contracts` (run `pnpm install --lockfile-only`).

The W2-MINDS instructions assign `apps/sparky-desktop/tests/e2e/minds.spec.ts` as a smoke test while W3-E2E owns runner configuration. This request covers the registry extension and lockfile coordination; W2-MINDS is creating the specifically named smoke spec per the wave-wide deliverables.
