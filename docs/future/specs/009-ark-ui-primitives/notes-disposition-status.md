# Disposition status tracker

**Layering rule**: Ark = behavior; DESIGN + tokens + Tailwind = look; local DS components = public API. Features must not import Ark or Kobalte.

| Export | Status | Task |
| --- | --- | --- |
| Dialog / A11yDialog | **done** (Ark) | T008 |
| Popover | **done** (Ark) | T009 |
| Drawer | **done** (Ark) | T010 |
| Tabs / TabPanel | **done** (Ark tabs + presentational panel) | T017 |
| MenuPanel / MenuItem | **done** (presentational rows, no false roles) | T019 |
| DropdownMenu* | **done** (Ark Menu wrapper; first consumer TBD) | T020–T022 |
| Toast | **deferred** — keep live region; optional later | T030 |
| Input/Textarea/Select/Field/FilterChip/Button/Badge/Card/… | **leave** | T028 |
| ResizableSplit | **out-of-scope** | T031 |

| Hand-roll | Status |
| --- | --- |
| Command palette | **done** → A11yDialog |
| Terminal side panel | **done** → Drawer |
| Relationship type picker | **done** → Popover + MenuItem rows |
| Context switchers / pickers | **verify** — still Popover + presentational rows |

| Kobalte purge | Status |
| --- | --- |
| DS imports | **done** |
| DS package.json | **done** |
| apps/desktop dep + vite | **done** |
