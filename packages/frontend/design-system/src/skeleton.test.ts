import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = dirname(fileURLToPath(import.meta.url));
const skeletonSource = readFileSync(resolve(root, "skeleton.tsx"), "utf8");
const skeletonCss = readFileSync(resolve(root, "skeleton.css"), "utf8");
const feedbackSource = readFileSync(resolve(root, "feedback.tsx"), "utf8");
const indexSource = readFileSync(resolve(root, "index.ts"), "utf8");

describe("Skeleton design-system contract", () => {
  it("exports Skeleton and PageSkeleton from the package entry", () => {
    expect(indexSource).toContain('export { Skeleton, PageSkeleton } from "./skeleton"');
    expect(indexSource).toContain("PageSkeletonLayout");
  });

  it("PageLoading delegates to PageSkeleton", () => {
    expect(feedbackSource).toContain("PageSkeleton");
    expect(feedbackSource).toContain('layout={props.layout ?? "page"}');
  });

  it("ships opaque pulse bones and reduced-motion static fill", () => {
    expect(skeletonSource).toContain("PageSkeletonLayout");
    expect(skeletonSource).toContain('"table"');
    expect(skeletonSource).toContain('"canvas"');
    expect(skeletonSource).toContain('"detail"');
    expect(skeletonSource).toContain('"page"');
    expect(skeletonSource).toContain("sr-only");
    expect(skeletonCss).toContain("@keyframes pw-skeleton-pulse");
    expect(skeletonCss).toContain("--color-surface-raised");
    expect(skeletonCss).toContain("--color-surface-selected");
    expect(skeletonCss).toContain("@media (prefers-reduced-motion: reduce)");
    expect(skeletonCss).not.toMatch(/rgb\(.*\/\s*0\./);
  });
});
