# Polterware GPUI Desktop

The Polterware Desktop in GPUI, on the shared library [polterware/ui](https://github.com/polterware/ui). It is a client of the local Go core like the Electron Desktop (`apps/polterware-desktop`), which stays the shipped app until this one reaches parity.

It finds the core that owns the database through `<db dir>/run/core.json` and reads it over Connect-JSON (`src/client.rs`). It never opens SQLite. The database is `POLTERWARE_DB`, else `~/Library/Application Support/polterware/local.db`. When no core runs, Start the core runs `polterware serve` detached, with the binary from `POLTERWARE_BIN`, the installed app, or this checkout's `dist/polterware` (`pnpm build:cli`).

## Run

```bash
pnpm build:cli
cargo run --manifest-path desktop-gpui/Cargo.toml
```

## Checks

```bash
cd desktop-gpui
cargo fmt --check
cargo clippy --all-targets
cargo test
POLTERWARE_TEST_BIN=../dist/polterware cargo test --test live_core
```

`POLTERWARE_GPUI_SHOTS=<dir>` runs a tour that captures every screen to PNG and quits. Point `POLTERWARE_DB` at a scratch database to run it.

## What it has

- The frame: the title bar (Home, the breadcrumb, Commander, App settings), the project sidebar (Now: Inbox with its open count, Activity; Project: Milestones, Canvas, Structure, Knowledge; Project settings in the footer), and the inset main surface.
- The Projects screen, reached through Home, with every local project.
- Commander (⌘K): the project's sections, the projects to switch to, App settings, and Reload.
- Inbox (`WorkService.ListInbox`): items grouped by kind, open before Later, with the active item's detail. It does not act on items yet.

The rest, in the order worth porting, is in [docs/future.md](../docs/future.md#gpui-desktop).
