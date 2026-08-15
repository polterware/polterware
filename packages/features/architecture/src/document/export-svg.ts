import type { ArchitectureDocument } from "@polterware/frontend-canvas";
import {
  documentBounds,
  pointsToRoundedSvgPath,
  resolveRelationshipRoute,
} from "@polterware/frontend-canvas";
import {
  relationshipMarkers,
  resolveRelationshipAppearance,
} from "../domain/relationship-appearance";

const SVG_COLORS = {
  background: "#0f0f0f",
  node: "#1a1a1a",
  border: "#2c2c2c",
  relationship: "#4b4b4b",
  relationshipDefault: "#6f6f6f",
  text: "#ececec",
} as const;

/** Honest SVG export of nodes (rects) + relationship paths. */
export function exportDocumentSvg(doc: ArchitectureDocument): string {
  const b = documentBounds(doc);
  const pad = 24;
  const w = Math.max(1, b.maxX - b.minX + pad * 2);
  const h = Math.max(1, b.maxY - b.minY + pad * 2);
  const ox = -b.minX + pad;
  const oy = -b.minY + pad;
  const parts: string[] = [
    `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">`,
    '<defs><marker id="arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="context-stroke"/></marker></defs>',
    `<rect width="100%" height="100%" fill="${SVG_COLORS.background}"/>`,
  ];
  for (const r of doc.relationships) {
    const a = doc.nodes.find((n) => n.id === r.sourceNodeId);
    const t = doc.nodes.find((n) => n.id === r.targetNodeId);
    if (!a || !t) continue;
    const d = pointsToRoundedSvgPath(
      resolveRelationshipRoute(
        { ...a, x: a.x + ox, y: a.y + oy },
        { ...t, x: t.x + ox, y: t.y + oy },
        r.manualPoints?.map((p) => ({ x: p.x + ox, y: p.y + oy })),
      ),
    );
    const appearance = resolveRelationshipAppearance(r);
    const markers = relationshipMarkers(r.direction);
    parts.push(
      `<path d="${d}" fill="none" stroke="${appearance.dim ? SVG_COLORS.relationship : SVG_COLORS.relationshipDefault}" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" marker-end="${markers.end}" ${markers.start ? `marker-start="${markers.start}"` : ""} />`,
    );
  }
  for (const n of doc.nodes) {
    parts.push(
      `<rect x="${n.x + ox}" y="${n.y + oy}" width="${n.width}" height="${n.height}" rx="8" fill="${SVG_COLORS.node}" stroke="${SVG_COLORS.border}"/>`,
      `<text x="${n.x + ox + 10}" y="${n.y + oy + 28}" fill="${SVG_COLORS.text}" font-size="12" font-family="system-ui">${escapeXml(n.name)}</text>`,
    );
  }
  parts.push("</svg>");
  return parts.join("\n");
}

function escapeXml(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
