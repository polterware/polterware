import type { DitherIconTone } from "@polterware/frontend-design-system";
import type { HostNodeDraft } from "./create-host-node-dialog";
import type { ProcessDraft } from "./create-process-dialog";

export type ResourceInventoryDraft = {
  name: string;
  type: string;
};

export type ResourceInventoryTemplate = {
  id: string;
  title: string;
  description: string;
  tone: DitherIconTone;
  brand?: string;
  draft: ResourceInventoryDraft;
};

export type ResourceNodeTemplate = {
  id: string;
  title: string;
  description: string;
  tone: DitherIconTone;
  draft: HostNodeDraft;
};

export type ResourceProcessTemplate = {
  id: string;
  title: string;
  description: string;
  tone: DitherIconTone;
  draft: ProcessDraft;
};

export const RESOURCE_INVENTORY_TEMPLATES: ResourceInventoryTemplate[] = [
  {
    id: "inventory-api-gateway",
    title: "API gateway",
    description: "Edge service that routes and authenticates traffic.",
    tone: "architecture",
    draft: { name: "API gateway", type: "Service" },
  },
  {
    id: "inventory-postgres",
    title: "Postgres",
    description: "Primary relational database for the project.",
    tone: "data",
    brand: "postgres",
    draft: { name: "Postgres", type: "Database" },
  },
  {
    id: "inventory-redis",
    title: "Redis cache",
    description: "Shared cache and short-lived session store.",
    tone: "data",
    brand: "redis",
    draft: { name: "Redis cache", type: "Cache" },
  },
];

export const RESOURCE_NODE_TEMPLATES: ResourceNodeTemplate[] = [
  {
    id: "node-ssh-host",
    title: "SSH host",
    description: "Generic remote host opened with system ssh.",
    tone: "infrastructure",
    draft: {
      name: "SSH host",
      connectionKind: "ssh",
      address: "",
      sshUser: "ubuntu",
      sshPort: 22,
      sshIdentityFile: "",
      platform: "linux",
      shell: "bash",
    },
  },
  {
    id: "node-vps",
    title: "VPS",
    description: "Cloud VPS profile for deploy and shell access.",
    tone: "infrastructure",
    draft: {
      name: "VPS",
      connectionKind: "ssh",
      address: "",
      sshUser: "root",
      sshPort: 22,
      sshIdentityFile: "",
      platform: "linux",
      shell: "bash",
    },
  },
  {
    id: "node-jump-box",
    title: "Jump box",
    description: "Bastion host used as an SSH entry point.",
    tone: "execution",
    draft: {
      name: "Jump box",
      connectionKind: "ssh",
      address: "",
      sshUser: "ubuntu",
      sshPort: 22,
      sshIdentityFile: "",
      platform: "linux",
      shell: "bash",
    },
  },
];

export const RESOURCE_PROCESS_TEMPLATES: ResourceProcessTemplate[] = [
  {
    id: "process-sleep-probe",
    title: "Sleep probe",
    description: "Safe short-lived process to verify local execution.",
    tone: "execution",
    draft: { name: "Sleep probe", command: "sleep 60", cwd: "" },
  },
  {
    id: "process-echo-ok",
    title: "Echo check",
    description: "Print a quick confirmation and exit.",
    tone: "execution",
    draft: { name: "Echo check", command: "echo ok", cwd: "" },
  },
  {
    id: "process-git-status",
    title: "Git status",
    description: "Inspect the working tree in the current directory.",
    tone: "integration",
    draft: { name: "Git status", command: "git status", cwd: "" },
  },
];
