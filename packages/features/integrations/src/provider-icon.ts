/**
 * Map integration provider display names / catalog ids to architecture
 * service-mark keys (same marks used on canvas cards).
 */
const ALIASES = {
  github: "github",
  gitlab: "gitlab",
  aws: "aws",
  "amazon web services": "aws",
  "google cloud": "gcp",
  gcp: "gcp",
  gcloud: "gcp",
  azure: "azure",
  "microsoft azure": "azure",
  vercel: "vercel",
  supabase: "supabase",
  terraform: "terraform",
  docker: "docker",
} satisfies Record<string, string>;

function normalizeToken(raw: string): string {
  return raw
    .trim()
    .toLowerCase()
    .replace(/^cat-/, "")
    .replace(/^int-/, "")
    .replace(/[_-]+/g, " ")
    .replace(/\s+/g, " ");
}

function toKeyCandidate(raw: string): string {
  return normalizeToken(raw).replace(/\s+/g, "_").replace(/[^a-z0-9_]/g, "");
}

/**
 * Resolve a service-mark / type-icon key for an integration provider.
 * Falls back to `plug` (Hugeicons) when no brand mark exists.
 */
export function providerIconKey(input: {
  name?: string;
  provider?: string;
  id?: string;
}): string {
  const candidates = [input.provider, input.name, input.id].filter(
    (v): v is string => v !== undefined && v !== null && v.trim().length > 0,
  );

  for (const c of candidates) {
    const token = normalizeToken(c);
    const tokenAlias = Object.entries(ALIASES).find(([alias]) => alias === token)?.[1];
    if (tokenAlias) return tokenAlias;

    const key = toKeyCandidate(c);
    const normalizedKey = key.replace(/_/g, " ");
    const keyAlias = Object.entries(ALIASES).find(([alias]) => alias === normalizedKey)?.[1];
    if (keyAlias) return keyAlias;
    if (key) return key;
  }

  return "plug";
}
