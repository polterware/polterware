// @vitest-environment node

import { existsSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const TABLE_CONSUMERS = [
  "packages/features/architecture/src/views-table.tsx",
  "packages/features/integrations/src/index.tsx",
  "packages/features/knowledge/src/index.tsx",
  "packages/features/operations/src/index.tsx",
  "packages/features/resources/src/index.tsx",
];

const FEATURE_SELECTION_ACTIONS = [
  "packages/features/knowledge/src/selection-actions.ts",
  "packages/features/knowledge/src/selection-actions.test.ts",
  "packages/features/operations/src/selection-actions.ts",
  "packages/features/operations/src/selection-actions.test.ts",
  "packages/features/shell/src/selection-actions.ts",
  "packages/features/shell/src/selection-actions.test.ts",
];

describe("product table selection policy", () => {
  it("keeps checkbox selection and selection toolbars out of product table consumers", () => {
    for (const file of TABLE_CONSUMERS) {
      const source = readFileSync(file, "utf8");
      expect(source, file).not.toMatch(/DataTable\.(?:SelectionCell|SelectionToolbar)/);
      expect(source, file).not.toMatch(/\bselectionMode=/);
    }
  });

  it("does not host a product-level selection toolbar", () => {
    const source = readFileSync("packages/features/shell/src/main-surface.tsx", "utf8");
    expect(source).not.toContain("DataTableSelectionToolbarHost");
  });

  it("removes feature-specific batch selection action helpers", () => {
    for (const file of FEATURE_SELECTION_ACTIONS) {
      expect(existsSync(file), file).toBe(false);
    }
  });

  it("uses fill-only active and focus states without a left edge accent", () => {
    const css = readFileSync("packages/frontend/design-system/src/table.css", "utf8");
    expect(css).not.toContain("inset 2px 0 0");
    expect(css).toMatch(
      /\.pw-data-table-row:focus-visible > \.pw-data-table-cell[\s\S]*?background: var\(--color-surface-hover\)/,
    );
  });
});
