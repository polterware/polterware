import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const workspaceRoot = fileURLToPath(new URL("../../../../", import.meta.url));

const sharedMaterialFiles = [
  "packages/configuration/tailwind/base.css",
  "packages/frontend/design-system/src/materials.css",
];

const directConsumerFiles = [
  "packages/frontend/design-system/src/button.css",
  "packages/frontend/design-system/src/status.css",
  "packages/frontend/design-system/src/tabs.css",
  "packages/features/shell/src/main-surface.css",
];
const detailPanelFile = "packages/frontend/design-system/src/detail-panel.css";

const forbiddenInsetEdgeLight =
  /inset\s+0\s+(?:1px\s+0|0\s+0\s+1px)\s+var\(--(?:edge-light-(?:field|surface)-(?:highlight|rim)|texture-button-(?:highlight|edge|hover-highlight|hover-edge))\)/;
const forbiddenShadowEdgeLight =
  /(?:0\s+-1px\s+0|0\s+0\s+0\s+1px)\s+var\(--(?:edge-light-(?:field|surface)-(?:highlight|rim)|texture-button-(?:highlight|edge|hover-highlight|hover-edge))\)/;

function ruleBody(source: string, selector: string) {
  const escapedSelector = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = source.match(new RegExp(`${escapedSelector}\\s*\\{([^}]*)\\}`));
  expect(match, `Missing ${selector}`).not.toBeNull();
  return match?.[1].replace(/\s+/g, " ").trim();
}

function readWorkspaceFile(file: string) {
  return readFileSync(resolve(workspaceRoot, file), "utf8");
}

describe("edge-light policy", () => {
  it("uses real edge borders instead of inset or clipped outer shadows", () => {
    for (const file of [...sharedMaterialFiles, ...directConsumerFiles]) {
      const source = readWorkspaceFile(file);
      expect(source, file).not.toMatch(forbiddenInsetEdgeLight);
      expect(source, file).not.toMatch(forbiddenShadowEdgeLight);
    }
  });

  it("keeps the mirrored shared material recipes aligned", () => {
    const [base, materials] = sharedMaterialFiles.map((file) =>
      readWorkspaceFile(file),
    );
    const selectors = [
      ".edge-light-surface",
      ".edge-light-surface-top",
      ".surface-panel",
      ".surface-panel-float",
      ".control-field",
      ".menu-panel",
    ];

    for (const selector of selectors) {
      expect(ruleBody(base, selector), selector).toBe(
        ruleBody(materials, selector),
      );
    }
  });

  it("uses a crisp rim and top highlight at the component edge", () => {
    const materials = readWorkspaceFile(sharedMaterialFiles[1]);
    const button = readWorkspaceFile(directConsumerFiles[0]);

    expect(ruleBody(materials, ".surface-panel")).toContain(
      "border: 1px solid var(--edge-light-surface-rim)",
    );
    expect(ruleBody(materials, ".surface-panel")).toContain(
      "border-top-color: var(--edge-light-surface-highlight)",
    );
    expect(ruleBody(materials, ".control-field")).toContain(
      "border: 1px solid var(--edge-light-field-rim)",
    );
    // The rim rides the press layer, so the whole surface shrinks 1px when pressed.
    expect(ruleBody(button, ".texture-button-material::before")).toContain(
      "border: 1px solid var(--texture-button-edge)",
    );
  });

  it("keeps DetailPanel outside the shared real-border treatment", () => {
    const detailPanel = readWorkspaceFile(detailPanelFile);
    const surface = ruleBody(
      detailPanel,
      ".pw-detail-panel__surface.surface-panel-float",
    );

    expect(surface).toContain("border: 0");
    expect(detailPanel).not.toMatch(
      /\.pw-detail-panel__surface\.surface-panel-float::before\s*\{[^}]*border:\s*1px solid/,
    );
    expect(detailPanel).toMatch(
      /\.pw-detail-panel-root\[data-mode="maximized"\]\s+\.pw-detail-panel__surface\.surface-panel-float::before\s*\{[^}]*box-shadow:\s*none/,
    );
  });
});
