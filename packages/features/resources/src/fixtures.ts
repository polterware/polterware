import type { Resource } from "./types";

export const RESOURCE_FIXTURES: Resource[] = [
  {
    id: "res-api", name: "api-service", type: "Service", provider: "Local",
    environment: "Development", status: "healthy", region: "local", address: "http://127.0.0.1:4000",
    tags: ["api", "local"], description: "Local API service represented by fixture metadata.", dataSource: "fixture",
  },
  {
    id: "res-rds-prod", name: "prod-postgres", type: "Database", provider: "AWS",
    environment: "Production", status: "healthy", region: "us-east-1",
    tags: ["postgres", "data"], description: "Database metadata imported from the AWS fixture.", dataSource: "fixture",
  },
  {
    id: "res-cache", name: "session-cache", type: "Cache", provider: "Docker",
    environment: "Development", status: "degraded", region: "local",
    tags: ["redis", "cache"], description: "Local cache with a simulated degraded health state.", dataSource: "fixture",
  },
  {
    id: "res-repo-polter", name: "polterware/polter", type: "Repository", provider: "GitHub",
    environment: "All", status: "healthy", region: "remote",
    tags: ["git", "source"], description: "Repository metadata imported from the GitHub fixture.", dataSource: "fixture",
  },
  {
    id: "res-vercel-web", name: "polter-web", type: "Project", provider: "Vercel",
    environment: "Production", status: "healthy", region: "global",
    tags: ["web", "deployment"], description: "Deployment project metadata imported from the Vercel fixture.", dataSource: "fixture",
  },
  {
    id: "res-supabase-main", name: "polter-main", type: "Database project", provider: "Supabase",
    environment: "Development", status: "healthy", region: "remote",
    tags: ["database", "development"], description: "Project metadata imported from the Supabase fixture.", dataSource: "fixture",
  },
];
