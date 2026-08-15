# UI Contract: Knowledge Document Details

**Feature**: 034-knowledge-document-modules  
**Surface**: Knowledge → Docs | Decisions | Runbooks → global DetailPanel

## Host

- Single global `DetailPanel` hosted by shell `MainSurface`.
- Features contribute `DetailPanel.Content` only.
- Collection scopes: `knowledge:docs`, `knowledge:decisions`, `knowledge:runbooks`.
- Entity kinds: `document` | `decision` | `runbook`.

## Layout

```text
DetailPanel.Content
├── Header (domain icon + title + kind actions)
└── Body layout="document"
    └── properties row (fixed)
    └── MarkdownEditor (body; toolbar iff panel mode === maximized)
```

## Properties by kind

### Doc

- Status → StatusChip
- Owner, Author, Updated, Environment
- Tags (optional badges)
- Components (optional clickable badges via host)

### Decision

- Status → StatusChip
- Author, Date, ID
- Alternatives / Consequences summary (from structured fields)
- Components (optional)

### Runbook

- Severity → StatusChip
- Service, Owner, Last validated
- Steps summary, Commands summary (counts; not live runner)
- Component / Operation (optional links)

## States

| State | Behavior |
| --- | --- |
| ready + entity | Full kind detail |
| active target missing entity | Honest not-found header + message |
| collection empty | Existing PageEmpty (no detail fabrication) |
| panel mode half/open | No Markdown format toolbar |
| panel mode maximized | Markdown format toolbar visible |

## Forbidden

- Second entity page / drawer for knowledge docs
- Spec Kit kinds in this surface
- Claiming live Core persist or live runbook execution
- Actions column on Knowledge tables

## Modules

- Orchestration: `packages/features/knowledge/src/index.tsx`
- Shell: shared document layout module
- Kind details: document / decision / runbook modules
- Helpers: status/severity tones + property builders
