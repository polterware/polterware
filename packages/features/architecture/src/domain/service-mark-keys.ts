/** Pure service mark keys used by architecture cards and integrations. */
export const SERVICE_MARK_KEYS = [
  "auth0",
  "aws",
  "azure",
  "bigquery",
  "clerk",
  "cloudflare",
  "datadog",
  "digitalocean",
  "docker",
  "elasticsearch",
  "fly",
  "gcp",
  "github",
  "github_actions",
  "gitlab",
  "grafana",
  "graphql",
  "kafka",
  "kubernetes",
  "lambda",
  "mongodb",
  "mysql",
  "netlify",
  "nextjs",
  "nginx",
  "openai",
  "postgres",
  "prisma",
  "prometheus",
  "rabbitmq",
  "railway",
  "redis",
  "render",
  "s3",
  "snowflake",
  "sqs",
  "stripe",
  "supabase",
  "terraform",
  "vercel",
] as const;

export type ServiceMarkKey = (typeof SERVICE_MARK_KEYS)[number];

const KEY_SET = new Set<string>(SERVICE_MARK_KEYS);

export function hasServiceMark(name: string): name is ServiceMarkKey {
  return KEY_SET.has(name);
}
