# Wave 2 — features (UI + CLI)

Starts after G1. Each WS owns one `packages/features/sparky-<x>` package (and a few extras). They integrate through the shell
registry, `window.sparky` and `@polterware/sparky-contracts`; they **never** import each other. All UI uses the design system
(see AGENT-PROTOCOL §6 and [06-ui.md](06-ui.md)). Each WS ships: components, stores, tests (vitest + `@solidjs/testing-library`
style tests already used in Polterware), gallery-independent fixtures, and **Playwright-for-Electron** smoke tests for its
main flow in `apps/sparky-desktop/tests/e2e/<feature>.spec.ts` (WS-owned file; W3-E2E owns the runner config).
Merge order: W2-SHELL → W2-MEMORIES → W2-MINDS, W2-SCHEDULE → W2-CALENDAR, W2-ATTACH-UI, W2-FOCUS, W2-REVIEW, W2-MAP,
W2-SEARCH-UI, W2-SETTINGS, W2-CLI. Gate **G2** at the end.

Common Reads: the Swift behavior reference listed per WS (see [11-swift-reference-map.md](11-swift-reference-map.md)) and the
matching Polterware feature package for patterns (`packages/features/knowledge` for lists/detail, `settings` for the dialog).

---

## W2-SHELL — host, navigation, Commander

**Owns.** `packages/features/sparky-shell/**`, `apps/sparky-desktop/src/renderer/**`.
**Do.** Host for the registry; title bar, sidebar (Inbox/Today/Calendar/Minds tree/Map/Focus/Review/Settings), main surface +
global DetailPanel host, router (`route.ts` pattern), history controls, Commander (palette: navigate, quick add with natural
input "Buy milk tomorrow 9am #errands @Home" parsed by a small tested parser → `memories.create`, search, actions registered by
features), reminders/core status chip (`reminders.status`), onboarding dialog, boot splash, deep-link routing, keyboard map
from 06, error boundary + `PageOffline` when the core is unreachable (reconnect UI).
**Acceptance.** All routes render feature stubs from the registry; Commander finds routes and runs registered commands; quick-add
parser has table tests (dates in EN: today, tomorrow, weekdays, "in 2 hours", `#tag`, `@Mind`, `!` priority); core-offline state shown
and recovers; keyboard shortcuts work; a11y: landmarks, focus order, `aria-current` on the active route.
**Verify.** `pnpm --filter @polterware/feature-sparky-shell test typecheck && pnpm design:lint`.

## W2-MEMORIES — memory list and detail (the core UX)

**Owns.** `packages/features/sparky-memories/**`.
**Do.** Today and Inbox pages, Mind page content list (list ↔ table toggle, pinned section, sorting, filters via
`FilterChip/FilterPopover`), bulk selection + actions, **memory detail** in the global DetailPanel (title inline edit, Mind picker,
status/pin/priority/due, Markdown note with the DS editor + autosave with `expected_rev` and conflict banner, checklist with
reorder/detail/auto-complete, tags), activity attribution line, optimistic toggles, virtualization (`@tanstack/solid-virtual`) at
10k rows, empty states, delete with confirm, undo toast for complete/delete (soft undo via `Reopen`/recreate payload held in memory 8 s).
Hosts the memory detail and renders the sections other feature packages contribute through the registry field
`detailSections` (schedule, place, attachments, focus recipe; the type is created by W0-SCAFFOLD, see 06-ui.md). It owns the
ordering and layout of those sections (`DetailPanel.Main` / `Rail`), not their content.
**Acceptance.** Create/edit/complete/pin/delete from keyboard only; edit conflict shows the banner with "reload" and "overwrite";
10k-row fixture scrolls at 60 fps (measured in Playwright trace) and filters in < 50 ms; undo works; no data loss on rapid edits.

## W2-MINDS — Mind tree and management

**Owns.** `packages/features/sparky-minds/**`.
**Do.** Sidebar tree contribution (collapse state persisted, counts, dither tiles, color/icon), create/rename/recolor/re-icon/
move (drag within the DS TreeView with keyboard alternative), reorder, delete with a confirmation that states the cascade
("Deletes N child Minds; M memories move to Inbox"), Mind page header/breadcrumb, child-Mind tiles grid, empty states, IconPicker +
ColorSwatchPicker usage.
**Acceptance.** Cycles impossible in the UI and rejected by the core; delete confirmation numbers match `{deletedMindIds,
movedMemoryCount}`; drag/drop and keyboard parity; persisted collapse state.

## W2-SCHEDULE — schedule and reminders editor

**Owns.** `packages/features/sparky-schedule/**`.
**Do.** The **Schedule card** in the memory detail: date/time or all-day, timezone (default system, changeable), repeat
(none/hourly/daily/weekly/monthly/yearly) with every-N, weekday chips, end (never/until/after N), live **preview of the next 5
occurrences** via `ScheduleService.Preview`, human summary via `Describe`, focus recipe toggle + fields (Work 1–60, breaks 1–60,
cycles 1–12, auto-continue), validation messages from the core, honest reminder state per memory (`armed|planned|notifier off|
permission denied|completed`), a "Snooze" action for the next occurrence (`ReminderService.Snooze`), per-occurrence
complete/reopen for recurring memories,
`Place` is **not** here (W2-MAP).
**Acceptance.** Every vector in 03 can be configured in the UI and previews match; DST edge preview correct for
`America/New_York`; invalid combos blocked with clear messages; changing the schedule re-arms (status chip updates within 2 s —
needs the notifier); keyboard-complete form; unit tests for the form → `ScheduleInput` mapping.

## W2-CALENDAR — day / week / month

**Owns.** `packages/features/sparky-calendar/**`.
**Reads.** Swift `Views/Desktop/Calendar/*` and `Views/Memories/Calendar/*` for behavior (all-day lane, overflow "+n", empty period
buttons, quick create by clicking a slot).
**Do.** Range navigation (prev/next/today, jump to date via DatePicker), Day (hour grid, all-day lane, current-time line),
Week, Month (cells, overflow popover), occurrence pills colored by Mind, click → DetailPanel, click empty slot → quick create
with the slot's time, drag a pill to reschedule (updates the schedule anchor for one-shots; for recurring series ask
"This occurrence only is not supported in v1 → move the whole series?" — v1 moves the series anchor and says so), timezone-correct
rendering from `ListOccurrences`, locale week start setting (Mon/Sun), motion per 06.
**Acceptance.** Matches `ListOccurrences` for 3 months of mixed recurring data; DST week renders correctly; keyboard navigation
across days; 60 fps month switch; Playwright test creating via slot click.

## W2-ATTACH-UI — attachments

**Owns.** `packages/features/sparky-attachments/**`.
**Do.** Attachments section in the detail: add menu (photo, audio record, link, file), drag-drop zone, grid for photos with
viewer (carousel), audio player + recorder (waveform optional), link rows with favicon-less title/host, file rows with
reveal-in-Finder/open, reorder, delete confirm, size/limit errors from the core, microphone permission states, `links.fetchTitles`
disclosure copy.
**Acceptance.** Import of 20 mixed files; denied microphone permission handled; previews load only through `sparky-file://`; no
`file://` use; Playwright test for drag-drop import.

## W2-FOCUS — pomodoro

**Owns.** `packages/features/sparky-focus/**`, `packages/features/sparky-focus/src/timer.ts` (pure state machine).
**Reads.** Swift `Focus/*` and `Views/Focus/*` (FocusTimer, FocusRecipe, presets, sounds, ring, canvas).
**Do.** Pure `FocusTimer` state machine (phases work/short/long, cycles, auto-continue, manual start wait, absolute timestamps
`phaseEndsAt`, pause/resume/skip/reset) with a fake-clock test suite; Focus route (ring, time, phase, cycle dots, quick focus vs
memory-bound, recipe menu, today's sessions list); on each phase start it calls `reminders.armOneOff(phaseEndsAt)` so the **OS**
notifies even if the window is hidden; sounds via Web Audio (completion sounds from settings); records sessions via
`FocusService.RecordSession`; "Start focus" from a memory and from the notification deep link; recovery if the window was
closed mid-phase (session not resumed; says so honestly).
**Acceptance.** Timer tests cover drift (system sleep simulated by clock jump), auto-continue on/off, long-break cadence;
phase-end notification delivered with the window minimized (manual evidence + notifier log); sessions appear in Review.

## W2-REVIEW — review area

**Owns.** `packages/features/sparky-review/**`.
**Reads.** Swift `Views/Settings/Me*`, `WeeklyActivityCard`, `ContributionCalendarCard`, `ContributionCalendarLayout`.
**Do.** Review route: summary card (streak, total completions, character mood art), weekly activity by day/period, 12-month
contribution calendar (`Heatmap`), focus minutes per week, range selector (week/month/year), empty/first-run state, accessible
text alternatives for charts.
**Acceptance.** Numbers match `ReviewService` fixtures and the Swift thresholds; heatmap keyboard/hover details; handles
timezone/week start correctly; chart colors from tokens only.

## W2-MAP — places and the free map

**Owns.** `packages/frontend/map/**`, `packages/features/sparky-map/**`.
**Reads.** [05](05-search-attachments-map.md) §Map, W0-MAP status/prototype, Swift `Views/Memories/Map/*` and
`Triggers/Location/*` for behavior.
**Do.** `MapView` per O4 (markers, radius circles, selection, pin pick, cluster, dark style from tokens, attribution),
Map route (map + list, bounds query via `PlaceService.ListInBounds`), **Place card** in the memory detail (pin, drag, radius,
enter/exit, name, lat/lng inputs), offline/blocked-tiles empty state, first-run disclosure + `map.online` setting, optional
geocoding adapter behind the disabled-by-default setting, CSP coordination with W1-ELECTRON through a request file if a new host
is needed.
**Acceptance.** 500 markers cluster smoothly; tiles off → honest empty state with list; the app makes **no** map network
request while `map.online` is false (verified in Playwright network log); copy states that places do not trigger notifications
in v1.

## W2-SETTINGS — settings, agents, data

**Owns.** `packages/features/sparky-settings/**`.
**Reads.** `packages/features/settings` (dialog pattern, Agents > Access).
**Do.** Settings dialog: General (background running, login item state, open data folder), Notifications (sound picker with
preview, all-day time, snooze, notifier status + "Open System Settings" when denied), Focus defaults/sounds, Map, Search
(provider config, test, index status, reindex), **Agents** (Access: enable MCP, port, token reveal/copy/rotate, allow file
attach toggle, per-client snippets for Claude Code/Cursor/Claude Desktop, Test connection; Activity: list of agent writes with
actor/time/summary), Data (export zip, backup, reset with typed confirmation), Shortcuts, Advanced (reduce transparency, logs,
`doctor` output).
**Acceptance.** Every setting round-trips through `SettingsService` with validation errors shown; token rotation invalidates the
old token (verified); snippets are copy-paste valid (tested by executing them against a dev core in an e2e); Test connection reports
failure honestly when MCP is off.

## W2-SEARCH-UI — search and filters

**Owns.** `packages/features/sparky-search/**`.
**Do.** Search page and Commander search source: mode switch (FTS/semantic/hybrid) with the true `modeUsed`, filters (Mind,
tags, status, dates), highlighted snippets, keyboard navigation, empty/degraded states ("Semantic search is not configured —
using full-text"), recent searches (local only), saved filter chips.
**Acceptance.** Results match `SearchService` fixtures; degraded-state copy accurate; handles 10k-memory data under 100 ms of UI work.

## W2-CLI — `sparky` command line

**Owns.** `sparky/backend/runtimes/cli/{memories,minds,tags,search,agenda,focus,review,export,mcpcfg}.go`, CLI tests.
**Goal.** The terminal surface (like Polterware's CLI) for scripts and agents without MCP.
**Do.** Subcommands via the Connect API: `sparky add "title" [--mind X --due … --repeat …]`, `list`, `show`, `done`, `rm --yes`,
`mind add|ls|rm`, `tag ls`, `search`, `today`, `agenda --from --to`, `review`, `export`, `backup`, `status`, `doctor`,
`mcp config <client>` (prints client snippets), `--json` on every read command, stable exit codes, shell completion
(`completion zsh|bash|fish`), `NO_COLOR`. TUI is **not** in v1.
**Acceptance.** Golden output tests; `--json` schemas stable; works with the core already running or auto-starts it; destructive
commands require `--yes`.
**Verify.** `cd sparky/backend && go test ./runtimes/... -count=1`.

---

## Gate G2 (integrator)

- [ ] `bash tooling/verify-sparky.sh` green; all Playwright specs green (`pnpm test:e2e:sparky`, runner added by W3-E2E; until
      then run each spec file individually).
- [ ] Manual script (Erick or an agent with a browser/Electron driver): create → schedule → close **everything** → reminder
      fires → Done from the notification → relaunch shows completion; agent (Claude Code) creates a memory via MCP while the
      window is open and it appears live; map pin; focus phase notification with window minimized.
- [ ] `pnpm design:lint` green; no Polterware regression.
- [ ] Every feature's empty/loading/error/offline states exist.
