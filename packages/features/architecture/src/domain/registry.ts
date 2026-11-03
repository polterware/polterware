import type {
  BoundaryKind,
  NodeState,
  NodeType,
  Port,
  RelationshipType,
  Size,
} from "@polterware/frontend-canvas";
import { DEFAULT_NODE_H, DEFAULT_NODE_W } from "@polterware/frontend-canvas";

export type ComponentCategory =
  | "applications"
  | "data"
  | "infrastructure"
  | "development"
  | "external"
  | "structure"
  | "connected";

export type PlacementKind = "node" | "boundary" | "action";

export type ComponentTypeDefinition = {
  id: string;
  label: string;
  category: ComponentCategory;
  description: string;
  /** Hugeicons type key or service-mark key (product presets). */
  icon: string;
  defaultWidth: number;
  defaultHeight: number;
  defaultPorts: Port[];
  searchTerms: string[];
  synonyms: string[];
  order: number;
  placeable: boolean;
  placementKind: PlacementKind;
  /** When placementKind === boundary */
  boundaryKind?: BoundaryKind;
  /** When placementKind === node, the ArchNode.type */
  nodeType?: NodeType;
  /** Product mark under a semantic type (e.g. postgres under database). */
  subtype?: string;
  /** Optional cloud/host provider hint (aws, vercel, …). */
  provider?: string;
};

export type RelationshipTypeDefinition = {
  id: RelationshipType;
  label: string;
  description?: string;
  order: number;
};

const inOutPorts: Port[] = [
  { id: "in", direction: "in", side: "left", label: "In" },
  { id: "out", direction: "out", side: "right", label: "Out" },
];

function nodeDef(
  partial: Omit<ComponentTypeDefinition, "placeable" | "placementKind" | "defaultPorts" | "defaultWidth" | "defaultHeight"> & {
    defaultPorts?: Port[];
    defaultWidth?: number;
    defaultHeight?: number;
    /** Semantic-only types (defaults for ports/labels) stay in registry but out of Add catalog. */
    placeable?: boolean;
  },
): ComponentTypeDefinition {
  return {
    defaultWidth: DEFAULT_NODE_W,
    defaultHeight: DEFAULT_NODE_H,
    defaultPorts: inOutPorts,
    placeable: true,
    placementKind: "node",
    nodeType: partial.id,
    ...partial,
  };
}

/** Product preset: catalog id ≠ semantic NodeType; places with subtype + service mark. */
function productDef(partial: {
  id: string;
  label: string;
  category: ComponentCategory;
  description: string;
  icon: string;
  nodeType: NodeType;
  subtype: string;
  provider?: string;
  searchTerms?: string[];
  synonyms?: string[];
  order: number;
  defaultPorts?: Port[];
  defaultWidth?: number;
  defaultHeight?: number;
}): ComponentTypeDefinition {
  return {
    defaultWidth: DEFAULT_NODE_W,
    defaultHeight: DEFAULT_NODE_H,
    defaultPorts: partial.defaultPorts ?? inOutPorts,
    placeable: true,
    placementKind: "node",
    searchTerms: partial.searchTerms ?? [partial.id, partial.label.toLowerCase()],
    synonyms: partial.synonyms ?? [],
    ...partial,
  };
}

/** Full component type catalog (registry-driven). */
export const COMPONENT_TYPES: ComponentTypeDefinition[] = [
  // Applications
  nodeDef({
    id: "web_application",
    label: "Web Application",
    category: "applications",
    description: "Browser-facing web app",
    icon: "globe",
    searchTerms: ["web", "frontend", "spa", "ssr"],
    synonyms: ["react", "vue", "nuxt"],
    order: 10,
  }),
  nodeDef({
    id: "mobile_application",
    label: "Mobile Application",
    category: "applications",
    description: "iOS / Android client",
    icon: "smartphone",
    searchTerms: ["mobile", "ios", "android"],
    synonyms: ["react native", "flutter"],
    order: 20,
  }),
  nodeDef({
    id: "desktop_application",
    label: "Desktop Application",
    category: "applications",
    description: "Native or Electron desktop app",
    icon: "monitor",
    searchTerms: ["desktop", "electron"],
    synonyms: ["tauri", "wails"],
    order: 30,
  }),
  nodeDef({
    id: "api",
    label: "API",
    category: "applications",
    description: "HTTP/gRPC API surface",
    icon: "webhook",
    searchTerms: ["api", "rest", "grpc", "http"],
    synonyms: ["rest", "openapi"],
    order: 40,
  }),
  nodeDef({
    id: "service",
    label: "Service",
    category: "applications",
    description: "Backend service / microservice",
    icon: "boxes",
    searchTerms: ["service", "microservice", "backend"],
    synonyms: ["svc"],
    order: 50,
  }),
  nodeDef({
    id: "worker",
    label: "Worker",
    category: "applications",
    description: "Async worker / consumer process",
    icon: "cog",
    searchTerms: ["worker", "consumer", "job"],
    synonyms: ["sidekiq", "celery"],
    order: 60,
  }),
  nodeDef({
    id: "scheduled_job",
    label: "Scheduled Job",
    category: "applications",
    description: "Cron / scheduled task",
    icon: "clock",
    searchTerms: ["schedule", "cron", "job"],
    synonyms: ["cron", "crontab", "scheduler"],
    order: 70,
  }),
  // Semantic-only (not in Add catalog) — place via product presets
  nodeDef({
    id: "serverless_function",
    label: "Serverless Function",
    category: "applications",
    description: "FaaS / edge function",
    icon: "zap",
    searchTerms: ["function", "serverless", "edge"],
    synonyms: [],
    order: 80,
    placeable: false,
  }),
  // Data — catalog shows products only (PostgreSQL, Redis, …)
  nodeDef({
    id: "database",
    label: "Database",
    category: "data",
    description: "Relational or document database",
    icon: "database",
    searchTerms: ["db", "sql", "database"],
    synonyms: [],
    order: 100,
    placeable: false,
  }),
  nodeDef({
    id: "cache",
    label: "Cache",
    category: "data",
    description: "In-memory cache",
    icon: "memory-stick",
    searchTerms: ["cache"],
    synonyms: [],
    order: 110,
    placeable: false,
  }),
  nodeDef({
    id: "queue",
    label: "Queue",
    category: "data",
    description: "Message queue",
    icon: "list-ordered",
    searchTerms: ["queue", "mq", "broker"],
    synonyms: [],
    order: 120,
    placeable: false,
  }),
  nodeDef({
    id: "event_stream",
    label: "Event Stream",
    category: "data",
    description: "Event log / stream",
    icon: "activity",
    searchTerms: ["stream", "events", "pubsub"],
    synonyms: [],
    order: 130,
    placeable: false,
  }),
  nodeDef({
    id: "storage",
    label: "Object Storage",
    category: "data",
    description: "Blob / object storage",
    icon: "hard-drive",
    searchTerms: ["storage", "blob", "object"],
    synonyms: [],
    order: 140,
    placeable: false,
  }),
  nodeDef({
    id: "search_engine",
    label: "Search Engine",
    category: "data",
    description: "Full-text / vector search",
    icon: "search",
    searchTerms: ["search", "index"],
    synonyms: [],
    order: 150,
    placeable: false,
  }),
  nodeDef({
    id: "data_warehouse",
    label: "Data Warehouse",
    category: "data",
    description: "Analytics warehouse",
    icon: "warehouse",
    searchTerms: ["warehouse", "analytics", "olap"],
    synonyms: [],
    order: 160,
    placeable: false,
  }),
  // Infrastructure — products only in Add catalog
  nodeDef({
    id: "runtime_host",
    label: "Runtime",
    category: "infrastructure",
    description: "Host, VM, container, or cluster runtime",
    icon: "server",
    searchTerms: ["runtime", "host", "server", "vm", "container"],
    synonyms: [],
    order: 200,
    placeable: false,
  }),
  nodeDef({
    id: "load_balancer",
    label: "Load Balancer",
    category: "infrastructure",
    description: "Traffic load balancer",
    icon: "scale",
    searchTerms: ["lb", "load balancer"],
    synonyms: [],
    order: 210,
    placeable: false,
  }),
  nodeDef({
    id: "api_gateway",
    label: "API Gateway",
    category: "infrastructure",
    description: "Edge API gateway",
    icon: "route",
    searchTerms: ["gateway", "api gateway"],
    synonyms: ["kong", "apigee"],
    order: 220,
  }),
  nodeDef({
    id: "cdn",
    label: "CDN",
    category: "infrastructure",
    description: "Content delivery network",
    icon: "cloud",
    searchTerms: ["cdn", "edge"],
    synonyms: [],
    order: 230,
    placeable: false,
  }),
  nodeDef({
    id: "network",
    label: "Network",
    category: "infrastructure",
    description: "VPC / network segment",
    icon: "network",
    searchTerms: ["network", "vpc", "subnet"],
    synonyms: ["vpc"],
    order: 240,
  }),
  nodeDef({
    id: "dns",
    label: "DNS",
    category: "infrastructure",
    description: "DNS zone or resolver",
    icon: "globe-2",
    searchTerms: ["dns", "domain"],
    synonyms: ["route53", "cloudflare dns"],
    order: 250,
  }),
  nodeDef({
    id: "region",
    label: "Region",
    category: "infrastructure",
    description: "Cloud region marker",
    icon: "map-pin",
    searchTerms: ["region", "az"],
    synonyms: ["us-east-1"],
    order: 260,
  }),
  // Development — brand products in catalog; semantic types stay non-placeable
  nodeDef({
    id: "repository",
    label: "Repository",
    category: "development",
    description: "Source code repository",
    icon: "git-branch",
    searchTerms: ["repo", "git", "source"],
    synonyms: [],
    order: 300,
    placeable: false,
  }),
  nodeDef({
    id: "package",
    label: "Package",
    category: "development",
    description: "Library or package",
    icon: "package",
    searchTerms: ["package", "library", "npm", "crate"],
    synonyms: ["npm", "pypi"],
    order: 310,
  }),
  nodeDef({
    id: "ci_pipeline",
    label: "CI/CD Pipeline",
    category: "development",
    description: "Continuous integration pipeline",
    icon: "git-merge",
    searchTerms: ["ci", "cd", "pipeline"],
    synonyms: [],
    order: 320,
    placeable: false,
  }),
  nodeDef({
    id: "build_system",
    label: "Build System",
    category: "development",
    description: "Build / compile system",
    icon: "hammer",
    searchTerms: ["build", "compile"],
    synonyms: [],
    order: 330,
    placeable: false,
  }),
  nodeDef({
    id: "artifact_registry",
    label: "Artifact Registry",
    category: "development",
    description: "Container or package registry",
    icon: "archive",
    searchTerms: ["registry", "artifacts", "images"],
    synonyms: ["ecr", "ghcr", "docker hub"],
    order: 340,
  }),
  // External — brand products in catalog
  nodeDef({
    id: "external_service",
    label: "External Service",
    category: "external",
    description: "Third-party system",
    icon: "external-link",
    searchTerms: ["external", "third party"],
    synonyms: [],
    order: 400,
  }),
  nodeDef({
    id: "saas_provider",
    label: "SaaS Provider",
    category: "external",
    description: "SaaS product",
    icon: "cloud-cog",
    searchTerms: ["saas"],
    synonyms: [],
    order: 410,
    placeable: false,
  }),
  nodeDef({
    id: "third_party_api",
    label: "Third-party API",
    category: "external",
    description: "External HTTP API",
    icon: "plug",
    searchTerms: ["third-party api"],
    synonyms: [],
    order: 420,
  }),
  nodeDef({
    id: "auth_provider",
    label: "Authentication Provider",
    category: "external",
    description: "IdP / auth service",
    icon: "shield",
    searchTerms: ["auth", "sso", "oidc"],
    synonyms: [],
    order: 430,
    placeable: false,
  }),
  nodeDef({
    id: "payment_provider",
    label: "Payment Provider",
    category: "external",
    description: "Payment processing",
    icon: "credit-card",
    searchTerms: ["payment"],
    synonyms: [],
    order: 440,
    placeable: false,
  }),
  nodeDef({
    id: "integration",
    label: "Integration",
    category: "external",
    description: "Integration connector",
    icon: "link",
    searchTerms: ["integration", "connector"],
    synonyms: [],
    order: 450,
  }),
  // Structural markers
  nodeDef({
    id: "system",
    label: "System",
    category: "structure",
    description: "System root marker",
    icon: "layout-dashboard",
    searchTerms: ["system", "project"],
    synonyms: [],
    order: 500,
    defaultWidth: 160,
    defaultHeight: 64,
  }),
  {
    id: "boundary_system",
    label: "System Boundary",
    category: "structure",
    description: "System boundary region",
    icon: "square",
    defaultWidth: 320,
    defaultHeight: 200,
    defaultPorts: [],
    searchTerms: ["boundary", "system boundary"],
    synonyms: [],
    order: 510,
    placeable: false,
    placementKind: "boundary",
    boundaryKind: "system",
  },
  {
    id: "boundary_domain",
    label: "Domain Boundary",
    category: "structure",
    description: "Domain / bounded context",
    icon: "square",
    defaultWidth: 320,
    defaultHeight: 200,
    defaultPorts: [],
    searchTerms: ["domain", "context", "boundary"],
    synonyms: ["bounded context"],
    order: 520,
    placeable: false,
    placementKind: "boundary",
    boundaryKind: "domain",
  },
  {
    id: "boundary_deployment",
    label: "Deployment Boundary",
    category: "structure",
    description: "Deployment region",
    icon: "square",
    defaultWidth: 320,
    defaultHeight: 200,
    defaultPorts: [],
    searchTerms: ["deployment", "boundary"],
    synonyms: [],
    order: 530,
    placeable: false,
    placementKind: "boundary",
    boundaryKind: "deployment",
  },
  {
    id: "boundary_team",
    label: "Ownership Boundary",
    category: "structure",
    description: "Ownership region",
    icon: "square",
    defaultWidth: 320,
    defaultHeight: 200,
    defaultPorts: [],
    searchTerms: ["ownership", "boundary"],
    synonyms: [],
    order: 540,
    placeable: false,
    placementKind: "boundary",
    boundaryKind: "team",
  },
  nodeDef({
    id: "note",
    label: "Text Note",
    category: "structure",
    description: "Annotation note",
    icon: "sticky-note",
    searchTerms: ["note", "text", "annotation"],
    synonyms: [],
    order: 550,
    defaultPorts: [],
    defaultWidth: 160,
    defaultHeight: 80,
  }),
  nodeDef({
    id: "custom_resource",
    label: "Custom Resource",
    category: "structure",
    description: "Custom or unknown type",
    icon: "box",
    searchTerms: ["custom"],
    synonyms: [],
    order: 560,
  }),

  // ── Product presets (service marks; semantic type + subtype) ────────────
  // Data
  productDef({
    id: "postgres",
    label: "PostgreSQL",
    category: "data",
    description: "Relational database",
    icon: "postgres",
    nodeType: "database",
    subtype: "postgres",
    searchTerms: ["postgres", "postgresql", "pg", "sql"],
    synonyms: ["psql", "rds postgres"],
    order: 101,
  }),
  productDef({
    id: "mongodb",
    label: "MongoDB",
    category: "data",
    description: "Document database",
    icon: "mongodb",
    nodeType: "database",
    subtype: "mongodb",
    searchTerms: ["mongo", "mongodb", "document"],
    synonyms: ["atlas"],
    order: 102,
  }),
  productDef({
    id: "mysql",
    label: "MySQL",
    category: "data",
    description: "Relational database",
    icon: "mysql",
    nodeType: "database",
    subtype: "mysql",
    searchTerms: ["mysql", "mariadb", "sql"],
    synonyms: ["maria"],
    order: 103,
  }),
  productDef({
    id: "redis",
    label: "Redis",
    category: "data",
    description: "In-memory cache / store",
    icon: "redis",
    nodeType: "cache",
    subtype: "redis",
    searchTerms: ["redis", "cache", "valkey"],
    synonyms: ["elasticache"],
    order: 111,
  }),
  productDef({
    id: "rabbitmq",
    label: "RabbitMQ",
    category: "data",
    description: "Message broker",
    icon: "rabbitmq",
    nodeType: "queue",
    subtype: "rabbitmq",
    searchTerms: ["rabbit", "rabbitmq", "amqp", "queue"],
    synonyms: ["rmq"],
    order: 121,
  }),
  productDef({
    id: "kafka",
    label: "Apache Kafka",
    category: "data",
    description: "Event streaming platform",
    icon: "kafka",
    nodeType: "event_stream",
    subtype: "kafka",
    searchTerms: ["kafka", "stream", "events"],
    synonyms: ["msk", "confluent"],
    order: 131,
  }),
  productDef({
    id: "elasticsearch",
    label: "Elasticsearch",
    category: "data",
    description: "Search and analytics engine",
    icon: "elasticsearch",
    nodeType: "search_engine",
    subtype: "elasticsearch",
    searchTerms: ["elastic", "elasticsearch", "search", "opensearch"],
    synonyms: ["elk"],
    order: 151,
  }),
  productDef({
    id: "s3",
    label: "Amazon S3",
    category: "data",
    description: "Object storage",
    icon: "s3",
    nodeType: "storage",
    subtype: "s3",
    provider: "aws",
    searchTerms: ["s3", "bucket", "object storage"],
    synonyms: ["amazon s3"],
    order: 141,
  }),
  productDef({
    id: "supabase",
    label: "Supabase",
    category: "data",
    description: "Postgres platform / BaaS",
    icon: "supabase",
    nodeType: "database",
    subtype: "supabase",
    searchTerms: ["supabase", "postgres baas"],
    synonyms: [],
    order: 104,
  }),
  productDef({
    id: "snowflake",
    label: "Snowflake",
    category: "data",
    description: "Cloud data warehouse",
    icon: "snowflake",
    nodeType: "data_warehouse",
    subtype: "snowflake",
    searchTerms: ["snowflake", "warehouse"],
    synonyms: [],
    order: 161,
  }),
  productDef({
    id: "bigquery",
    label: "BigQuery",
    category: "data",
    description: "Google analytics warehouse",
    icon: "bigquery",
    nodeType: "data_warehouse",
    subtype: "bigquery",
    provider: "gcp",
    searchTerms: ["bigquery", "bq", "google warehouse"],
    synonyms: [],
    order: 162,
  }),
  productDef({
    id: "prisma",
    label: "Prisma",
    category: "data",
    description: "ORM / data layer",
    icon: "prisma",
    nodeType: "package",
    subtype: "prisma",
    searchTerms: ["prisma", "orm"],
    synonyms: [],
    order: 311,
  }),
  productDef({
    id: "graphql",
    label: "GraphQL",
    category: "applications",
    description: "GraphQL API surface",
    icon: "graphql",
    nodeType: "api",
    subtype: "graphql",
    searchTerms: ["graphql", "gql"],
    synonyms: ["apollo"],
    order: 41,
  }),

  // Applications / edge
  productDef({
    id: "nextjs",
    label: "Next.js",
    category: "applications",
    description: "React web application framework",
    icon: "nextjs",
    nodeType: "web_application",
    subtype: "nextjs",
    searchTerms: ["next", "nextjs", "react"],
    synonyms: ["next.js"],
    order: 11,
  }),
  productDef({
    id: "vercel",
    label: "Vercel",
    category: "external",
    description: "Frontend / edge hosting platform",
    icon: "vercel",
    nodeType: "saas_provider",
    subtype: "vercel",
    provider: "vercel",
    searchTerms: ["vercel", "hosting", "edge"],
    synonyms: [],
    order: 411,
  }),
  productDef({
    id: "cloudflare",
    label: "Cloudflare",
    category: "infrastructure",
    description: "CDN / edge network",
    icon: "cloudflare",
    nodeType: "cdn",
    subtype: "cloudflare",
    searchTerms: ["cloudflare", "cdn", "workers"],
    synonyms: ["cf"],
    order: 231,
  }),

  // Infrastructure / cloud
  productDef({
    id: "aws",
    label: "AWS",
    category: "infrastructure",
    description: "Amazon Web Services cloud",
    icon: "aws",
    nodeType: "saas_provider",
    subtype: "aws",
    provider: "aws",
    searchTerms: ["aws", "amazon", "cloud"],
    synonyms: ["amazon web services"],
    order: 401,
  }),
  productDef({
    id: "gcp",
    label: "Google Cloud",
    category: "infrastructure",
    description: "Google Cloud Platform",
    icon: "gcp",
    nodeType: "saas_provider",
    subtype: "gcp",
    provider: "gcp",
    searchTerms: ["gcp", "google cloud", "gcloud"],
    synonyms: [],
    order: 402,
  }),
  productDef({
    id: "azure",
    label: "Microsoft Azure",
    category: "infrastructure",
    description: "Azure cloud platform",
    icon: "azure",
    nodeType: "saas_provider",
    subtype: "azure",
    provider: "azure",
    searchTerms: ["azure", "microsoft cloud"],
    synonyms: [],
    order: 403,
  }),
  productDef({
    id: "docker",
    label: "Docker",
    category: "infrastructure",
    description: "Container runtime",
    icon: "docker",
    nodeType: "runtime_host",
    subtype: "docker",
    searchTerms: ["docker", "container"],
    synonyms: [],
    order: 201,
  }),
  productDef({
    id: "kubernetes",
    label: "Kubernetes",
    category: "infrastructure",
    description: "Container orchestration",
    icon: "kubernetes",
    nodeType: "runtime_host",
    subtype: "kubernetes",
    searchTerms: ["kubernetes", "k8s", "cluster"],
    synonyms: ["eks", "gke", "aks"],
    order: 202,
  }),
  productDef({
    id: "nginx",
    label: "Nginx",
    category: "infrastructure",
    description: "Reverse proxy / load balancer",
    icon: "nginx",
    nodeType: "load_balancer",
    subtype: "nginx",
    searchTerms: ["nginx", "proxy", "lb"],
    synonyms: [],
    order: 211,
  }),
  productDef({
    id: "lambda",
    label: "AWS Lambda",
    category: "applications",
    description: "Serverless functions on AWS",
    icon: "lambda",
    nodeType: "serverless_function",
    subtype: "lambda",
    provider: "aws",
    searchTerms: ["lambda", "serverless", "faas"],
    synonyms: [],
    order: 81,
  }),
  productDef({
    id: "sqs",
    label: "Amazon SQS",
    category: "data",
    description: "Managed message queue",
    icon: "sqs",
    nodeType: "queue",
    subtype: "sqs",
    provider: "aws",
    searchTerms: ["sqs", "queue", "amazon queue"],
    synonyms: [],
    order: 122,
  }),
  productDef({
    id: "digitalocean",
    label: "DigitalOcean",
    category: "infrastructure",
    description: "Cloud VMs and managed services",
    icon: "digitalocean",
    nodeType: "saas_provider",
    subtype: "digitalocean",
    provider: "digitalocean",
    searchTerms: ["digitalocean", "do", "droplet"],
    synonyms: [],
    order: 404,
  }),
  productDef({
    id: "terraform",
    label: "Terraform",
    category: "development",
    description: "Infrastructure as code",
    icon: "terraform",
    nodeType: "build_system",
    subtype: "terraform",
    searchTerms: ["terraform", "iac", "hcl"],
    synonyms: ["opentofu"],
    order: 331,
  }),

  // Development
  productDef({
    id: "github",
    label: "GitHub",
    category: "development",
    description: "Source code hosting",
    icon: "github",
    nodeType: "repository",
    subtype: "github",
    searchTerms: ["github", "git", "repo"],
    synonyms: [],
    order: 301,
  }),
  productDef({
    id: "gitlab",
    label: "GitLab",
    category: "development",
    description: "Source code hosting",
    icon: "gitlab",
    nodeType: "repository",
    subtype: "gitlab",
    searchTerms: ["gitlab", "git", "repo"],
    synonyms: [],
    order: 302,
  }),
  productDef({
    id: "github_actions",
    label: "GitHub Actions",
    category: "development",
    description: "CI/CD pipelines on GitHub",
    icon: "github_actions",
    nodeType: "ci_pipeline",
    subtype: "github_actions",
    searchTerms: ["github actions", "actions", "ci", "cd"],
    synonyms: ["gha"],
    order: 321,
  }),

  // External / SaaS
  productDef({
    id: "auth0",
    label: "Auth0",
    category: "external",
    description: "Authentication platform",
    icon: "auth0",
    nodeType: "auth_provider",
    subtype: "auth0",
    searchTerms: ["auth0", "auth", "oidc"],
    synonyms: [],
    order: 431,
  }),
  productDef({
    id: "clerk",
    label: "Clerk",
    category: "external",
    description: "Authentication platform",
    icon: "clerk",
    nodeType: "auth_provider",
    subtype: "clerk",
    searchTerms: ["clerk", "auth"],
    synonyms: [],
    order: 432,
  }),
  productDef({
    id: "stripe",
    label: "Stripe",
    category: "external",
    description: "Payments platform",
    icon: "stripe",
    nodeType: "payment_provider",
    subtype: "stripe",
    searchTerms: ["stripe", "payments"],
    synonyms: [],
    order: 441,
  }),
  productDef({
    id: "datadog",
    label: "Datadog",
    category: "external",
    description: "Observability platform",
    icon: "datadog",
    nodeType: "saas_provider",
    subtype: "datadog",
    searchTerms: ["datadog", "monitoring", "apm", "observability"],
    synonyms: [],
    order: 412,
  }),
  productDef({
    id: "openai",
    label: "OpenAI",
    category: "external",
    description: "AI / LLM API provider",
    icon: "openai",
    nodeType: "third_party_api",
    subtype: "openai",
    searchTerms: ["openai", "gpt", "llm", "ai"],
    synonyms: ["chatgpt"],
    order: 421,
  }),
  productDef({
    id: "prometheus",
    label: "Prometheus",
    category: "external",
    description: "Metrics and monitoring",
    icon: "prometheus",
    nodeType: "saas_provider",
    subtype: "prometheus",
    searchTerms: ["prometheus", "metrics", "monitoring"],
    synonyms: [],
    order: 413,
  }),
  productDef({
    id: "grafana",
    label: "Grafana",
    category: "external",
    description: "Dashboards and observability",
    icon: "grafana",
    nodeType: "saas_provider",
    subtype: "grafana",
    searchTerms: ["grafana", "dashboards", "observability"],
    synonyms: [],
    order: 414,
  }),

  // Connected (actions, not placeable)
  {
    id: "link-from-resources",
    label: "Link from synced resources",
    category: "connected",
    description: "Place a component linked to a synced integration resource",
    icon: "link-2",
    defaultWidth: 0,
    defaultHeight: 0,
    defaultPorts: [],
    searchTerms: ["link", "resource", "synced"],
    synonyms: ["link existing resource"],
    order: 900,
    placeable: false,
    placementKind: "action",
  },
  {
    id: "action_import_integration",
    label: "Import from integration",
    category: "connected",
    description: "Import discovered resources (not available yet)",
    icon: "download",
    defaultWidth: 0,
    defaultHeight: 0,
    defaultPorts: [],
    searchTerms: ["import", "integration"],
    synonyms: [],
    order: 910,
    placeable: false,
    placementKind: "action",
  },
  {
    id: "action_import_file",
    label: "Import from file",
    category: "connected",
    description: "Import architecture JSON (use Import)",
    icon: "file-up",
    defaultWidth: 0,
    defaultHeight: 0,
    defaultPorts: [],
    searchTerms: ["import", "file", "json"],
    synonyms: [],
    order: 920,
    placeable: false,
    placementKind: "action",
  },
];

/** @deprecated use COMPONENT_TYPES / listComponentTypes — kept for compatibility */
export type TypeMeta = { id: NodeType; label: string };

/** Semantic node types only (excludes product presets with subtype). */
export const NODE_TYPES: TypeMeta[] = COMPONENT_TYPES.filter(
  (t) => t.placementKind === "node" && t.nodeType && !t.subtype,
).map((t) => ({ id: t.nodeType ?? t.id, label: t.label }));

export const RELATIONSHIP_TYPES: RelationshipTypeDefinition[] = [
  { id: "calls", label: "Calls", order: 10, description: "Invokes or calls the target (RPC, HTTP, …)" },
  { id: "depends_on", label: "Depends on", order: 20, description: "Requires the target to function" },
  { id: "reads_from", label: "Reads from", order: 30, description: "Reads data from the target" },
  { id: "writes_to", label: "Writes to", order: 40, description: "Writes data to the target" },
  { id: "publishes_to", label: "Publishes to", order: 50, description: "Publishes events or messages to the target" },
  { id: "consumes_from", label: "Consumes from", order: 60, description: "Consumes events or messages from the target" },
  { id: "stores_in", label: "Stores in", order: 70, description: "Persists data in the target" },
  { id: "authenticates_with", label: "Authenticates with", order: 80, description: "Authenticates against the target" },
  { id: "routes_to", label: "Routes to", order: 90, description: "Routes traffic or requests to the target" },
  { id: "deploys_to", label: "Deploys to", order: 100, description: "Deploys artifacts to the target" },
  { id: "hosted_on", label: "Hosted on", order: 110, description: "Runs on the target host or platform" },
  { id: "monitors", label: "Monitors", order: 120, description: "Observes health or metrics of the target" },
  { id: "replicates_to", label: "Replicates to", order: 130, description: "Replicates data to the target" },
  { id: "synchronizes_with", label: "Synchronizes with", order: 140, description: "Keeps state in sync with the target" },
  { id: "exposes", label: "Exposes", order: 150, description: "Exposes the target as an interface or API" },
  { id: "contains", label: "Contains", order: 160, description: "Owns or contains the target" },
  { id: "related", label: "Related", order: 170, description: "Generic association with the target" },
  { id: "deployed_on", label: "Deployed on", order: 180, description: "Is deployed on the target runtime or host" },
  { id: "connects_to", label: "Connects to", order: 190, description: "Maintains a network or integration link" },
  { id: "custom", label: "Custom", order: 999, description: "Custom relationship type" },
];

export const NODE_STATES: NodeState[] = [
  "planned",
  "linked",
  "observed",
  "stale",
  "drifted",
  "unreachable",
  "unknown",
];

export const CATEGORY_LABELS = {
  applications: "Applications",
  data: "Data",
  infrastructure: "Infrastructure",
  development: "Development",
  external: "External",
  structure: "Structure",
  connected: "Connected resources",
} satisfies Record<ComponentCategory, string>;

export function listComponentTypes(): ComponentTypeDefinition[] {
  return [...COMPONENT_TYPES].sort((a, b) => a.order - b.order);
}

/** Placeable nodes + connected actions — what the Add dialog lists. */
export function isCatalogVisible(t: ComponentTypeDefinition): boolean {
  if (t.placementKind === "boundary") return false;
  return t.placeable || t.placementKind === "action";
}

export function listCatalogTypes(): ComponentTypeDefinition[] {
  return listComponentTypes().filter(isCatalogVisible);
}

export function getComponentType(id: string): ComponentTypeDefinition | undefined {
  return COMPONENT_TYPES.find((t) => t.id === id);
}

export function searchComponentTypes(query: string): ComponentTypeDefinition[] {
  const q = query.trim().toLowerCase();
  const all = listComponentTypes();
  if (!q) return all;
  return all.filter((t) => {
    if (t.label.toLowerCase().includes(q)) return true;
    if (t.id.toLowerCase().includes(q)) return true;
    if (t.category.toLowerCase().includes(q)) return true;
    if (t.description.toLowerCase().includes(q)) return true;
    if (t.searchTerms.some((s) => s.toLowerCase().includes(q))) return true;
    if (t.synonyms.some((s) => s.toLowerCase().includes(q))) return true;
    return false;
  });
}

/** Search restricted to Add-catalog entries (products + placeable abstracts + actions). */
export function searchCatalogTypes(query: string): ComponentTypeDefinition[] {
  return searchComponentTypes(query).filter(isCatalogVisible);
}

export function listRelationshipTypes(): RelationshipTypeDefinition[] {
  return [...RELATIONSHIP_TYPES].sort((a, b) => a.order - b.order);
}

export const CREATABLE_RELATIONSHIP_TYPES = [
  "depends_on",
  "calls",
  "reads_from",
  "writes_to",
  "publishes_to",
  "consumes_from",
  "routes_to",
  "deployed_on",
  "connects_to",
] as const satisfies readonly RelationshipType[];

export function listCreatableRelationshipTypes(): RelationshipTypeDefinition[] {
  return CREATABLE_RELATIONSHIP_TYPES.map((id) => getRelationshipType(id)!);
}

export function getRelationshipType(id: string): RelationshipTypeDefinition | undefined {
  return RELATIONSHIP_TYPES.find((t) => t.id === id);
}

export function labelForNodeType(t: NodeType | string): string {
  const byNode = COMPONENT_TYPES.find(
    (x) => !x.subtype && (x.nodeType === t || x.id === t),
  );
  return byNode?.label ?? String(t);
}

/** Prefer product label when subtype is set (e.g. postgres → PostgreSQL). */
export function labelForNode(input: {
  type: NodeType | string;
  subtype?: string | null;
}): string {
  if (input.subtype) {
    const product = COMPONENT_TYPES.find(
      (x) => x.subtype === input.subtype || x.id === input.subtype,
    );
    if (product) return product.label;
  }
  return labelForNodeType(input.type);
}

export function labelForRelType(t: RelationshipType | string): string {
  return RELATIONSHIP_TYPES.find((x) => x.id === t)?.label ?? String(t);
}

/** Resolve catalog entry by semantic type id (not product preset id). */
function semanticTypeDef(nodeType: string): ComponentTypeDefinition | undefined {
  return COMPONENT_TYPES.find(
    (t) => !t.subtype && t.placementKind === "node" && (t.id === nodeType || t.nodeType === nodeType),
  );
}

export function defaultPorts(nodeType: NodeType | string): Port[] {
  const def = semanticTypeDef(String(nodeType)) ?? getComponentType(String(nodeType));
  if (def) return def.defaultPorts.map((p) => ({ ...p }));
  if (nodeType === "note") return [];
  return inOutPorts.map((p) => ({ ...p }));
}

export function defaultNodeSize(nodeType: NodeType | string): Size {
  const def = semanticTypeDef(String(nodeType)) ?? getComponentType(String(nodeType));
  if (def && def.placementKind === "node") {
    return { width: def.defaultWidth, height: def.defaultHeight };
  }
  if (nodeType === "note") return { width: 160, height: 80 };
  if (nodeType === "system") return { width: 160, height: 64 };
  return { width: DEFAULT_NODE_W, height: DEFAULT_NODE_H };
}

export function statusTone(
  state: NodeState,
): "ok" | "warn" | "danger" | "unknown" | "muted" {
  switch (state) {
    case "observed":
      return "ok";
    case "linked":
      // Linked ≠ healthy — neutral tone until observed evidence exists
      return "unknown";
    case "stale":
    case "drifted":
      return "warn";
    case "unreachable":
      return "danger";
    case "planned":
      return "muted";
    default:
      return "unknown";
  }
}

/** Derive display state from origin + link fields (honest defaults). */
export function deriveNodeState(input: {
  origin?: string;
  state?: NodeState;
  resourceId?: string | null;
  linkStatus?: string;
}): NodeState {
  if (input.state && input.state !== "unknown" && input.state !== "planned") {
    return input.state;
  }
  if (input.resourceId || input.linkStatus === "linked") return "linked";
  if (input.origin === "discovered") return "observed";
  if (input.origin === "manual" || !input.origin) return "planned";
  return input.state ?? "unknown";
}
