export const CURRENT_SCOPE = [
  "Polterware v1 is a local SQLite-backed Desktop.",
  "The renderer reads and writes only through window.polterware. It never spawns shells or uses Node APIs.",
  "Go core owns local execution: terminal PTY sessions, operations, workflows, and integration CLI probes.",
  "Product surfaces are Architecture, Resources, Knowledge, Operations, Terminal, and Integrations. Settings stays in the sidebar footer.",
  "Knowledge exposes Docs, ADRs, Runbooks, and Specs (including one project Constitution and per-feature Spec, Plan, Tasks, Clarify, and Checklist).",
  "The integration catalog is exactly GitHub, GitLab, AWS, Google Cloud, Azure, Vercel, Supabase, Terraform, and Docker.",
  "Provider credentials stay in the OS or CLI credential store. SQLite must not store provider tokens.",
  "Web App, Cloud, the Polterware Project Model, and in-product agents are product direction. They are not current behavior.",
  "Shared UI uses the design-system controls: Input, Textarea, Select, Field, Button, Card, Surface, overlays, PageToolbar, FilterChip, DataTable, DitherIconTile, StatusChip, and DetailPanel.",
].join(" ");

export const AREAS = {
  desktop: "Electron Main, preload, or the Solid renderer under apps/desktop",
  core: "polterware-core under backend/runtimes/core",
  "shared-package": "packages/features, packages/frontend, or packages/contracts",
  schema: "Protobuf schemas and generated contracts",
  documentation: "docs/spec.md, ADRs, or other repository documentation",
  none: "Tooling, configuration, or a change that does not belong to one product area",
};
