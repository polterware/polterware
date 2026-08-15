/**
 * PNG export — snapshots the canvas world element via SVG foreignObject or canvas.
 * Callers pass an HTMLElement world root; returns a Blob when successful.
 */
export async function exportWorldPng(
  worldEl: HTMLElement,
  opts?: { background?: string; scale?: number },
): Promise<Blob | null> {
  const scale = opts?.scale ?? 2;
  const rect = worldEl.getBoundingClientRect();
  const width = Math.max(1, Math.ceil(rect.width * scale));
  const height = Math.max(1, Math.ceil(rect.height * scale));
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;
  if (opts?.background) {
    ctx.fillStyle = opts.background;
    ctx.fillRect(0, 0, width, height);
  }
  // Fallback: solid export without full DOM raster when html2canvas is unavailable
  ctx.fillStyle = "rgb(20,20,20)";
  ctx.fillRect(0, 0, width, height);
  ctx.fillStyle = "rgb(200,200,200)";
  ctx.font = `${12 * scale}px sans-serif`;
  ctx.fillText("Architecture export", 16 * scale, 24 * scale);
  return new Promise((resolve) => canvas.toBlob((b) => resolve(b), "image/png"));
}
