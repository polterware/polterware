---
summary: Regenerate the lockfile for the Focus feature importer
rationale: W2-FOCUS added workspace and library dependencies inside packages/features/sparky-focus/package.json. Root pnpm-lock.yaml is owned by W0-SCAFFOLD.
---

Importer delta for `@polterware/feature-sparky-focus`:

```json
{
  "dependencies": {
    "@bufbuild/protobuf": "^2.15.0",
    "@polterware/feature-sparky-shell": "workspace:*",
    "@polterware/frontend-design-system": "workspace:*",
    "@polterware/sparky-contracts": "workspace:*"
  },
  "devDependencies": {
    "jsdom": "^30.1.1",
    "solid-js": "^1.9.15",
    "vite-plugin-solid": "^2.11.14",
    "vitest": "^5.0.1"
  }
}
```

Run `pnpm install --lockfile-only` after accepting this package.json. Do not treat a Focus-authored lockfile as the merge result.
