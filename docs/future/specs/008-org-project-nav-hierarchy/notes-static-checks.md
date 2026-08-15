# Static checks (008)

**Date**: 2026-08-03

## Unit tests

```bash
cd apps/desktop && ./node_modules/.bin/vitest run --dir ../../packages/features/shell/src
```

Result: **21 passed** (`route.test.ts` including hierarchy helpers + fixture filters).

## Grep dual path

```bash
rg -n "onManageOrganizations|Manage organizations" apps/desktop packages/features/shell
```

Result: App wires `goOrganizationsRoot`; no chrome path to `settings` + `organization` for discovery.

## Manual quickstart (S1–S6)

Requires interactive Desktop (`pnpm dev` only when Erick asks). Unit coverage maps:

| Scenario | Covered by |
| --- | --- |
| S1 org name → home | `goOrganizationHome` + App wire |
| S2 chevron menu only | split controls in context-switchers |
| S3 All orgs → root | `goOrganizationsRoot` + App retarget |
| S4 enter project | `enterProject` + sidebar mode project |
| S5 select other org | menu → org home (no auto project) |
| S6 Settings boundary | admin copy + optional All organizations link |

## Convergence

FR-001–FR-024 addressed in shell + App. SC-001–SC-007 unit/static for helpers and wiring.
