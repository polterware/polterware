# Polterware ecosystem

Polterware is an umbrella for apps and tools by Erick Barcelos that can work **together**. It is not a bucket for everything Erick builds. A project that cannot integrate with the rest stays a personal project.

## Admission criteria

A project joins the umbrella when both of these hold:

1. **It integrates with at least one sibling** through a real contract: CLI, MCP, Connect RPC, shared schemas, or a documented file format. "Could integrate someday" does not count; the first integration must be nameable.
2. **It is useful alone.** Each member runs without the others; integrations add value but are never required.

Not requirements: being local-first, having its own core or store, keeping a separate data model, or using the shared design system and release path. Members may share them (most do today), but that is a convention, not a gate.

Anything that fails either criterion is a personal project: it lives in its own repository and is not listed on the Polterware site.

## Integration is allowed to be active

Members may command each other. A member can launch agents, call another member's tools, or write into another member's store through its public contract. There is no "observe only" rule between members. The only limit is each member's own security boundaries (credentials, sandboxes).

## Roster

| Member | Role | Integrates with |
|---|---|---|
| Polterware | Project context, work cards, decisions, commits | Agents via MCP/CLI (`polter.apply`, `polter.query`, AgentRun) |
| Polterware Vitals | SSH host metrics | Local core protocol |
| Polterware Dispatch | Routes a task to the right coding-agent CLI by difficulty and quota | Polterware through `DispatchHostService` and AgentRun; Polterware reads its runs (below) |

## Polterware Dispatch: integrations

Admitted. It is useful alone (a workspace can be any folder) and integrates with Polterware in both directions:

- **Polterware → Dispatch**: Dispatch reads Polterware's projects, a run's directory, area, and card text through `DispatchHostService`, and records each delegation as an AgentRun that its agent writes under.
- **Dispatch → Polterware**: Polterware lists a card's runs and Activity's dispatch entries from Dispatch's core, and Commander and Dispatch this card start runs there, opening them in the Dispatch app through `dispatch://`.

Still open: Polterware Vitals host metrics informing where heavy runs go.
