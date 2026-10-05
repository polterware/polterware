# W3-POLISH — design, motion, accessibility and copy

- Result: **partial**. 32/32 audit surfaces recorded; visual/label/copy changes complete within this workstream's ownership. Full accessibility, theme/motion and integrated-state acceptance remain blocked by the requests below. No G3 approval is claimed.
- Branch: `sparky/w3-polish`, based on `sparky-next` at `0a5dcfc`. Implementation commits: `fa311cd`, `4eb96cd`, `415f21f`, `bdb9354`, `a54b0c0`, `0b2f42f`, `610be04`, `14a6612`.
- Inputs read: AGENT-PROTOCOL.md, 00-decisions.md, 06-ui.md, root DESIGN.md, 10-wave3-and-verification.md; Wave 2 shell, glue, wire, memories, minds, attachments, schedule, calendar, focus, review, map, search and settings status files, including follow-up variants.

## Changes

Shared Button radii now use the control token; a shared opaque focus outline covers native controls and Ark focus states without changing component APIs. No palette/root configuration or contract edits. Review summary uses spacing instead of sibling dividers, names metrics/charts for assistive technology, and separates numbers from character art. Focus announces phase changes without announcing each countdown tick. Calendar uses shared sampled spring tokens, full date/occurrence names, compact named slot buttons and aligned all-day lanes. Map place rows and Mind tiles use shared Buttons and fit narrow windows. Settings has independently scrollable rail/content, wrapping descriptions and named tables. Commander truncates long results with title access. Shell mounts the existing polite toast region. English copy accurately describes unavailable login items, updates, reset and link-title fetching.

## Screenshot evidence and reproduction

196 PNGs (~10.2 MiB total; each below 200 KiB) are in `w3-polish/`: 93 before and 93 after captures at **800×560**, **1280×800**, **1920×1080**, plus 10 targeted after captures. Browser is headless Chromium with the real Solid renderer/features and a fake `window.sparky` bridge. No personal data or real permission/network action is used. All 93 final standard captures report no body overflow. Screenshot capture disables animations; separate reduced-motion evidence inspects computed styles. This is not native Electron acceptance.

Reproduce from repository root with `pnpm exec vite --config plans/sparky-next/status/w3-polish/audit-harness/vite.config.mjs`; then use Playwright CLI `run-code --filename .../capture.cjs` (set `phase` to `before` only on the baseline). The browser-only harness imports existing fixtures and intentionally stubs their `vi.fn`. `preview=list` disables virtualization only in the harness; `preview=map-settings` removes the conflicting registration only in the harness. Those images are **component previews**, never proof of integrated behavior. Fake reminder counts and Live chips do not prove notifier operation. Shared gallery was also inspected at 127.0.0.1:5175: 41 sections, visible 2px focus outline; only console error was a missing favicon.

Evidence: [capture report](w3-polish/capture-evidence.json), [keyboard/preferences report](w3-polish/accessibility-evidence.json), [gallery report](w3-polish/gallery-evidence.json), [shared gallery](w3-polish/after-shared-gallery-1280.png). Integrated virtualized Today/Inbox currently show an empty tbody despite the counter; see request 5 and [explicit populated component preview](w3-polish/after-list-component-preview-800.png).

## Per-screen checklist

Columns: spacing, radius roles, states, keyboard/focus, names/roles, contrast, copy. **V** = reviewed through source, package tests and/or browser evidence (not exhaustive assistive-technology certification); **H** = hand/native verification remains; **B<n>** = blocked by request n. **B2** applies throughout: dark muted text fails AA on two backgrounds and the light theme is not implemented. Shared empty/loading/error/offline layouts are captured below; feature-specific state paths are reviewed in source/tests rather than injected for every screen. No checklist row claims every interaction passed.

Each screenshot cell links before/after at 800, 1280 and 1920 respectively. The separate Map settings panel cannot be reached in the baseline or integrated after build, so only an explicit 800px component preview is provided.

| Surface | Spacing | Radii | States | Keyboard | Labels/roles | Contrast | Copy | Before / after screenshots |
|---|---|---|---|---|---|---|---|---|
| Today | V | V | B5 | B8 | H | B2 | V | [800 before](w3-polish/before-today-800.png) / [after](w3-polish/after-today-800.png) · [1280 before](w3-polish/before-today-1280.png) / [after](w3-polish/after-today-1280.png) · [1920 before](w3-polish/before-today-1920.png) / [after](w3-polish/after-today-1920.png) |
| Inbox / Memories list | V | V | B5 | B8 | H | B2 | V | [800 before](w3-polish/before-inbox-800.png) / [after](w3-polish/after-inbox-800.png) · [1280 before](w3-polish/before-inbox-1280.png) / [after](w3-polish/after-inbox-1280.png) · [1920 before](w3-polish/before-inbox-1920.png) / [after](w3-polish/after-inbox-1920.png) |
| Minds | V | V | V | B3/B8 | H | B2 | V | [800 before](w3-polish/before-minds-800.png) / [after](w3-polish/after-minds-800.png) · [1280 before](w3-polish/before-minds-1280.png) / [after](w3-polish/after-minds-1280.png) · [1920 before](w3-polish/before-minds-1920.png) / [after](w3-polish/after-minds-1920.png) |
| Review | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-review-800.png) / [after](w3-polish/after-review-800.png) · [1280 before](w3-polish/before-review-1280.png) / [after](w3-polish/after-review-1280.png) · [1920 before](w3-polish/before-review-1920.png) / [after](w3-polish/after-review-1920.png) |
| Map | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-map-800.png) / [after](w3-polish/after-map-800.png) · [1280 before](w3-polish/before-map-1280.png) / [after](w3-polish/after-map-1280.png) · [1920 before](w3-polish/before-map-1920.png) / [after](w3-polish/after-map-1920.png) |
| Focus | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-focus-800.png) / [after](w3-polish/after-focus-800.png) · [1280 before](w3-polish/before-focus-1280.png) / [after](w3-polish/after-focus-1280.png) · [1920 before](w3-polish/before-focus-1920.png) / [after](w3-polish/after-focus-1920.png) |
| Search | V | V | V | B8 | V | B2 | V | [800 before](w3-polish/before-search-800.png) / [after](w3-polish/after-search-800.png) · [1280 before](w3-polish/before-search-1280.png) / [after](w3-polish/after-search-1280.png) · [1920 before](w3-polish/before-search-1920.png) / [after](w3-polish/after-search-1920.png) |
| Calendar — Month | V | V | V | B1 | V | B2 | V | [800 before](w3-polish/before-calendar-month-800.png) / [after](w3-polish/after-calendar-month-800.png) · [1280 before](w3-polish/before-calendar-month-1280.png) / [after](w3-polish/after-calendar-month-1280.png) · [1920 before](w3-polish/before-calendar-month-1920.png) / [after](w3-polish/after-calendar-month-1920.png) |
| Calendar — Week | V | V | V | B1 | V | B2 | V | [800 before](w3-polish/before-calendar-week-800.png) / [after](w3-polish/after-calendar-week-800.png) · [1280 before](w3-polish/before-calendar-week-1280.png) / [after](w3-polish/after-calendar-week-1280.png) · [1920 before](w3-polish/before-calendar-week-1920.png) / [after](w3-polish/after-calendar-week-1920.png) |
| Calendar — Day | V | V | V | B1 | V | B2 | V | [800 before](w3-polish/before-calendar-day-800.png) / [after](w3-polish/after-calendar-day-800.png) · [1280 before](w3-polish/before-calendar-day-1280.png) / [after](w3-polish/after-calendar-day-1280.png) · [1920 before](w3-polish/before-calendar-day-1920.png) / [after](w3-polish/after-calendar-day-1920.png) |
| Memory detail | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-memory-detail-800.png) / [after](w3-polish/after-memory-detail-800.png) · [1280 before](w3-polish/before-memory-detail-1280.png) / [after](w3-polish/after-memory-detail-1280.png) · [1920 before](w3-polish/before-memory-detail-1920.png) / [after](w3-polish/after-memory-detail-1920.png) |
| Schedule editor | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-schedule-800.png) / [after](w3-polish/after-schedule-800.png) · [1280 before](w3-polish/before-schedule-1280.png) / [after](w3-polish/after-schedule-1280.png) · [1920 before](w3-polish/before-schedule-1920.png) / [after](w3-polish/after-schedule-1920.png) |
| Commander | V | V | V | B8 | V | B2 | V | [800 before](w3-polish/before-commander-800.png) / [after](w3-polish/after-commander-800.png) · [1280 before](w3-polish/before-commander-1280.png) / [after](w3-polish/after-commander-1280.png) · [1920 before](w3-polish/before-commander-1920.png) / [after](w3-polish/after-commander-1920.png) |
| Settings — General | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-settings-general-800.png) / [after](w3-polish/after-settings-general-800.png) · [1280 before](w3-polish/before-settings-general-1280.png) / [after](w3-polish/after-settings-general-1280.png) · [1920 before](w3-polish/before-settings-general-1920.png) / [after](w3-polish/after-settings-general-1920.png) |
| Settings — Notifications | V | V | H | H | V | B2 | V | [800 before](w3-polish/before-settings-notifications-800.png) / [after](w3-polish/after-settings-notifications-800.png) · [1280 before](w3-polish/before-settings-notifications-1280.png) / [after](w3-polish/after-settings-notifications-1280.png) · [1920 before](w3-polish/before-settings-notifications-1920.png) / [after](w3-polish/after-settings-notifications-1920.png) |
| Settings — Focus | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-settings-focus-800.png) / [after](w3-polish/after-settings-focus-800.png) · [1280 before](w3-polish/before-settings-focus-1280.png) / [after](w3-polish/after-settings-focus-1280.png) · [1920 before](w3-polish/before-settings-focus-1920.png) / [after](w3-polish/after-settings-focus-1920.png) |
| Settings — Calendar | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-settings-calendar-800.png) / [after](w3-polish/after-settings-calendar-800.png) · [1280 before](w3-polish/before-settings-calendar-1280.png) / [after](w3-polish/after-settings-calendar-1280.png) · [1920 before](w3-polish/before-settings-calendar-1920.png) / [after](w3-polish/after-settings-calendar-1920.png) |
| Settings — Tiles (integrated Map selection) | V | V | V | B4 | V | B2 | V | [800 before](w3-polish/before-settings-map-800.png) / [after](w3-polish/after-settings-map-800.png) · [1280 before](w3-polish/before-settings-map-1280.png) / [after](w3-polish/after-settings-map-1280.png) · [1920 before](w3-polish/before-settings-map-1920.png) / [after](w3-polish/after-settings-map-1920.png) |
| Settings — Map (component preview) | V | V | B4 | B4 | V | B2 | V | Baseline/integrated: B4; [after preview](w3-polish/after-settings-map-component-preview-800.png) |
| Settings — Search | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-settings-search-800.png) / [after](w3-polish/after-settings-search-800.png) · [1280 before](w3-polish/before-settings-search-1280.png) / [after](w3-polish/after-settings-search-1280.png) · [1920 before](w3-polish/before-settings-search-1920.png) / [after](w3-polish/after-settings-search-1920.png) |
| Settings — Agents / Access | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-settings-access-800.png) / [after](w3-polish/after-settings-access-800.png) · [1280 before](w3-polish/before-settings-access-1280.png) / [after](w3-polish/after-settings-access-1280.png) · [1920 before](w3-polish/before-settings-access-1920.png) / [after](w3-polish/after-settings-access-1920.png) |
| Settings — Agents / Activity | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-settings-activity-800.png) / [after](w3-polish/after-settings-activity-800.png) · [1280 before](w3-polish/before-settings-activity-1280.png) / [after](w3-polish/after-settings-activity-1280.png) · [1920 before](w3-polish/before-settings-activity-1920.png) / [after](w3-polish/after-settings-activity-1920.png) |
| Settings — Data | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-settings-data-800.png) / [after](w3-polish/after-settings-data-800.png) · [1280 before](w3-polish/before-settings-data-1280.png) / [after](w3-polish/after-settings-data-1280.png) · [1920 before](w3-polish/before-settings-data-1920.png) / [after](w3-polish/after-settings-data-1920.png) |
| Settings — Shortcuts | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-settings-shortcuts-800.png) / [after](w3-polish/after-settings-shortcuts-800.png) · [1280 before](w3-polish/before-settings-shortcuts-1280.png) / [after](w3-polish/after-settings-shortcuts-1280.png) · [1920 before](w3-polish/before-settings-shortcuts-1920.png) / [after](w3-polish/after-settings-shortcuts-1920.png) |
| Settings — Advanced | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-settings-advanced-800.png) / [after](w3-polish/after-settings-advanced-800.png) · [1280 before](w3-polish/before-settings-advanced-1280.png) / [after](w3-polish/after-settings-advanced-1280.png) · [1920 before](w3-polish/before-settings-advanced-1920.png) / [after](w3-polish/after-settings-advanced-1920.png) |
| Onboarding — Welcome | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-onboarding-1-800.png) / [after](w3-polish/after-onboarding-1-800.png) · [1280 before](w3-polish/before-onboarding-1-1280.png) / [after](w3-polish/after-onboarding-1-1280.png) · [1920 before](w3-polish/before-onboarding-1-1920.png) / [after](w3-polish/after-onboarding-1-1920.png) |
| Onboarding — Notifications | V | V | H | H | V | B2 | V | [800 before](w3-polish/before-onboarding-2-800.png) / [after](w3-polish/after-onboarding-2-800.png) · [1280 before](w3-polish/before-onboarding-2-1280.png) / [after](w3-polish/after-onboarding-2-1280.png) · [1920 before](w3-polish/before-onboarding-2-1920.png) / [after](w3-polish/after-onboarding-2-1920.png) |
| Onboarding — Agents | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-onboarding-3-800.png) / [after](w3-polish/after-onboarding-3-800.png) · [1280 before](w3-polish/before-onboarding-3-1280.png) / [after](w3-polish/after-onboarding-3-1280.png) · [1920 before](w3-polish/before-onboarding-3-1920.png) / [after](w3-polish/after-onboarding-3-1920.png) |
| Empty state | V | V | V | B8 | V | B2 | V | [800 before](w3-polish/before-state-empty-800.png) / [after](w3-polish/after-state-empty-800.png) · [1280 before](w3-polish/before-state-empty-1280.png) / [after](w3-polish/after-state-empty-1280.png) · [1920 before](w3-polish/before-state-empty-1920.png) / [after](w3-polish/after-state-empty-1920.png) |
| Loading state | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-state-loading-800.png) / [after](w3-polish/after-state-loading-800.png) · [1280 before](w3-polish/before-state-loading-1280.png) / [after](w3-polish/after-state-loading-1280.png) · [1920 before](w3-polish/before-state-loading-1920.png) / [after](w3-polish/after-state-loading-1920.png) |
| Error state | V | V | B5 | H | V | B2 | V | [800 before](w3-polish/before-state-error-800.png) / [after](w3-polish/after-state-error-800.png) · [1280 before](w3-polish/before-state-error-1280.png) / [after](w3-polish/after-state-error-1280.png) · [1920 before](w3-polish/before-state-error-1920.png) / [after](w3-polish/after-state-error-1920.png) |
| Offline state | V | V | V | H | V | B2 | V | [800 before](w3-polish/before-state-offline-800.png) / [after](w3-polish/after-state-offline-800.png) · [1280 before](w3-polish/before-state-offline-1280.png) / [after](w3-polish/after-state-offline-1280.png) · [1920 before](w3-polish/before-state-offline-1920.png) / [after](w3-polish/after-state-offline-1920.png) |

## Keyboard, motion, transparency, truncation and states

Commander and Settings traps were probed with 14 and 24 Tab presses; after dialog settlement, focus stayed inside and Escape restored the opener. Their native Enter triggers fail on Today because a collection shortcut consumes Enter (request 8). Search input has a visible 2px opaque ring. Calendar grid arrow/focus semantics remain incomplete (request 1); adding grid roles alone would conceal the defect. Existing Ark tabs/radios/dialog semantics are retained. Map online navigation remains a native/network manual check.

The OS Reduce motion probe reports Calendar transform `none` and transition duration `1e-05s`. Existing shared motion guards remain. App Reduce transparency is represented by the actual `data-shell-chrome="solid"` attribute: titlebar background is opaque rgb(7,7,7), [capture](w3-polish/after-solid-shell-800.png). Native preference propagation was not launched/tested. There is no app Reduce motion field in the frozen contract; request 2 covers it. Selecting an OS light preference still yields the dark rgb(7,7,7) shell, [capture](w3-polish/after-light-preference-800.png). Muted text is ~3.8:1 on surface and ~3.1:1 on selected; shared root palette correction requires its owner. Focus ring uses text-secondary (~6.3:1 on selected).

Long title detail: [800px](w3-polish/after-long-title-detail-800.png). Commander ellipsis exposes a title; Mind tiles wrap; calendar occurrence names contain the full title even where visual text truncates. RTL basics use logical shell/feature properties; [RTL Map](w3-polish/after-rtl-map-800.png) has no body overflow. This is basic structural coverage, not complete RTL localization. Window minimum and large/fullscreen browser sizes are covered; native fullscreen/vibrancy/multi-monitor behavior remains manual.

PageLoading delegates to the existing opaque PageSkeleton with one polite busy status and decorative bones hidden. ToastRegion has `aria-live="polite"`, UndoNotice has role status/polite and the existing 8-second undo text, and StatusChip has role status. Focus timer has aria-live off while Running/Paused phase has polite atomic status; [running capture](w3-polish/after-focus-running-800.png). Real toast/undo/reconnect announcement timing needs VoiceOver. No unavailable login item, data reset or update is advertised as working. No new placeholder/TODO UI was introduced.

## VoiceOver notes — manual, not executed

VoiceOver is unavailable in the headless browser. On a native macOS build, run each note below with VO keys and ordinary Tab, Shift+Tab, arrows, Space, Enter and Escape. Check name, role, value, ordering, interruption frequency, focus restoration and clipping at the minimum window size. Repeat with Reduce motion/transparency and both themes once request 2 is resolved.

| Surface | What to verify by hand |
|---|---|
| Today | Verify Today heading, filter names, row action order and full long titles. |
| Inbox / Memories list | Read table headers and selection state; activate row, complete, undo, move and delete. Confirm rows exist in packaged app. |
| Minds | Traverse tree levels with arrows; hear expanded, selected and parent names. Reach create, rename, move and delete. |
| Review | Read two metric names and percentages once; traverse activity bars and heatmap with arrows; verify date totals and period tables. |
| Map | With tiles off, read stored places and instructions. With explicit online consent, verify canvas name, zoom buttons and keyboard pan without a mouse. |
| Focus | Start, pause, resume and cancel. Phase changes should announce once; countdown must not interrupt every second. Inspect recipe menu. |
| Search | Hear search field, result counts and full title/snippet. Reach filters and Save filters, activate a result, verify no duplicate announcements. |
| Calendar — Month | Read full date and occurrence names, current date and selected day. Confirm actual focus moves with arrows and radios remain operable. |
| Calendar — Week | Check day/time relationship, all-day lane and completed/overdue names. Arrow through hours without tabbing through 168 slots. |
| Calendar — Day | Read date and time for Add memory slots; activate occurrence and return focus to its original slot. |
| Memory detail | Read Title and Note labels, checklist states, attachment controls, place status and editor errors. Verify a long title is editable and readable. |
| Schedule editor | Traverse repeat, date/time, timezone and reminder controls; read validation errors and Save schedule result; use date-picker arrows. |
| Commander | Open with Cmd+K and native button activation; hear dialog title and search label; Tab remains inside, Escape restores the opener. |
| Settings — General | Read version, disabled Updates and background switch description; confirm launch at login is not promised. |
| Settings — Notifications | Verify denied, not-determined and authorized permission text; distinguish planned and armed reminders. Use permission action only deliberately. |
| Settings — Focus | Read each duration spinbutton, unit and bounds; verify Sound and completion preference switch state. |
| Settings — Calendar | Read week-start and default-view selections. Arrow within controls without changing a background Calendar. |
| Settings — Tiles (integrated Map selection) | Read consent/provider status and Keep tiles off. Confirm selecting App > Map does not incorrectly land in Tiles after request 4. |
| Settings — Map (component preview) | Read provider selector and Address search checkbox description. Verify both controls become reachable in the integrated dialog. |
| Settings — Search | Read search preferences and unavailable link-title fetch explanation. Check every checkbox name/state. |
| Settings — Agents / Access | Read agent access mode and disabled/unavailable controls; inspect destructive confirmations before submitting. |
| Settings — Agents / Activity | Read Activity table label, headers, empty state and audit entries in order; do not imply activity data from this fake fixture is live backend evidence. |
| Settings — Data | Read export, backup and restore descriptions; Type RESET must explicitly say no data changes. Verify disabled actions cannot activate. |
| Settings — Shortcuts | Read Keyboard shortcuts table name, command and binding columns; verify shortcut keys are pronounced understandably. |
| Settings — Advanced | Read appearance and Reduce transparency controls; verify light theme behavior after request 2 and native restart/reload descriptions. |
| Onboarding — Welcome | Hear dialog title, step text and Skip/Continue. Tab remains within onboarding and focus moves sensibly between steps. |
| Onboarding — Notifications | Verify permission prompt is user-initiated; read armed status and unavailable/denied alternatives honestly. |
| Onboarding — Agents | Hear Settings > Agents > Access instructions and finish action. Ensure returning to the shell has a sensible initial focus. |
| Empty state | Read empty heading and New memory action; verify state is not announced repeatedly when filters change. |
| Loading state | Hear one Loading memories status; decorative skeleton bones must be hidden; focus must not disappear when data arrives. |
| Error state | Hear query error and retry action. Reject sidebar count calls too and confirm shell survives after request 5. |
| Offline state | Hear Core offline once; reach Retry connection; verify reconnect preserves route/focus and does not claim reminders are armed. |

## Validation

Final checks after the last implementation change:

| Gate | Result |
|---|---|
| DS and all 11 Sparky packages `test` | PASS: 617 assertions total; Review rerun after final art/value CSS: 70 passed. |
| DS and all Sparky packages `typecheck` | PASS (12 packages). |
| Whole-repo `pnpm typecheck` | PASS: 27 tasks. |
| `pnpm design:lint` | PASS: ui opacity check OK. |
| `pnpm test:ui-opacity` | PASS: 10 tests. |
| `git diff --check` | PASS. |
| Affected Polterware features | Shell 7, knowledge 21, settings 26 passed; architecture 152 passed / 5 failed in unchanged source-text assertions. |
| Polterware Desktop | 136 passed / 2 existing source assertions failed; 2 IPC suites blocked by incomplete Electron installation. 28/31 files passed. Request 7. |
| VoiceOver / native Electron / online Map / light theme | NOT VERIFIED; light theme/app motion unavailable (request 2). |

Polterware source assertions were checked against baseline `0a5dcfc`: table.css is byte-identical; the baseline Sparky list already contains SelectionCell and SelectionToolbar. Architecture source/tests were untouched. No test assertions were weakened to make those gates green. Existing Settings copy expectations were updated only for the changed, more accurate product wording.

## Requests and follow-ups

- [Request 1: Calendar roving focus, grid model and control arrow composition](../requests/w3-polish-1.md).
- [Request 2: Light/dark contrast, applied theme and app Reduce motion](../requests/w3-polish-2.md).
- [Request 3: Reachable Mind editor route](../requests/w3-polish-3.md).
- [Request 4: Unique/canonical Map settings registration](../requests/w3-polish-4.md).
- [Request 5: Investigate virtualized list rendering and failed count recovery](../requests/w3-polish-5.md).
- [Request 6: Implement or remove unavailable link-title fetching preference](../requests/w3-polish-6.md).
- [Request 7: Reconcile baseline Polterware gates and Electron installation](../requests/w3-polish-7.md).
- [Request 8: Preserve native Enter/Space activation around collection shortcuts](../requests/w3-polish-8.md).

The protocol requires requests for behavior, frozen contracts, shared root files and outside-owner gate failures. These subtasks stopped at written requests; all independent authorized visual/copy work and evidence continued. No Main/preload, Go, contracts, root shared files, lockfile, other status or Sparky Desktop e2e tests were changed. No rebase, push or tag. Local commits are small and isolated for coordination with W3-SECURITY.

Polterware CLI: `status` and `query cards` fail with “no project for this directory”; the format grammar was read. `apply` of a Work card with summary/rationale and the closing `handoff` also fail because this worktree is not registered. No arbitrary project/database was initialized. This file is the reviewable fallback; the integrator can apply the [prepared Work card](w3-polish/work-card.pw.md) in the registered project.

## Erick's visual acceptance

- Compare compact 800px Calendar slots/all-day alignment, Map stacked canvas/place list, Settings rail/content scrolling and titlebar density against large windows.
- Judge Review character art, adjacent counts and two-column summary, long title truncation/wrapping, and the shared Button radius change in both Sparky and Polterware gallery.
- Inspect visible focus rings across selected/hovered controls after keyboard owners resolve requests 1/8; approve light/dark contrast after request 2.
- Native macOS: judge vibrancy vs Reduce transparency, OS/app reduced motion, VoiceOver notes, real notification state, real Map controls and error/reconnect timing. Full W3-POLISH acceptance remains pending those checks and eight requests.
