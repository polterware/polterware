# Agent tooling in the project sidebar

Status: Idea only — not shipped, not a specification

Date: 2026-09-28

Source: Discussion about seeing and potentially managing skills, plugins, hooks, and related agent configuration across coding harnesses, both globally and per project.

## Intent

Make the local agent setup legible in one place: which harness has which capability, where it is defined, whether it applies globally or to the selected project, and where setups differ. The proposed navigation is an **Agents group in the sidebar with its own routes**, rather than more tabs inside the current Agents page.

## Candidate navigation

The group name and route set are provisional:

| Route | Candidate responsibility |
| --- | --- |
| Inventory | Cross-harness view of discovered configuration, source paths, scope, and overlaps or differences. |
| Skills | Skill definitions, source, content, and availability by harness and project. |
| Plugins | Installed plugins, versions when available, and harness-specific enablement. |
| Hooks | Triggers, configured actions, and scope. |
| Connections | Existing Polterware MCP access and harness connection setup, currently under Agents → Access. |
| Context | Existing facts, evidence, compiled context, and drift preview, currently under Agents → Facts and Context. |
| Activity | Existing change history and agent activity, currently under Agents → History. |

This is a navigation sketch, not a decision to ship seven routes. Inventory might be the entry point while some asset types share a route if the sidebar becomes too dense.

## Boundaries and open decisions

- Decide whether the first release only discovers and compares configuration or also installs, edits, enables, and removes it.
- Decide how a project-scoped sidebar represents global configuration and whether an all-project view is needed.
- Keep each harness's configuration files as the source of truth unless a later specification explicitly defines an owned canonical configuration and synchronization behavior.
- Distinguish discovery from support: harnesses have different skill, plugin, hook, and MCP conventions. Unsupported or unreadable configuration must be shown honestly.
- Any file mutation needs a preview of the exact target and change, with conflict handling and a clear owner in the local core; the renderer does not access the filesystem directly.
- Reconcile this proposal with the existing Agents routes, the current product navigation boundary, and the separate context-files idea before promoting it to a specification.

Nothing here changes the shipped sidebar or authorizes implementation.
