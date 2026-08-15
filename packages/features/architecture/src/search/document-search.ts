import type { ArchitectureDocument, ArchNode } from "@polterware/frontend-canvas";

export type SearchHit = {
  id: string;
  name: string;
  type: string;
  score: number;
  context: string;
};

export function searchDocument(doc: ArchitectureDocument, query: string): SearchHit[] {
  const q = query.trim().toLowerCase();
  if (!q) return [];
  const hits: SearchHit[] = [];
  for (const n of doc.nodes) {
    const score = scoreNode(n, q);
    if (score > 0) {
      hits.push({
        id: n.id,
        name: n.name,
        type: n.type,
        score,
        context: [n.type, n.provider, n.owner, ...(n.tags ?? [])].filter(Boolean).join(" · "),
      });
    }
  }
  return hits.sort((a, b) => b.score - a.score || a.name.localeCompare(b.name));
}

function scoreNode(n: ArchNode, q: string): number {
  let s = 0;
  if (n.name.toLowerCase() === q) s += 100;
  if (n.name.toLowerCase().includes(q)) s += 50;
  if (n.id.toLowerCase().includes(q)) s += 40;
  if (n.type.includes(q)) s += 30;
  if (n.provider?.toLowerCase().includes(q)) s += 20;
  if (n.owner?.toLowerCase().includes(q)) s += 15;
  if (n.externalKey?.toLowerCase().includes(q)) s += 25;
  if ((n.tags ?? []).some((t) => t.toLowerCase().includes(q))) s += 20;
  if ((n.environmentIds ?? []).some((e) => e.toLowerCase().includes(q))) s += 10;
  if (n.state.includes(q)) s += 10;
  return s;
}
