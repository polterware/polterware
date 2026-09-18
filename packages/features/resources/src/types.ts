export type ResourceStatus = "healthy" | "degraded" | "offline" | "unknown";

export type Resource = {
  id: string;
  name: string;
  type: string;
  provider: string;
  environment: string;
  status: ResourceStatus;
  region: string;
  address?: string;
  tags: string[];
  description: string;
  dataSource: "fixture" | "core";
};

export type NodeFixture = {
  id: string;
  name: string;
  platform: string;
  environment: string;
  status: ResourceStatus;
  address: string;
  cpu: string;
  memory: string;
  lastSeen: string;
};

export type ProcessFixture = {
  id: string;
  name: string;
  nodeId: string;
  nodeName: string;
  status: "running" | "stopped" | "failed";
  pid?: number;
  command: string;
  startedAt: string;
};
