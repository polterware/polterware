import { defineConfig } from "../../../../../apps/gallery/node_modules/vite/dist/node/index.js";
import solid from "../../../../../apps/gallery/node_modules/vite-plugin-solid/dist/esm/index.mjs";
import tailwindcss from "../../../../../apps/gallery/node_modules/@tailwindcss/vite/dist/index.mjs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL(".", import.meta.url));
const repo = fileURLToPath(new URL("../../../../..", import.meta.url));
const pkg = (rel) => `${repo}/${rel}`;

export default defineConfig({
  root,
  plugins: [solid({ include: [/\.(tsx|jsx)$/] }), tailwindcss()],
  resolve: {
    alias: {
      "solid-js": pkg("apps/gallery/node_modules/solid-js"),
      "@polterware/frontend-design-system": pkg("packages/frontend/design-system/src/index.ts"),
      "@polterware/frontend-platform": pkg("packages/frontend/platform/src/index.ts"),
      "@polterware/frontend-state": pkg("packages/frontend/state/src/index.ts"),
      "@polterware/frontend-shell": pkg("packages/frontend/shell/src/index.ts"),
      "@polterware/feature-shell": pkg("packages/features/shell/src/index.tsx"),
      "@polterware/feature-sparky-shell": pkg("packages/features/sparky-shell/src/index.tsx"),
      "@polterware/sparky-contracts": pkg("packages/sparky-contracts/src/index.ts"),
      "@fontsource-variable/inter/opsz.css": pkg("apps/gallery/node_modules/@fontsource-variable/inter/opsz.css"),
    },
    dedupe: ["solid-js"],
    conditions: ["solid", "browser", "development", "import"],
  },
  server: {
    host: "127.0.0.1",
    port: 5179,
    fs: { allow: [repo] },
  },
});
