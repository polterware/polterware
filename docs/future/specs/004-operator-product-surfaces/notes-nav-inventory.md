# Nav inventory (004 implement)

## Current → target

| Legacy section | New home |
| --- | --- |
| architecture | architecture (tab canvas default) |
| resources | resources (tab inventory default) |
| ops | ops (tab runs default) |
| processes | resources + tab processes |
| logs | hand-off / ops quick action (not sidebar) |
| knowledge | knowledge (tab docs default) |
| approvals | ops + tab approvals |
| workflows | ops + tab workflows |
| commander | command palette overlay |
| agents | access + tab agents |
| integrations | integrations (tab connected default) |
| settings | settings (tab project default) |

## Package map

| Destination | Package | Action |
| --- | --- | --- |
| shell | feature-shell | expand route + DEFAULT_NAV |
| architecture | feature-architecture | expand tabs |
| resources | feature-resources | nodes/processes/drawer |
| operations | feature-operations | expand tabs |
| knowledge | feature-knowledge | expand tabs |
| access | **feature-access (new)** | full destination |
| integrations | feature-integrations | expand |
| settings | feature-settings | expand |
| commander | feature-commander | palette overlay only |
| processes/approvals/workflows/agents | keep helpers; not primary nav | absorb |

## New packages

Only `packages/features/access` — justified primary destination.
