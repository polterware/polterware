# Contract: Access UI

**Owner**: `packages/features/access` (new; may wrap `feature-agents`)  
**Consumers**: `App.tsx`

## Tabs

`members` | `agents` | `policies` | `audit`

## Members

- List: name, role, teams, status, last seen  
- Detail: permissions, auth, activity; edit/remove when permitted  
- Invite → dialog

## Agents

- List: name, type, scope, status, last activity  
- Detail: capabilities allow/deny matrix, activity; disable/rotate token when permitted  
- Register → dialog

## Policies

- List: name, scope, enforcement, status, updated  
- Detail: applies-to, conditions, requirements, exceptions  
- Create/edit → dialog or detail form

## Audit

- List: time, actor, action, target, result  
- Filters: actor, action, result, time range  
- Export when permitted (download file or copy—implementation may stub export with honesty if core lacks endpoint)

## Security

Role chips are not authorization. Forbidden actions use `PageForbidden` or disabled controls with reason. Agents cannot approve own high-risk requests when policy forbids.
