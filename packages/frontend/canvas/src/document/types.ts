/** Architecture / generic canvas document types (formatVersion 1). */

export type NodeType =
  | "system"
  | "web_application"
  | "mobile_application"
  | "desktop_application"
  | "api"
  | "service"
  | "worker"
  | "scheduled_job"
  | "serverless_function"
  | "database"
  | "cache"
  | "queue"
  | "event_stream"
  | "storage"
  | "search_engine"
  | "data_warehouse"
  | "repository"
  | "package"
  | "ci_pipeline"
  | "build_system"
  | "artifact_registry"
  | "runtime_host"
  | "load_balancer"
  | "api_gateway"
  | "cdn"
  | "network"
  | "dns"
  | "region"
  | "external_service"
  | "saas_provider"
  | "third_party_api"
  | "auth_provider"
  | "payment_provider"
  | "integration"
  | "custom_resource"
  | "note"
  /** Unknown / future catalog ids remain loadable */
  | (string & {});

export type NodeState =
  | "planned"
  | "linked"
  | "observed"
  | "stale"
  | "drifted"
  | "unreachable"
  | "unknown";

export type RelationshipType =
  | "depends_on"
  | "calls"
  | "reads_from"
  | "writes_to"
  | "publishes_to"
  | "consumes_from"
  | "authenticates_with"
  | "deploys_to"
  | "stores_in"
  | "monitors"
  | "routes_to"
  | "hosted_on"
  | "replicates_to"
  | "synchronizes_with"
  | "exposes"
  | "contains"
  | "related"
  | "custom"
  | "deployed_on"
  | "connects_to"
  | (string & {});

export type PortDirection = "in" | "out" | "both";
export type PortSide = "left" | "right" | "top" | "bottom" | "auto";

export type Port = {
  id: string;
  direction: PortDirection;
  side: PortSide;
  label?: string;
  maxConnections?: number | null;
};

export type DocLink = { label: string; href: string };

export type ArchNode = {
  id: string;
  name: string;
  description?: string;
  type: NodeType;
  subtype?: string;
  origin: "manual" | "discovered";
  provider?: string;
  environmentIds?: string[];
  state: NodeState;
  x: number;
  y: number;
  width: number;
  height: number;
  zIndex: number;
  boundaryId?: string | null;
  collapsed?: boolean;
  linkStatus?: "unlinked" | "linked" | "partial";
  resourceId?: string | null;
  externalKey?: string | null;
  owner?: string;
  tags?: string[];
  links?: DocLink[];
  locked?: boolean;
  layoutPinned?: boolean;
  ports?: Port[];
  metadata?: Record<string, string | number | boolean>;
};

export type RelationshipOrigin = "manual" | "discovered" | "imported" | "inferred";

export type Relationship = {
  id: string;
  sourceNodeId: string;
  targetNodeId: string;
  sourcePortId?: string;
  targetPortId?: string;
  type: RelationshipType;
  label?: string;
  direction: "forward" | "bidirectional";
  origin?: RelationshipOrigin;
  protocol?: string;
  state?: string;
  evidence?: boolean;
  style?: "default" | "emphasis" | "muted";
  zIndex?: number;
  manualPoints?: Point[];
  metadata?: Record<string, string | number | boolean>;
};

export type BoundaryKind =
  | "system"
  | "domain"
  | "context"
  | "team"
  | "deployment"
  | "region"
  | "custom";

export type Boundary = {
  id: string;
  name: string;
  kind: BoundaryKind;
  x: number;
  y: number;
  width: number;
  height: number;
  zIndex: number;
  collapsed?: boolean;
  locked?: boolean;
  memberNodeIds?: string[];
  colorToken?: string;
};

export type Viewport = {
  x: number;
  y: number;
  zoom: number;
  focusedElementId?: string | null;
};

export type CanvasPreferences = {
  gridEnabled?: boolean;
  snapToGrid?: boolean;
  snapToObjects?: boolean;
  gridSize?: number;
  showMinimap?: boolean;
  edgeRouting?: "orthogonal" | "bezier" | "straight";
};

export type ArchitectureDocument = {
  formatVersion: 1;
  id: string;
  systemId: string;
  workspaceId?: string;
  activeEnvironmentId?: string | "all";
  nodes: ArchNode[];
  relationships: Relationship[];
  boundaries: Boundary[];
  preferences: CanvasPreferences;
  viewport: Viewport;
  createdAt?: string;
  updatedAt?: string;
  meta?: { migratedFrom?: number; generator?: string };
};

export type Point = { x: number; y: number };
export type Size = { width: number; height: number };
export type Rect = { x: number; y: number; width: number; height: number };

export type SelectionState = {
  nodeIds: string[];
  relationshipIds: string[];
  boundaryIds: string[];
};

/** Floor: enough overview for a full system map without losing the graph. */
export const MIN_ZOOM = 0.25;
/** Ceiling: close-up for dense cards; still short of “pixel peeping”. */
export const MAX_ZOOM = 2.5;
export const DEFAULT_NODE_W = 188;
export const DEFAULT_NODE_H = 56;
export const MAX_BODY_BYTES = 10 * 1024 * 1024;
