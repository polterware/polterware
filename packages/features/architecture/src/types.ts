export type ArchitectureLayer = "cloud" | "applications" | "data" | "other";

export type ConnectionRef = {
  nodeId: string;
  label: string;
  relation: string;
  evidence: boolean;
};

export type ActivityItem = {
  id: string;
  at: string;
  title: string;
  detail?: string;
};

export type DocLink = {
  id: string;
  title: string;
  href?: string;
};

export type ArchitectureNodeView = {
  id: string;
  label: string;
  kind: string;
  layer: ArchitectureLayer;
  role?: string;
  environment?: string;
  runtime?: string;
  owner?: string;
  /** Lifecycle label: Planned, Linked, Observed, … (honest — not fake health). */
  lifecycle?: string;
  linkStatus?: "unlinked" | "linked" | "partial";
  status?: "healthy" | "degraded" | "down" | "unknown" | "syncing";
  resourceId?: string;
  resourceName?: string;
  /** True when operator can link an inventory resource from detail. */
  canLinkResource?: boolean;
  connected: ConnectionRef[];
  activity: ActivityItem[];
  notes?: string;
  docs: DocLink[];
  x: number;
  y: number;
  dataSource: "fixture";
};
