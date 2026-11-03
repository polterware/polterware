// Legacy transitional API
export {
  type CanvasNode,
  type CanvasEdge,
  type CanvasModel,
  type HistoryStack,
  createEmptyCanvas,
  addNode,
  addEdge,
  selectNodes,
  selectedNode,
  serialize,
  deserialize,
  buildOperationalMap,
  pushHistory,
  undo,
} from "./legacy";

// Document v1
export type {
  ArchitectureDocument,
  ArchNode,
  Relationship,
  Boundary,
  Viewport,
  CanvasPreferences,
  SelectionState,
  BoundaryKind,
  NodeType,
  NodeState,
  RelationshipOrigin,
  RelationshipType,
  Port,
  Point,
  Rect,
  Size,
} from "./document/types";
export {
  MIN_ZOOM,
  MAX_ZOOM,
  DEFAULT_NODE_W,
  DEFAULT_NODE_H,
  MAX_BODY_BYTES,
} from "./document/types";
export {
  createEmptyDocument,
  validateDocument,
  defaultViewport,
  defaultNodeSize,
  touchUpdated,
  newId,
  type ValidationIssue,
} from "./document/document";
export { migrateLegacyToV1, isArchitectureDocumentV1 } from "./document/migrate";
export {
  serializeDocument,
  deserializeAndMigrate,
  validateImportJson,
  sanitizeText,
  type ParseResult,
} from "./document/serialize";
export type { DomainAdapter } from "./document/domain-adapter";
export {
  addNode as docAddNode,
  updateNode,
  moveNodes,
  setNodePositions,
  addRelationship,
  updateRelationship,
  deleteSelection,
  addBoundary,
  moveBoundaryWithMembers,
  documentBounds,
  emptySelection as emptyDocSelection,
} from "./document/mutations";
export {
  alignNodes,
  distributeNodes,
  reverseRelationship,
  type AlignMode,
  type DistributeMode,
} from "./document/align";

// Viewport
export {
  clampZoom,
  screenToDocument,
  documentToScreen,
  panBy,
  zoomAt,
  fitBounds,
  centerOn,
  visibleDocumentRect,
  rectsIntersect,
  rectContains,
} from "./viewport/viewport";

// History
export {
  createHistory,
  commit,
  undo as historyUndo,
  redo as historyRedo,
  canUndo,
  canRedo,
  type CaptureMode,
  type HistoryState,
  type DocPatch,
} from "./history/history";

// Selection
export {
  emptySelection,
  selectClick,
  selectMarquee,
  marqueeModeFromDrag,
  selectAll,
  selectionCount,
  type MarqueeMode,
} from "./selection/selection";

// Clipboard
export {
  buildFragment,
  pasteFragment,
  serializeFragment,
  parseFragment,
  CLIPBOARD_MIME,
  type DocumentFragment,
} from "./clipboard/clipboard";

// Commands
export {
  createCommandRegistry,
  registerCommand,
  dispatch,
  matchShortcut,
  listCommands,
  type Command,
  type CommandContext,
  type CommandRegistry,
} from "./commands/registry";

// Hit test
export {
  hitTest,
  isNodeConnectable,
  type Hit,
  type HitKind,
} from "./hit-test/hit-test";

// Spatial
export {
  createGridIndex,
  clearIndex,
  upsertEntry,
  queryRect,
  type GridIndex,
  type IndexEntry,
} from "./spatial/grid-index";

// Render helpers
export {
  orthogonalPath,
  resolveAutomaticConnection,
  routeOrthogonalLine,
  resolveRelationshipRoute,
  moveRouteSegment,
  removeRouteVertex,
  cleanupRoutePoints,
  nearestSegmentIndex,
  pointsToSvgPath,
  hitTestPolyline,
  polylineMidpoint,
  straightPath,
  bezierPath,
  pointsToRoundedSvgPath,
  pointNearSegment,
  edgeHitPadding,
  type ConnectionAnchor,
  type ConnectionSide,
  type OrthogonalRoute,
} from "./render/edge-path";

// Snap
export { snapToGrid, snapDelta, objectSnapGuides, type GuideLine } from "./snap/snap";

// Layout
export { layoutSync, type LayoutRequest, type LayoutResponse, type LayoutNode } from "./layout-algo";
