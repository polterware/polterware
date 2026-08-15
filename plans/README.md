# Animation implementation plans

These plans turn the selected Polterware Desktop motion opportunities into bounded implementation tasks. They are stamped against commit `497766a` and intentionally reuse the current `--ease-standard`, `--ease-enter`, `--duration-fast`, and `--duration-control` tokens.

## Plans

| # | Plan | Severity | Status | Dependencies |
| --- | --- | --- | --- | --- |
| 001 | [Shared presence motion and selection toolbar](001-shared-presence-selection-toolbar.md) | MEDIUM | TODO | None |
| 002 | [Canvas dropdown spatial motion](002-canvas-dropdown-spatial-motion.md) | MEDIUM | TODO | None |
| 003 | [Notification bulk exit](003-notification-bulk-exit.md) | MEDIUM | TODO | 001 |
| 004 | [Integration method swap](004-integration-method-swap.md) | LOW | TODO | 001 |
| 005 | [Inline validation presence](005-inline-validation-presence.md) | LOW | TODO | 001 |
| 006 | [Interruptible DetailPanel settlement](006-interruptible-detail-panel-settlement.md) | HIGH | DONE | None |
| 007 | [Faster collapsed DetailPanel hover](007-speed-up-detail-panel-hover.md) | MEDIUM | DONE | 006 |
| 008 | [Transform-based DetailPanel settlement](008-transform-detail-panel-settlement.md) | HIGH | DONE | None |
| 009 | [Coordinated collapsed DetailPanel chrome](009-coordinate-detail-panel-chrome.md) | MEDIUM | DONE | 008 |
| 010 | [Grouped DetailPanel action tooltips](010-group-detail-panel-tooltips.md) | MEDIUM | DONE | 009 |

## Recommended execution order

1. Execute 001 first. It establishes the Ark-backed `MotionPresence` API, centralized presence CSS, opacity-policy coverage, and the highest-leverage consumer.
2. Execute 002 next. It is independent from 001 and moves the canvas toolbar menus to the existing Ark-backed dropdown behavior and motion.
3. Execute 003 after 001. It has the most domain-state coordination of the dependent consumers and should prove the shared exit lifecycle before simpler swaps use it.
4. Execute 004 after 001 and 003. It reuses `motion="swap"` for a rare form-state transition.
5. Execute 005 last. It reuses `motion="alert"` and is the smallest consumer.
6. Execute 006 before 007. It makes every DetailPanel transition start from one live size and establishes the settlement lifecycle used to distinguish structural motion from hover response.
7. Execute 007 after 006. It reuses that lifecycle and changes only the high-frequency collapsed hover/focus timing.
8. Execute 008 against the current button-only DetailPanel. It replaces the obsolete layout-height trajectory left after plans 006–007 with a compositor transform and a new viewport lifecycle.
9. Execute 009 after 008. It uses the new settlement lifecycle to keep title geometry stable and coordinate collapsed chrome/hover.
10. Execute 010 last. It is behaviorally independent but touches the same DetailPanel header and should land after its markup settles.

Plans 003–005 may be implemented independently after 001, but execute and review one plan at a time so any change to the shared presence API is reconciled before the next consumer.

## Execution handoff

Use the named skill with one plan at a time:

```text
improve-animations execute plans/001-shared-presence-selection-toolbar.md
improve-animations execute plans/002-canvas-dropdown-spatial-motion.md
improve-animations execute plans/003-notification-bulk-exit.md
improve-animations execute plans/004-integration-method-swap.md
improve-animations execute plans/005-inline-validation-presence.md
improve-animations execute plans/006-interruptible-detail-panel-settlement.md
improve-animations execute plans/007-speed-up-detail-panel-hover.md
improve-animations execute plans/008-transform-detail-panel-settlement.md
improve-animations execute plans/009-coordinate-detail-panel-chrome.md
improve-animations execute plans/010-group-detail-panel-tooltips.md
```

Each executor must respect the plan's commit stamp and stop on material source drift instead of improvising. Runtime feel checks require Erick to explicitly authorize launching the Desktop app; static gates and focused tests remain required even when that authorization is absent.
