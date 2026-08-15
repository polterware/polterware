# Research: Project Settings Environments

**Date**: 2026-08-12  
**Feature**: 030-project-settings-environments

## Current state

- `packages/features/settings/src/index.tsx` (~244 lines): three tabs already (General · Environments · Advanced).
- General uses bare `<label>` + Input/Textarea/Select — no Field, no save toast.
- Environments uses DataTable but protection is plain text; no kind column/chip; no nodes column consistency; no row activation → DetailPanel.
- Danger zone appears on General (disabled) and Advanced (delete logs disabled) — no Archive confirm Dialog.
- No tests, no package vitest, no frontend-state dependency.
- Shell `ENVIRONMENT_FIXTURES` are breadcrumb-grade (`id`, `name`, `kind`, `projectId`) without protection/rules/counts.

## Decisions

### D1 — Package-local environment fixtures

**Decision**: Define `SETTINGS_ENVIRONMENT_MOCKS` in `feature-settings` with protection, counts, rules, default, `dataSource: "fixture"`.  
**Rationale**: Shell fixtures serve breadcrumb switcher; enriching them would risk parallel-agent conflicts and mix IA concerns.  
**Alternatives**: Extend shell EnvironmentItem — rejected for boundary/parallelism.

### D2 — StatusChip for kind and protection

**Decision**: Both columns use StatusChip with mapped tones (prod→danger/warn family, staging→warn, dev→muted/healthy; Strict→danger, Standard→warn, Relaxed→muted).  
**Rationale**: AGENTS.md LAW — status/kind/protection everywhere must use StatusChip.  
**Alternatives**: Badge or plain text — rejected.

### D3 — DetailPanel for environment

**Decision**: `EnvironmentDetail` with sections layout; registerCollection scope `settings:environments`.  
**Rationale**: Matches all other operational collections.  
**Alternatives**: Inline expand row — rejected by DetailPanel LAW.

### D4 — Advanced owns danger zone Archive

**Decision**: Move destructive Archive mock exclusively under Advanced with A11yDialog confirm; remove misleading Delete-as-enabled claims. General keeps identity form only.  
**Rationale**: User requirement + cleaner General form.  
**Alternatives**: Keep danger on both tabs — rejected as duplicated chrome.

### D5 — Tab IA freeze

**Decision**: Tabs remain General · Environments · Advanced; legacy tab ids normalize to general without rendering Access/Integrations.  
**Rationale**: Product IA (docs/spec §8.12 + AGENTS hierarchy).  

## Risks

- Parallel agents: do not touch org-team, integrations, architecture, operations, terminal.
- Browser verification: Agents.md forbids `pnpm dev` unless asked — verify via vitest contracts.
