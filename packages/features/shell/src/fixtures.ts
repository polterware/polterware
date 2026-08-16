import type {
  EnvironmentItem,
  ProjectSwitcherItem,
  TerminalTargetItem,
} from "./types";

export const PROJECT_SWITCHER_FIXTURES: ProjectSwitcherItem[] = [
  {
    projectId: "proj-polter",
    name: "Polter",
    directory: "~/projects/polter",
    environmentSummary: "Production",
    isCurrent: true,
    status: "active",
    compute: "STD",
    region: "local · main",
    createdAt: "12 Mar 26 09:14:02",
    refId: "proj_polter_01",
  },
  {
    projectId: "proj-sparky",
    name: "Sparky",
    directory: "~/projects/sparky",
    environmentSummary: "Development",
    isCurrent: false,
    status: "active",
    compute: "DEV",
    region: "local · dev",
    createdAt: "02 Feb 26 18:22:11",
    refId: "proj_sparky_02",
  },
  {
    projectId: "proj-converge",
    name: "Converge",
    directory: "~/projects/converge",
    environmentSummary: "Production",
    isCurrent: false,
    status: "paused",
    compute: "STD",
    region: "local · main",
    createdAt: "28 Jan 26 11:03:44",
    refId: "proj_converge_03",
  },
  {
    projectId: "proj-local",
    name: "Local playground",
    directory: "~/projects/playground",
    environmentSummary: "Local only",
    isCurrent: false,
    status: "active",
    compute: "NANO",
    region: "local",
    createdAt: "20 Apr 26 13:46:40",
    refId: "proj_local_04",
  },
];

export const ENVIRONMENT_FIXTURES: EnvironmentItem[] = [
  { id: "env-polter-prod", name: "main", kind: "production", projectId: "proj-polter" },
  { id: "env-polter-staging", name: "staging", kind: "staging", projectId: "proj-polter" },
  { id: "env-polter-dev", name: "dev", kind: "development", projectId: "proj-polter" },
  { id: "env-sparky-prod", name: "main", kind: "production", projectId: "proj-sparky" },
  { id: "env-sparky-dev", name: "dev", kind: "development", projectId: "proj-sparky" },
  { id: "env-converge-prod", name: "main", kind: "production", projectId: "proj-converge" },
  { id: "env-local", name: "local", kind: "local", projectId: "proj-local" },
];

/** Ids/status align with Resources NODE_FIXTURES where the same target is shared. */
export const TERMINAL_TARGET_FIXTURES: TerminalTargetItem[] = [
  { id: "node-nitro", name: "nitro", shell: "bash", status: "online", environmentId: "env-polter-prod" },
  { id: "node-prod-vps", name: "prod-vps-01", shell: "bash", status: "degraded", environmentId: "env-polter-prod" },
  { id: "node-local", name: "local", shell: "zsh", status: "online", environmentId: "env-local" },
  { id: "node-worker-01", name: "worker-node-01", shell: "bash", status: "online", environmentId: "env-polter-prod" },
  /** Unusable fixtures — picker must block open with honest feedback. */
  { id: "node-offline-01", name: "offline-edge-01", shell: "bash", status: "offline", environmentId: "env-polter-prod" },
  { id: "node-revoked-01", name: "revoked-node-01", shell: "bash", status: "revoked", environmentId: "env-polter-prod" },
];

export const TERMINAL_OUTPUT_FIXTURES = {
  "node-nitro": [
    "nitro $ docker ps",
    "CONTAINER ID   IMAGE            STATUS",
    "83f1a92d       postgres:17      Up 4 hours",
    "92ac31e8       redis:8          Up 4 hours",
    "nitro $ _",
  ],
  "node-prod-vps": [
    "prod-vps-01 $ uptime",
    " 18:04:12 up 42 days,  3:11,  load average: 0.12, 0.18, 0.15",
    "prod-vps-01 $ _",
  ],
  "node-local": [
    "local $ git status --short",
    " M packages/features/shell/src/fixtures.ts",
    "local $ _",
  ],
  "node-worker-01": [
    "worker-node-01 $ uptime",
    " 14:12:40 up 18 days, 2:04, load average: 0.08, 0.12, 0.10",
    "worker-node-01 $ _",
  ],
} satisfies Record<string, string[]>;
