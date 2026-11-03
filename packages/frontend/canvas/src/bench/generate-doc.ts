import type { ArchitectureDocument } from "../document/types";
import { createEmptyDocument } from "../document/document";
import { DEFAULT_NODE_H, DEFAULT_NODE_W } from "../document/types";

/** Generate a synthetic document for performance benchmarks. */
export function generateDoc(nodeCount: number, edgeCount: number): ArchitectureDocument {
  const doc = createEmptyDocument({ systemId: "bench" });
  const cols = Math.ceil(Math.sqrt(nodeCount));
  for (let i = 0; i < nodeCount; i++) {
    const col = i % cols;
    const row = Math.floor(i / cols);
    doc.nodes.push({
      id: `n${i}`,
      name: `Node ${i}`,
      type: i % 5 === 0 ? "database" : "service",
      origin: "manual",
      state: "unknown",
      x: col * 180,
      y: row * 100,
      width: DEFAULT_NODE_W,
      height: DEFAULT_NODE_H,
      zIndex: i,
    });
  }
  for (let i = 0; i < edgeCount; i++) {
    const a = i % nodeCount;
    const b = (i * 7 + 1) % nodeCount;
    if (a === b) continue;
    doc.relationships.push({
      id: `r${i}`,
      sourceNodeId: `n${a}`,
      targetNodeId: `n${b}`,
      type: "calls",
      direction: "forward",
      evidence: i % 3 === 0,
    });
  }
  return doc;
}
