# Contract: Settings UI

**Owner**: `packages/features/settings`  
**Consumers**: `App.tsx`

## Tabs

`project` | `environments` | `notifications` | `core` | `account`

## Project (System alias)

- Name, description, default environment, system/project ID with copy  
- Danger zone delete with confirm; disable if core cannot wipe

## Environments

- Table: name, resource count, node count, protection, default  
- Protection rules visible for selected/default production

## Notifications

- Event × channel matrix (desktop, email, slack)  
- Quiet hours  
- Unwired channels: toggle may no-op with honesty or disabled

## Core

- Status, version, endpoint, database label, data directory  
- Restart / open logs / open data directory (bridge when available)  
- Startup toggles, updates check  
- MUST reflect real offline state

## Account

- Profile name/email, appearance (theme, density), MFA/session summary  
- Manage sessions entry when available
