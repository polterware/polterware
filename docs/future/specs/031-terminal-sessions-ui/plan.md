# Implementation Plan: Terminal Sessions UI (Mock)

**Branch**: `031-terminal-sessions-ui` | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/031-terminal-sessions-ui/spec.md`

## Summary

Elevate Terminal page, side panel, and target picker in `@polterware/feature-shell` to professional mock quality: session StatusChip (active/idle/closed), target health StatusChip, blocked unusable targets with feedback, mock new/close, correlation/session id mono audit chrome, empty states, and pure helper tests. No SSH/PTY/main-process runtime.

## Technical Context

**Language/Version**: TypeScript 5.x · SolidJS 1.9  

**Primary Dependencies**: `@polterware/frontend-design-system` (StatusChip, Button, PageToolbar, Tabs, Popover, MenuItem, Drawer, pushToast)  

**Storage**: N/A (in-memory session signals + fixtures)  

**Testing**: Vitest (node) for pure helpers; package `src/**/*.{test,spec}.ts`  

**Target Platform**: Desktop Solid renderer (fixture/mock only)  

**Project Type**: Feature package operator UI  

**Performance Goals**: Instant local mock interactions  

**Constraints**: Design-system StatusChip for labeled status; no real SSH/PTY/Node spawn; mock-honest copy; shell package only  

**Scale/Scope**: 3 UI modules + session helpers + fixtures + tests  

## Constitution Check

- **I. Operational product**: Strengthens safe execution UI honesty for terminal — pass.
- **II. Domain before framework**: Session/target rules in pure helpers — pass.
- **III. Runtime separation**: Renderer only; no live runtime claim — pass.
- **IV. Monorepo deps**: shell → design-system; no cycles — pass.
- **V. Contracts-first**: Local UI contract note; no new IPC — pass.
- **VI. Security**: No unrestricted renderer OS; secrets not shown — pass.
- **VII. Auditable mutations**: correlationId + session id visible — pass.
- **VIII. Local-first**: Fixture authority; offline targets explicit — pass.
- **IX–XI**: Small lists; correlation ids required — pass.
- **XII. Tests**: Helper + UI contract tests — pass.
- **XIII–XIV**: Extend existing shell surfaces; single mock model — pass.

## Project Structure

### Documentation (this feature)

```text
specs/031-terminal-sessions-ui/
├── spec.md
├── plan.md
├── tasks.md
├── checklists/requirements.md
└── contracts/terminal-sessions-ui.md
```

### Source (owned)

```text
packages/features/shell/src/
├── terminal-session-model.ts       # NEW pure helpers
├── terminal-session-model.test.ts  # NEW
├── types.ts                        # target usability (existing) + re-exports if needed
├── types.test.ts                   # keep / expand target usability
├── fixtures.ts                     # add offline/revoked targets for honesty paths
├── workspace-mocks/types.ts        # session shape: status active|idle|closed, shell, startedAt, correlationId
├── workspace-mocks/data.ts         # fixture sessions updated
├── target-picker.tsx               # StatusChip + unusable feedback path
├── terminal-page.tsx               # professional chrome, empty, create/close
└── terminal-side-panel.tsx         # audit ids + StatusChip + mock honesty
```

### Out of bounds

architecture · operations workflows · org-team · integrations · project settings · Electron main/preload · real SSH/PTY

## Implementation Approach

1. **Model first** — `TerminalSessionStatus` = active | idle | closed; helpers for tone/label, create/close mock session, unusable feedback message, dataSource label; expand fixtures with offline/revoked targets; migrate session mocks.
2. **Target picker** — StatusChip per row; unusable rows either disabled with reason or select path that calls feedback without `onSelect` session open; empty “no targets”.
3. **Terminal page** — session tabs + active chrome (StatusChip, target, shell, startedAt, mono ids); create only if usable; close mock; empty no-sessions; fixture label; toast on blocked open.
4. **Side panel** — StatusChip target health; session/correlation mono; never “Connected” for real host; fixture/mock subtitle.
5. **Tests** — session model helpers + expanded target usability / UI contract; optional source-contract asserts for StatusChip / correlation strings.

## Rejection Rubric (definition of done)

- [x] StatusChip on session and/or target health
- [x] Unusable targets blocked with feedback (not silent fail)
- [x] correlation/session id visible in UI mock
- [x] Tests pass
- [x] FORBIDDEN: SSH/PTY real or Node runtime spawn
- [x] FORBIDDEN: claim “already implemented” from thin page

## Complexity Tracking

No constitution violations. Session status vocabulary simplified from legacy connected/connecting/disconnected/error to active/idle/closed for operator grammar; fixtures migrated in place.
