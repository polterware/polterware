import type { NodeFixture, ProcessFixture, Resource } from "./types";

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

export const NODE_FIXTURES: NodeFixture[] = [
  { id: "node-local", name: "local", platform: "macOS · arm64", environment: "Development", status: "healthy", address: "127.0.0.1", cpu: "8 cores", memory: "16 GB", lastSeen: "Now" },
  { id: "node-nitro", name: "nitro", platform: "Linux · amd64", environment: "Production", status: "healthy", address: "10.0.0.12", cpu: "8 cores", memory: "32 GB", lastSeen: "12s ago" },
  { id: "node-prod-vps", name: "prod-vps-01", platform: "Linux · amd64", environment: "Production", status: "degraded", address: "10.0.0.20", cpu: "4 cores", memory: "8 GB", lastSeen: "1m ago" },
  { id: "node-worker-01", name: "worker-node-01", platform: "Linux · amd64", environment: "Production", status: "healthy", address: "10.0.0.22", cpu: "4 cores", memory: "8 GB", lastSeen: "30s ago" },
];

export const PROCESS_FIXTURES: ProcessFixture[] = [
  { id: "proc-api", name: "example-api", nodeId: "node-local", nodeName: "local", status: "running", pid: 4122, command: "./example-api serve", startedAt: "2h ago" },
  { id: "proc-postgres", name: "postgres", nodeId: "node-nitro", nodeName: "nitro", status: "running", pid: 1012, command: "postgres -D /var/lib/postgresql/data", startedAt: "4d ago" },
  { id: "proc-worker", name: "settlement-worker", nodeId: "node-worker-01", nodeName: "worker-node-01", status: "running", pid: 2280, command: "./settlement-worker", startedAt: "6h ago" },
  { id: "proc-cache", name: "redis", nodeId: "node-prod-vps", nodeName: "prod-vps-01", status: "failed", command: "redis-server /etc/redis.conf", startedAt: "Stopped 18m ago" },
];
