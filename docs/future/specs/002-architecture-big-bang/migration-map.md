# Migration Map: Current → Target

**Mode**: Big bang · greenfield DISCARD for listed stores · git-only rollback · single Solid renderer final

Disposition: **P** preserve capability · **M** migrate · **R** replace · **C** consolidate · **X** remove · **D** deferred · **DISCARD** data

---

## File-level inventory requirement (FR-001 / CHK018)

Before structural deletes, produce:

1. `docs/migration/baseline/tree.txt` — relevant tree under `apps/`, `packages/`, `legacy/`
2. `docs/migration/baseline/file-register.md` — one row per **relevant** file or file group with columns: path · role · disposition (P/M/R/C/X/DISCARD/DEFERRED)

**Must cover**: entrypoints; Main; preload; renderer; workers/async (see research R18); services; persistence; IPC; integrations; scripts; configs; tests; updater; functional assets; legacy stores; React entry; preserve/migrate/remove targets.

**May omit**: trivial static logos/icons copies; `node_modules`; build `out/` artifacts; pure generated noise.

Validation: SC-001 — every register row has disposition + destination (T001, T008, T066).

---

## Async / worker disposition (CHK028)

See [research.md R18](./research.md). Summary: process exec → polterware-core; canvas layout → Web Worker; Main stays thin; skills MCP installer product surface → REMOVE; no utilityProcess this feature.

---

## Applications & packages

| Current | Destination | Strategy | Validation |
| --- | --- | --- | --- |
| Desktop Main (lifecycle, shortcuts, icons, **updater**) | Main thin + core supervisor + updater | P/M | boot + updater tests |
| Preload bridge | Typed Connect façade | R | security tests |
| React renderer features | Solid `packages/features/*` | R | e2e; then delete React |
| mock workbench product path | real core clients | X | grep prod |
| UI-only IPC ops | real core or delete | X | no UI-only success |
| packages/core | Go domains/providers | M then X package | zero app imports |
| packages/ui React | Solid design-system | R | no react on desktop product |
| skill preview/setup | — | **X REMOVE** | no product surface |
| MCP helpers | MCP runtime | M | policy tests |
| legacy/tui | — | X | zero imports |
| presentation | isolate | P | no domain imports |
| conf/pins/pipelines/repos/.polterware/polterware.yaml | new SQLite greenfield | **DISCARD data** | no dual-read tests |
| apps/web, server, node | not built | **D** | scenic check script |

---

## Data disposition (nominal)

| Store | Class | Notes |
| --- | --- | --- |
| globalPipelinesV1 | DISCARD | |
| desktopRepositoriesV1 | DISCARD | |
| pinnedCommandBasesV2 / pinnedCommandRunsV1 | DISCARD | |
| pinnedCommands legacy | DISCARD | |
| .polterware/config.json | DISCARD | |
| polterware.yaml as required old format | DISCARD | new declarative greenfield |
| mock/prototype product state | DISCARD | |
| process in-memory | N/A | ephemeral |
| App icon selection capability | PRESERVE capability | re-store greenfield OK |
| CONVERT | none | |

---

## Capability disposition

| Area | Disposition |
| --- | --- |
| Shell/commander/processes/pipelines/terminal UI | P/M |
| Architecture canvas semantics | P/M |
| Local git/pkg/process real ops | M |
| Vendor CLIs | handoff/status |
| Approvals/audit concepts | M real |
| Knowledge light | M new |
| Skills UI/helpers | **X** |
| Updater | **P** |
| Cloud/web/node/org | **D** |

---

## Temporary branch only

| Item | Must remove before done |
| --- | --- |
| React tree after cutover | Yes |
| TS core imports | Yes |
| Dual entrypoints | Yes |
| compat/** shims | Yes |

---

## Removal checklist

- [ ] No mock production adapter  
- [ ] No UI-only ops success  
- [ ] No @polterware/core from apps  
- [ ] No React on desktop product  
- [ ] No skills product IPC/UI  
- [ ] No legacy/tui imports  
- [ ] No dual-format conf readers  
- [ ] Single Solid entrypoint  
