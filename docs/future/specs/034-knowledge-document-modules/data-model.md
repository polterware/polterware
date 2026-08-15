# Data Model: Knowledge Document Detail Modules

**Date**: 2026-08-12 · **Feature**: 034-knowledge-document-modules

Presentation projection only. Source of truth for fixtures: `@polterware/feature-shell` workspace mocks.

## Entities

### WorkspaceDocument (Doc)

| Field | Role in detail |
| --- | --- |
| id, title | Identity / header |
| bodyMarkdown | Document canvas SSOT |
| status | StatusChip (draft / published) |
| owner, author, updatedAt, environment | Properties |
| tags | Quiet badges when present |
| componentIds / componentNames | Linked component badges |

### WorkspaceDecision

| Field | Role in detail |
| --- | --- |
| id, title | Identity / header |
| status | StatusChip (proposed / accepted / deprecated / superseded) |
| author, date | Properties |
| alternatives[], consequences[] | Summary props + body sections via existing `adrMarkdown` |
| bodyMarkdown | Document canvas SSOT |
| componentIds / componentNames | Linked component badges |
| supersedesId, relatedIds | Optional quiet links when present |

### WorkspaceRunbook

| Field | Role in detail |
| --- | --- |
| id, title | Identity / header |
| severity | StatusChip (low / medium / high / critical) |
| service, owner, lastValidated | Properties |
| steps[], commands[] | Summary counts in properties; full lists in body |
| bodyMarkdown | Document canvas SSOT |
| componentId/Name, operationId, resource* | Optional relation badges |

## Session state (Knowledge view)

| State | Purpose |
| --- | --- |
| bodyOverrides[id] | Session-local Markdown edits (mock) |
| archivedIds | Mock archive filter for docs |
| detailPanel activeTarget | Collection selection |

## No new persistence

No SQLite/Core schema changes. No dual write path.
