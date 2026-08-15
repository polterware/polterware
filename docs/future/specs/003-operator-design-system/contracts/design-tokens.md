# Contract: Design Tokens

**Owner**: `packages/configuration/tailwind/base.css` + root `DESIGN.md`  
**Consumers**: Desktop renderer, design-system, features

## Required CSS variables (minimum)

Names may use the project’s final aliasing, but **semantics** MUST exist:

### Surfaces

| Token | Role | Initial value (guidance) |
| --- | --- | --- |
| app-background | Window behind chrome | `#070707` |
| shell-background / sidebar-background | Integrated chrome | `#090909` |
| surface | Main surface fill | `#0f0f0f` |
| surface-subtle | Nested quiet areas | `#121212` |
| surface-raised | Raised controls/rows | `#151515` |
| surface-hover | Hover | `#191919` |
| surface-active | Pressed | `#1e1e1e` |
| surface-selected | Selection / active nav | `#232323` |

### Borders & text

| Token | Guidance |
| --- | --- |
| border-subtle | `rgb(255 255 255 / 0.055)` |
| border-default | `rgb(255 255 255 / 0.085)` |
| border-strong | `rgb(255 255 255 / 0.14)` |
| text-primary | `rgb(255 255 255 / 0.92)` |
| text-secondary | `rgb(255 255 255 / 0.62)` |
| text-muted | `rgb(255 255 255 / 0.40)` |
| text-disabled | `rgb(255 255 255 / 0.25)` |

### Primary CTA & brand & semantic

| Token | Guidance |
| --- | --- |
| primary | `#8eccc9` — pastel cyan; rare special buttons only |
| primary-muted | `rgb(142 204 201 / 0.14)` |
| brand | `#01DD96` — restricted (status/success; not routine buttons) |
| brand-muted | `rgb(1 221 150 / 0.12)` |
| danger | `#e46666` |
| warning | `#d8a85d` |
| info | `#7e9fd8` |

### Radius

`control` 7px · `item` 8px · `card` 10px · `surface` 14px · `window` 18px

### Motion

- `ease-standard`: `cubic-bezier(0.2, 0, 0, 1)`  
- `ease-enter`: `cubic-bezier(0.16, 1, 0.3, 1)`  
- Durations: short control ≤ 160ms; enter ≤ 200ms; respect `prefers-reduced-motion`

### Shell dimensions

| Token | Guidance |
| --- | --- |
| sidebar-width | ~220–240px (compact) |
| page-header-height | ~48–56px |
| control-height-sm | ~28–32px |
| control-height-md | ~32–36px |

## Compatibility aliases

Existing `--color-bg`, `--color-surface`, `--color-fg`, `--color-muted`, `--color-border` MUST map to the new system so transitional call sites do not break during migration. Prefer migrating call sites to semantic names.

## Forbidden

- Pure `#000` as sole surface language without luminance ladder  
- Brand green as active nav background  
- Arbitrary hex in feature TSX for in-scope screens after migration
)
