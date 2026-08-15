# Contract: Core Data Adapter (Desktop composition)

**Owner**: `apps/desktop/src/renderer/App.tsx` (composition root)  
**Client**: `@polterware/frontend-data` → Connect local services  

## Required calls (existing)

| Need | Client method |
| --- | --- |
| Health / shell status | `health()` |
| Systems list | `workspace()` |
| Resources list | `listResources(systemId)` |
| Layout get/save | `getLayout` / `updateLayout` |
| Discovery | `listDiscoveryCandidates` (optional on canvas tools) |
| Knowledge for detail | `listKnowledge` when subject known |
| Ops / audit snippets | `listOps` / `listAudit` for activity sections when available |

## Adapter behavior

```text
try core → map to view models (dataSource: "core")
catch / empty → optional fixture (dataSource: "fixture") OR empty state
never paint fixture as "live synced" without label
```

## Privilege

- All network to core uses session token from `window.polterware.session`  
- No Node/fs from renderer  
- Mutations for layout save remain explicit user action; governed ops stay on safe path  

## System id

Composition may continue using a local default system id when workspace returns one system; multi-system switcher is optional polish.
)
