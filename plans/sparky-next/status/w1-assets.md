# w1-assets status
- Result: partial
- Branch / last commit: `sparky/w1-assets` / pending
- Acceptance criteria:
  - App icon, alternate icons, logos, and preview sheet: **pass** — `sips -g pixelWidth -g pixelHeight apps/sparky-desktop/resources/app-icons/*.png apps/sparky-desktop/resources/logo/*.png`; main PNG is 1024 × 1024; `iconutil --convert iconset --output /tmp/sparky-verify.iconset apps/sparky-desktop/resources/icon.icns` enumerates 16, 32, 128, 256, and 512 px 1x/2x representations; visual review of `resources/previews/legibility-sheet.png` shows all 8 points at 16/32/64/128 on light and dark without clipping.
  - Mood characters exported: **pass** — 8 states (streak/completed × sad/hopeful/happy/euphoric), each at 1x/2x/3x in `apps/sparky-desktop/resources/characters/`; source PNG dimensions preserved.
  - Sounds for notification and Web Audio: **pass** — 8 named sounds copied as CAF into `apps/sparky-notifier/Resources/` and transcoded to Opus/WebM into `apps/sparky-desktop/resources/sounds/`; `ffprobe` duration check reports 0.18–0.51 s, below 30 s.
  - Inventory table: **pass** — below.
- Asset inventory:

  | Name | Notification file | Web Audio file |
  |---|---|---|
  | bell | `apps/sparky-notifier/Resources/bell.caf` | `apps/sparky-desktop/resources/sounds/bell.webm` |
  | chime | `apps/sparky-notifier/Resources/chime.caf` | `apps/sparky-desktop/resources/sounds/chime.webm` |
  | end | `apps/sparky-notifier/Resources/end.caf` | `apps/sparky-desktop/resources/sounds/end.webm` |
  | glass | `apps/sparky-notifier/Resources/glass.caf` | `apps/sparky-desktop/resources/sounds/glass.webm` |
  | pause | `apps/sparky-notifier/Resources/pause.caf` | `apps/sparky-desktop/resources/sounds/pause.webm` |
  | ping | `apps/sparky-notifier/Resources/ping.caf` | `apps/sparky-desktop/resources/sounds/ping.webm` |
  | pop | `apps/sparky-notifier/Resources/pop.caf` | `apps/sparky-desktop/resources/sounds/pop.webm` |
  | start | `apps/sparky-notifier/Resources/start.caf` | `apps/sparky-desktop/resources/sounds/start.webm` |

  Character inventory: `sparky-character.webp`; `me-streak-{sad,hopeful,happy,euphoric}` and `me-completed-{sad,hopeful,happy,euphoric}` (PNG 1x/2x/3x).
- Deviations from the plan (and why): `actool` on the installed Xcode emitted only metadata for the `.icon` source; the 1024 px PNG uses its approved `Star 1.png` layer and dark fill. The 1024 px black/white variants and logo marks are monochrome versions of that symbol. Optional green dot omitted for Erick's review. The accepted `iconutil` conversion is a generated iconset using the same 1024 px source.
- Polterware session commands: `polterware` is absent from PATH. `go run ./backend/runtimes/cli format` succeeded. `go run ./backend/runtimes/cli query cards` and `status` returned `no project for this directory`; did not initialize a project or alter another database. `handoff` cannot run without a project.
- Requests filed: none.
- Follow-ups for other WS: W1-ELECTRON should add/port `branding-assets.test.ts` to assert the Sparky resource paths and exact icon variants. Erick to review the legibility sheet and optional green dot.
