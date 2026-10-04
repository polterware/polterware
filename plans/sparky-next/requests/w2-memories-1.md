# W2-MEMORIES: registry memory hosts

Owner: W2-SHELL. No frozen RPC change.

The v1.4 shell registry only has parameterless routes and detail sections; it cannot host the memory editor or compose the Mind header with its memory list. W2-MEMORIES exports these components and contributes the following structurally typed optional fields. Add them to `packages/features/sparky-shell/src/feature.ts`:

```ts
memoryDetail?: Component<{ memoryId: string; sections: DetailSectionDef[] }>
memoryCollection?: Component<{ mindId: string }>
```

In the shell's single `DetailPanel.Root`, mount the registry's `memoryDetail` component inside `DetailPanel.Content`, passing the selected memory ID and all features' `detailSections`. W2-MEMORIES supplies Body/Main/Rail and orders the visible contributions; the shell supplies Header/close/expand. W2-SHELL now exposes `useSparkyShell` from the registry entry; section owners can use its `selectedMemoryId`. Section components remain parameterless, so expose the selected ID in shell context for their owners (or add an optional `memoryId` prop with a coordinated request).

Handle `window.sparky.app.openDeepLink('sparky://memory/<id>')` by selecting that ID and opening details. Today's and Inbox's registered routes are `/today` and `/inbox`. W2-MINDS should compose `memoryCollection({ mindId })` under its own `/mind/:id` header through the registry, without importing this package. Preserve draft edits when closing or switching details.

W2-MEMORIES does not modify shell or renderer ownership.

The in-progress W2-SHELL already has `registerMemoryDetail`, `detailRenderer` and a `detailRoute` fallback. To wire the always-available editor without depending on visiting Today first, initialize/select the renderer from the registry's optional `memoryDetail` field and pass `sections={detailSections()}` along with `memoryId`. Update `MemoryDetailRenderer` to accept optional sections. Do not also render sections in the shell when this editor is active. Keep the existing registration hook for callers that need it.

W2-MINDS needs the optional `memoryCollection` field made available through `context.features`; it owns its page header and passes the current Mind ID. The `memories.create` command creates an untitled memory through the core and opens it for editing; the shell's existing Quick Add remains responsible for natural-language input.
