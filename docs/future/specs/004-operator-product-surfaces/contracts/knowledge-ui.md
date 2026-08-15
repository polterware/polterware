# Contract: Knowledge UI

**Owner**: `packages/features/knowledge`  
**Consumers**: `App.tsx`

## Tabs

`docs` | `decisions` | `runbooks`

Operational memory only. Spec Kit–shaped standards live on the separate **Spec Kit** destination (`packages/features/spec-kit`, section `speckit`) — see `spec-kit-ui.md`.

Legacy route `knowledge/specs` (and Spec Kit kind tabs under knowledge) **MUST** redirect to `speckit/*`.

## Shared document model

All three tabs share one **document canvas** inside the global `DetailPanel`:

- List discovery: `DataTable` (essential columns only; no Actions column)
- Detail: `DetailPanel` with `Body layout="document"`
  - Title
  - Fixed **properties block** under the title (Notion-like label/value rows: status, owner/author, timestamps, kind-specific fields). Status uses filled `StatusChip` only.
  - **Markdown-backed WYSIWYG** body (`MarkdownEditor` from design-system)
- Maximize the panel for primary writing surface (Notion-like column measure)
- Create via light dialog (title + template); body edited in panel after create
- No permanent split editor; no second entity-detail page

Ownership remains metadata on documents/components — not a Knowledge tab.

## Docs

- List: title, status, author, updated  
- Properties: status (`draft` | `published`), owner, author, updated, environment, tags  
- Body: freeform Markdown  
- Actions: Archive

## Decisions

- List: title, status, author, date; status filters  
- Properties: status (`proposed` | `accepted` | `deprecated` | `superseded`), author, date, id  
- Body: ADR template Markdown (Context / Decision / Alternatives / Consequences)  
- Actions: Accept (proposed), Deprecate (accepted)

## Runbooks

- List: title, service, severity, last validated  
- Properties: severity, service, owner, last validated  
- Body: Markdown template (Objective / Prerequisites / Steps / Commands / Rollback)  
- Actions: Run (mock), Duplicate, Open terminal

## Create flows

Create document/decision/runbook via dialog or header action; document body edited in DetailPanel. No permanent split editor.

## Storage

- UI SSOT for body: **Markdown string** (`bodyMarkdown`)
- Session-local mock overrides until backend document persistence
- Architecture-attached light notes remain separate (`KnowledgeItem` proto) — not a full wiki
