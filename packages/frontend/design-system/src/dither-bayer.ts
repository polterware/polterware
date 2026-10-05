/** Ordered Bayer 4×4 thresholds in (0, 1). Shared by icon tiles and chart fills. */
export const BAYER_4X4: readonly (readonly number[])[] = [
  [0, 8, 2, 10],
  [12, 4, 14, 6],
  [3, 11, 1, 9],
  [15, 7, 13, 5],
].map((row) => row.map((value) => (value + 0.5) / 16));

export function bayerThreshold(x: number, y: number): number {
  return BAYER_4X4[y & 3]?.[x & 3] ?? 0.5;
}
