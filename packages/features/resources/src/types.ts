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
