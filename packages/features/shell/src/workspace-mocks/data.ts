import type {
  WorkspaceCatalogProvider,
  WorkspaceDocument,
  WorkspaceIntegration,
  WorkspaceIntegrationResource,
  WorkspaceIntegrationSyncRun,
  WorkspaceRunbook,
} from "./types";

export const CATALOG_PROVIDER_MOCKS: WorkspaceCatalogProvider[] = [
  {
    id: "github",
    name: "GitHub",
    description: "Repositories and project metadata through the GitHub CLI.",
    category: "Source Control",
    binary: "gh",
    authCommands: ["gh auth login"],
    state: "connected",
    transcript: ["$ gh auth login", "? Where do you use GitHub? GitHub.com", "✓ Authentication complete (mock)"],
  },
  {
    id: "gitlab",
    name: "GitLab",
    description: "Repositories and pipelines through the GitLab CLI.",
    category: "Source Control",
    binary: "glab",
    authCommands: ["glab auth login"],
    state: "ready",
    transcript: ["$ glab auth login", "? GitLab hostname: gitlab.com", "✓ Authentication complete (mock)"],
  },
  {
    id: "aws",
    name: "AWS",
    description: "Resources through the AWS CLI SSO flow.",
    category: "Cloud",
    binary: "aws",
    authCommands: ["aws configure sso", "aws sso login"],
    state: "connected",
    transcript: ["$ aws configure sso", "SSO profile configured (mock)", "$ aws sso login", "✓ Login complete (mock)"],
  },
  {
    id: "google-cloud",
    name: "Google Cloud",
    description: "Projects and resources through the Google Cloud CLI.",
    category: "Cloud",
    binary: "gcloud",
    authCommands: ["gcloud auth login"],
    state: "ready",
    transcript: ["$ gcloud auth login", "Waiting for browser confirmation… (mock)", "✓ Authentication complete (mock)"],
  },
  {
    id: "azure",
    name: "Azure",
    description: "Subscriptions and resources through the Azure CLI.",
    category: "Cloud",
    binary: "az",
    authCommands: ["az login"],
    state: "ready",
    transcript: ["$ az login", "Select a subscription… (mock)", "✓ Authentication complete (mock)"],
  },
  {
    id: "vercel",
    name: "Vercel",
    description: "Projects and deployments through the Vercel CLI.",
    category: "Deployment",
    binary: "vercel",
    authCommands: ["vercel login"],
    state: "connected",
    transcript: ["$ vercel login", "Waiting for confirmation… (mock)", "✓ Authentication complete (mock)"],
  },
  {
    id: "supabase",
    name: "Supabase",
    description: "Local project links through the Supabase CLI.",
    category: "Databases",
    binary: "supabase",
    authCommands: ["supabase login"],
    state: "connected",
    transcript: ["$ supabase login", "Opening confirmation flow… (mock)", "✓ Authentication complete (mock)"],
  },
  {
    id: "terraform",
    name: "Terraform",
    description: "Workspace access through the Terraform CLI.",
    category: "Infrastructure",
    binary: "terraform",
    authCommands: ["terraform login"],
    state: "ready",
    transcript: ["$ terraform login", "Terraform Cloud hostname confirmed (mock)", "✓ Authentication complete (mock)"],
  },
  {
    id: "docker",
    name: "Docker",
    description: "Registry access through the Docker CLI.",
    category: "Infrastructure",
    binary: "docker",
    authCommands: ["docker login"],
    state: "ready",
    transcript: ["$ docker login", "Using device confirmation flow (mock)", "✓ Authentication complete (mock)"],
  },
];

export const INTEGRATION_MOCKS: WorkspaceIntegration[] = [
  {
    id: "int-github", providerId: "github", name: "GitHub", provider: "GitHub",
    status: "connected", connectionLabel: "local-demo", environment: "All",
    environments: ["Production", "Staging"], lastSync: "30s ago",
    summary: "Four repositories available in this session fixture.", resourceCount: 4, cliCommand: "gh auth login", syncInterval: "Manual", freshness: "fresh",
  },
  {
    id: "int-aws", providerId: "aws", name: "AWS", provider: "AWS",
    status: "connected", connectionLabel: "demo-profile", environment: "Production",
    environments: ["Production"], lastSync: "1m ago",
    summary: "Three resources available in this session fixture.", resourceCount: 3, cliCommand: "aws configure sso", syncInterval: "Manual", freshness: "fresh",
  },
  {
    id: "int-vercel", providerId: "vercel", name: "Vercel", provider: "Vercel",
    status: "connected", connectionLabel: "demo-workspace", environment: "Production",
    environments: ["Production", "Preview"], lastSync: "2m ago",
    summary: "Two projects available in this session fixture.", resourceCount: 2, cliCommand: "vercel login", syncInterval: "Manual", freshness: "fresh",
  },
  {
    id: "int-supabase", providerId: "supabase", name: "Supabase", provider: "Supabase",
    status: "connected", connectionLabel: "demo-project", environment: "Development",
    environments: ["Development"], lastSync: "5m ago",
    summary: "One project available in this session fixture.", resourceCount: 1, cliCommand: "supabase login", syncInterval: "Manual", freshness: "fresh",
  },
];

export const INTEGRATION_RESOURCE_MOCKS: WorkspaceIntegrationResource[] = [
  { id: "ires-gh", integrationId: "int-github", name: "polterware/polter", kind: "Repository", externalId: "repo-polter", environment: "All", status: "synced" },
  { id: "ires-aws", integrationId: "int-aws", name: "prod-postgres", kind: "RDS", externalId: "db-prod-01", environment: "Production", status: "synced" },
  { id: "ires-vercel", integrationId: "int-vercel", name: "polter-web", kind: "Project", externalId: "prj_web", environment: "Production", status: "synced" },
  { id: "ires-supabase", integrationId: "int-supabase", name: "polter-main", kind: "Project", externalId: "sb-polter-main", environment: "Development", status: "synced" },
];

export const INTEGRATION_SYNC_RUN_MOCKS: WorkspaceIntegrationSyncRun[] = INTEGRATION_MOCKS.map((integration, index) => ({
  id: `sync-${integration.providerId}`,
  integrationId: integration.id,
  startedAt: `${index + 1}m ago`,
  finishedAt: `${index + 1}m ago`,
  result: "success",
  imported: integration.resourceCount,
  changed: 0,
  message: "Fixture metadata refreshed in memory.",
}));

export const DOCUMENT_MOCKS: WorkspaceDocument[] = [
  {
    id: "doc-architecture", title: "System architecture",
    summary: "How the local project components fit together.",
    bodyMarkdown: "This document describes the local project topology represented on the Architecture canvas.\n\n## Boundaries\n\nComponents, resources, and operational workflows stay scoped to this project.",
    author: "Project maintainer", updatedAt: "2h ago", tags: ["architecture", "local"],
    componentIds: ["cmp-api", "cmp-database"], componentNames: ["API", "Database"],
    environment: "All", status: "published", owner: "Platform", entityLinks: [],
  },
  {
    id: "doc-database", title: "Database conventions",
    summary: "Naming, migrations, and backup windows.",
    bodyMarkdown: "Use expand-and-contract migrations and validate backups before maintenance.\n\n## Naming\n\n- Tables: `snake_case` plural\n- Indexes: `idx_<table>_<columns>`",
    author: "Project maintainer", updatedAt: "4d ago", tags: ["database", "conventions"],
    componentIds: ["cmp-database"], componentNames: ["Database"],
    environment: "All", status: "draft", owner: "Platform", entityLinks: [],
  },
];

function runbookMarkdown(input: Omit<WorkspaceRunbook, "bodyMarkdown" | "entityLinks">): string {
  return `## Objective\n\n${input.objective}\n\n## Prerequisites\n\n${input.prerequisites.map((item) => `- ${item}`).join("\n")}\n\n## Steps\n\n${input.steps.map((item, index) => `${index + 1}. ${item}`).join("\n")}\n\n## Commands\n\n\`\`\`bash\n${input.commands.join("\n")}\n\`\`\`\n\n## Rollback\n\n${input.rollback}`;
}

const RUNBOOK_SEEDS: Omit<WorkspaceRunbook, "bodyMarkdown" | "entityLinks">[] = [
  {
    id: "rb-service", title: "Service recovery",
    objective: "Restore a local service after a failed health check.", service: "API", severity: "high",
    prerequisites: ["Confirm the affected environment", "Confirm the service component in the Atlas"],
    steps: ["Inspect the service fixture", "Review the restart command", "Record the expected result"],
    commands: ["systemctl status example-api", "systemctl restart example-api"],
    rollback: "Restore the previous service version if the health check remains degraded.",
    owner: "Platform", lastValidated: "4 days ago", componentId: "cmp-api", componentName: "API",
  },
  {
    id: "rb-database", title: "Database failover",
    objective: "Review a failover sequence for the primary database.", service: "Database", severity: "critical",
    prerequisites: ["Verify the standby fixture is healthy", "Pause local writes"],
    steps: ["Review replication state", "Promote the standby", "Validate application health"],
    commands: ["pg_isready", "psql -c 'select now();'"],
    rollback: "Restore the previous primary reference if validation fails.",
    owner: "Platform", lastValidated: "12 days ago", componentId: "cmp-database", componentName: "Database", resourceId: "res-rds-prod", resourceName: "prod-postgres",
  },
];

export const RUNBOOK_MOCKS: WorkspaceRunbook[] = RUNBOOK_SEEDS.map((runbook) => ({
  ...runbook,
  entityLinks: [],
  bodyMarkdown: runbookMarkdown(runbook),
}));
