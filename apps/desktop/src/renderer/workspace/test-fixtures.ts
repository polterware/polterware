import type { WorkspaceCatalogProvider } from "@polterware/feature-shell";
import type { TerminalTargetItem } from "@polterware/feature-shell";

/** Catalog names that must remain available from ListCliProviders. */
export const CATALOG_PROVIDER_NAMES = [
  "GitHub",
  "GitLab",
  "AWS",
  "Google Cloud",
  "Azure",
  "Vercel",
  "Supabase",
  "Terraform",
  "Docker",
] as const;

export const CATALOG_TEST_PROVIDERS: WorkspaceCatalogProvider[] = CATALOG_PROVIDER_NAMES.map((name) => ({
  id: name.toLowerCase().replace(/\s+/g, "-"),
  name,
  description: `${name} CLI provider`,
  category: "Source Control",
  binary: name === "Google Cloud" ? "gcloud" : name.toLowerCase().replace(/\s+/g, ""),
  authCommands: [`${name.toLowerCase()} auth login`],
  transcript: [`$ ${name.toLowerCase()} auth login`],
  state: "ready",
}));

/** Picker must block open with honest feedback. Not seeded into SQLite. */
export const UNUSABLE_TERMINAL_TEST_TARGETS: TerminalTargetItem[] = [
  { id: "node-offline-01", name: "offline-edge-01", shell: "bash", status: "offline", environmentId: "env-polter-prod" },
  { id: "node-revoked-01", name: "revoked-node-01", shell: "bash", status: "revoked", environmentId: "env-polter-prod" },
];
