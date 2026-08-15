# Contracts Overview: Remaining Operator Surfaces

**Feature**: `007-remaining-operator-surfaces`  
**Kind**: UI contracts (presentation boundaries), not wire protobuf schemas  
**Extends**: `specs/004-operator-product-surfaces/contracts/*`

## Index

| Contract | Path | Owner |
| --- | --- | --- |
| Shell IA v2 | [shell-ia.md](./shell-ia.md) | `features/shell` + App |
| Global chrome | [global-chrome.md](./global-chrome.md) | shell, commander, title bar |
| Observability UI | [observability-ui.md](./observability-ui.md) | `features/observability` |
| Provisioning UI | [provisioning-ui.md](./provisioning-ui.md) | `features/resources` |
| Access expansions | [access-expansions-ui.md](./access-expansions-ui.md) | `features/access` |
| Integrations expansions | [integrations-expansions-ui.md](./integrations-expansions-ui.md) | `features/integrations` |
| Settings expansions | [settings-expansions-ui.md](./settings-expansions-ui.md) | `features/settings` |
| Architecture deep flows | [architecture-deep-ui.md](./architecture-deep-ui.md) | `features/architecture` |
| Workflow builder | [workflow-builder-ui.md](./workflow-builder-ui.md) | `features/operations` |
| Data honesty | [data-honesty.md](./data-honesty.md) | all features |

## Relationship to 004 contracts

| 004 contract | 007 impact |
| --- | --- |
| shell-ia.md | Superseded by shell-ia v2 (eight destinations, INSIGHTS, chrome sections) |
| overlays.md | Extended: notifications drawer; process drawer retained |
| resources-ui.md | + Provisioning tab and wizards |
| access-ui.md | + Teams, Invitations, roles, audit detail, agent session |
| integrations-ui.md | + Webhooks, setup wizard, shared conflict |
| settings-ui.md | + Organization, Developer, Billing, Security, Data |
| operations-ui.md | + Workflow builder, schedules strip |
| architecture-ui.md | + Import, compare, view editor |
| page-tabs-states.md | Still applies to all new surfaces |
| core-data-adapter.md | Still applies; fixtures remain honest |

## Non-goals for contracts

- Full specialized-platform API surface  
- Hand-maintained duplicate protobuf types  
- React component APIs
