import { For, type JSX } from "solid-js";
import "./dither-icon.css";

export type DitherIconTone =
  | "neutral"
  | "architecture"
  | "infrastructure"
  | "data"
  | "execution"
  | "knowledge"
  | "identity"
  | "security"
  | "integration"
  | "observability";

export type DitherIconSize = 20 | 24 | 28;

export type DitherIconTileProps = {
  children: JSX.Element;
  size?: DitherIconSize;
  tone?: DitherIconTone;
  brand?: string | readonly string[];
  dimmed?: boolean;
  class?: string;
};

/**
 * Light pastel accents only — Bayer cells sit behind a dark mark.
 * Keep brand hue, raise luminance; never use dark brand hexes here.
 */
export const DITHER_BRAND_ACCENTS = {
  auth0: "#F29A7A",
  aws: "#FFB84D",
  azure: "#6BB3F0",
  bigquery: "#8FB8F8",
  clerk: "#A994FF",
  cloudflare: "#F5A85A",
  datadog: "#B08FD9",
  digitalocean: "#66B3FF",
  docker: "#6BB8F5",
  elasticsearch: "#6EB8C9",
  gcp: "#7EB3F5",
  github: "#EDEDED",
  github_actions: "#6BB3FF",
  gitlab: "#F5A06A",
  grafana: "#F5A05A",
  graphql: "#F08CC8",
  kafka: "#C8C8C8",
  kubernetes: "#7EA8F0",
  lambda: "#FFB84D",
  mongodb: "#7EC98A",
  mysql: "#7EAFD0",
  nextjs: "#EDEDED",
  nginx: "#6EC99A",
  openai: "#6EC9B0",
  postgres: "#7EA8C9",
  prisma: "#B8C4D0",
  prometheus: "#F08A6A",
  rabbitmq: "#FF9966",
  redis: "#F08A80",
  s3: "#8EC97A",
  snowflake: "#6EC8F0",
  sqs: "#FF8FB8",
  stripe: "#A99CFF",
  supabase: "#6ED9A8",
  terraform: "#B08FD9",
  vercel: "#EDEDED",
} as const;

const DITHER_TONE_ACCENTS = {
  neutral: "var(--color-dither-accent-neutral)",
  architecture: "var(--color-chart-blue)",
  infrastructure: "var(--color-chart-cyan)",
  data: "var(--color-chart-green)",
  execution: "var(--color-chart-blue)",
  knowledge: "var(--color-chart-amber)",
  identity: "var(--color-chart-cyan)",
  security: "var(--color-chart-amber)",
  integration: "var(--color-chart-blue)",
  observability: "var(--color-chart-green)",
} satisfies Record<DitherIconTone, string>;

const BRAND_ALIASES = {
  amazon_web_services: "aws",
  githubactions: "github_actions",
  google_cloud: "gcp",
  google_cloud_platform: "gcp",
  next_js: "nextjs",
  postgresql: "postgres",
} satisfies Record<string, keyof typeof DITHER_BRAND_ACCENTS>;

const BRAND_KEYS = Object.keys(DITHER_BRAND_ACCENTS).sort(
  (left, right) => right.length - left.length,
);

const BAYER_4X4 = [
  [0, 8, 2, 10],
  [12, 4, 14, 6],
  [3, 11, 1, 9],
  [15, 7, 13, 5],
].map((row) => row.map((value) => (value + 0.5) / 16));

const GRID_SIZE = 16;

/**
 * Ordered Bayer texture with mild density only — no top→bottom fade into a
 * dark void. Unpainted cells show the light accent-tinted base, not black.
 */
const DITHER_CELLS = Array.from({ length: GRID_SIZE * GRID_SIZE }, (_, index) => {
  const x = index % GRID_SIZE;
  const y = Math.floor(index / GRID_SIZE);
  // ~44%–58% fill: even texture, slight depth, never sparse black top.
  const density = 0.44 + ((y + 1) / GRID_SIZE) * 0.14;
  const threshold = BAYER_4X4[y & 3]?.[x & 3] ?? 0.5;
  return density >= threshold ? { x, y } : null;
}).filter((cell): cell is { x: number; y: number } => cell !== null);

function normalizeBrandCandidate(value: string): string {
  return value
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "");
}

function hasOwnKey<T extends {}>(value: T, key: PropertyKey): key is keyof T {
  return Object.hasOwn(value, key);
}

function isBrandList(value: string | readonly string[]): value is readonly string[] {
  return Array.isArray(value);
}

function resolveBrandAccent(candidate: string): string | undefined {
  const normalized = normalizeBrandCandidate(candidate);
  if (!normalized) return undefined;

  if (hasOwnKey(BRAND_ALIASES, normalized)) {
    return DITHER_BRAND_ACCENTS[BRAND_ALIASES[normalized]];
  }

  if (hasOwnKey(DITHER_BRAND_ACCENTS, normalized)) {
    return DITHER_BRAND_ACCENTS[normalized];
  }

  const bounded = `_${normalized}_`;
  const key = BRAND_KEYS.find((brandKey) => bounded.includes(`_${brandKey}_`));
  return key && hasOwnKey(DITHER_BRAND_ACCENTS, key)
    ? DITHER_BRAND_ACCENTS[key]
    : undefined;
}

export function resolveDitherIconAccent(
  brand: string | readonly string[] | undefined,
  tone: DitherIconTone = "neutral",
): string {
  const candidates = brand === undefined ? [] : isBrandList(brand) ? brand : [brand];
  for (const candidate of candidates) {
    const accent = resolveBrandAccent(candidate);
    if (accent) return accent;
  }
  return DITHER_TONE_ACCENTS[tone];
}

export function DitherIconTile(props: DitherIconTileProps) {
  const size = () => props.size ?? 24;
  const accent = () =>
    props.dimmed
      ? "var(--color-dither-accent-neutral)"
      : resolveDitherIconAccent(props.brand, props.tone);

  return (
    <span
      data-dither-icon
      data-size={size()}
      class={`pw-dither-icon ${props.dimmed ? "pw-dither-icon--dimmed" : ""} ${props.class ?? ""}`}
      style={{ width: `${size()}px`, height: `${size()}px`, color: accent() }}
      aria-hidden="true"
    >
      <svg
        class="pw-dither-icon-pattern"
        viewBox={`0 0 ${GRID_SIZE} ${GRID_SIZE}`}
        preserveAspectRatio="none"
        {...{ "shape-rendering": "crispEdges" }}
      >
        <For each={DITHER_CELLS}>
          {(cell) => (
            <rect x={cell.x} y={cell.y} width="1" height="1" fill="currentColor" />
          )}
        </For>
      </svg>
      <span class="pw-dither-icon-mark">{props.children}</span>
    </span>
  );
}
