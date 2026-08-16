import type { CanvasModel } from "@polterware/frontend-canvas";
import type { ArchitectureNodeView } from "./types";

export const ARCH_CANVAS_FIXTURE: CanvasModel = {
  revision: 1,
  selection: [],
  nodes: [
    { id: "n-gha", x: 320, y: 40, label: "GitHub Actions", kind: "resource", source: "github" },
    { id: "n-vercel", x: 80, y: 160, label: "Vercel Web", kind: "resource", source: "vercel" },
    { id: "n-gw", x: 280, y: 160, label: "API Gateway", kind: "resource", source: "aws" },
    { id: "n-auth", x: 480, y: 120, label: "Auth Service", kind: "resource", source: "aws" },
    { id: "n-pay", x: 480, y: 200, label: "Payment Service", kind: "resource", source: "aws" },
    { id: "n-worker", x: 480, y: 280, label: "Worker", kind: "resource", source: "aws" },
    { id: "n-pg", x: 160, y: 360, label: "PostgreSQL", kind: "resource", source: "aws" },
    { id: "n-redis", x: 320, y: 360, label: "Redis", kind: "resource", source: "aws" },
    { id: "n-queue", x: 480, y: 360, label: "Event Queue", kind: "resource", source: "aws" },
  ],
  edges: [
    { id: "e1", from: "n-gha", to: "n-gw", kind: "related", evidence: false },
    { id: "e2", from: "n-vercel", to: "n-gw", kind: "related", evidence: true },
    { id: "e3", from: "n-gw", to: "n-auth", kind: "related", evidence: true },
    { id: "e4", from: "n-gw", to: "n-pay", kind: "related", evidence: true },
    { id: "e5", from: "n-gw", to: "n-worker", kind: "related", evidence: false },
    { id: "e6", from: "n-gw", to: "n-pg", kind: "related", evidence: true },
    { id: "e7", from: "n-gw", to: "n-redis", kind: "related", evidence: true },
    { id: "e8", from: "n-gw", to: "n-queue", kind: "related", evidence: true },
  ],
};

export const NODE_DETAIL_FIXTURES = {
  "n-pay": {
    id: "n-pay",
    label: "Payment Service",
    kind: "Service",
    layer: "applications",
    role: "Payment processing orchestration",
    environment: "Production",
    runtime: "Node.js service",
    owner: "Platform",
    lifecycle: "Observed",
    linkStatus: "linked",
    canLinkResource: false,
    status: "healthy",
    resourceId: "res-payment",
    resourceName: "payment-service",
    x: 480,
    y: 200,
    dataSource: "fixture",
    connected: [
      { nodeId: "n-gw", label: "API Gateway", relation: "Upstream", evidence: true },
      { nodeId: "n-auth", label: "Auth Service", relation: "Dependency", evidence: true },
      { nodeId: "n-worker", label: "Worker", relation: "Async consumer", evidence: false },
      { nodeId: "n-queue", label: "Event Queue", relation: "Queue", evidence: true },
      { nodeId: "n-pg", label: "PostgreSQL", relation: "Database", evidence: true },
      { nodeId: "n-redis", label: "Redis", relation: "Cache", evidence: true },
    ],
    activity: [
      { id: "1", at: "14m ago", title: "Config synced", detail: "Configuration was synchronized successfully" },
      { id: "2", at: "17m ago", title: "Health check passed", detail: "All readiness and liveness checks are healthy" },
    ],
    notes: "Handles payment creation, capture, and settlement orchestration.",
    docs: [
      { id: "d1", title: "Runbook" },
      { id: "d2", title: "Service contract" },
    ],
  },
  "n-planned": {
    id: "n-planned",
    label: "Settlement Worker",
    kind: "Worker",
    layer: "applications",
    role: "Async settlement processing",
    environment: "Staging",
    runtime: "Container job",
    owner: "Platform",
    lifecycle: "Planned",
    linkStatus: "unlinked",
    canLinkResource: true,
    status: "unknown",
    x: 100,
    y: 100,
    dataSource: "fixture",
    connected: [],
    activity: [],
    docs: [],
    notes: "Placed on the map; not linked to inventory yet.",
  },
} satisfies Record<string, ArchitectureNodeView>;

export function fixtureNodeDetail(id: string, label?: string): ArchitectureNodeView {
  const fixture = Object.entries(NODE_DETAIL_FIXTURES).find(([key]) => key === id)?.[1];
  if (fixture) return fixture;
  return {
    id,
    label: label ?? id,
    kind: "Service",
    layer: "applications",
    lifecycle: "Unknown",
    linkStatus: "unlinked",
    canLinkResource: true,
    status: "unknown",
    environment: "Production",
    connected: [],
    activity: [],
    docs: [],
    x: 0,
    y: 0,
    dataSource: "fixture",
  };
}
