import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

const rootDir = dirname(fileURLToPath(import.meta.url));
const packagesDir = resolve(rootDir, "../../packages");

export default defineConfig({
  root: resolve(rootDir, "src/web"),
  server: { host: "127.0.0.1", port: 5176, strictPort: true },
  plugins: [
    solid({
      include: [
        `${resolve(rootDir, "src")}/**/*.{tsx,jsx}`,
        `${packagesDir}/frontend/**/*.{tsx,jsx}`,
        /packages[\\/]frontend[\\/].*\.tsx$/,
        /node_modules[\\/]\.pnpm[\\/].*@ark-ui\+solid@.*[\\/].*\.jsx$/,
        /node_modules[\\/]@ark-ui[\\/]solid[\\/].*\.jsx$/,
        /node_modules[\\/]\.pnpm[\\/].*@corvu\+.*[\\/].*\.jsx$/,
        /node_modules[\\/]@corvu[\\/].*\.jsx$/,
      ],
      extensions: [".tsx", ".jsx"],
    }),
    tailwindcss(),
  ],
  resolve: {
    dedupe: ["solid-js", "solid-js/web", "solid-js/store"],
    conditions: ["solid", "browser", "development|production", "import", "module", "default"],
    alias: {
      "@": resolve(rootDir, "src/renderer"),
      "@polterware/frontend-design-system": resolve(rootDir, "../../packages/frontend/design-system/src/index.ts"),
    },
    preserveSymlinks: false,
  },
  esbuild: {
    jsx: "automatic",
    jsxImportSource: "solid-js",
  },
  optimizeDeps: {
    include: ["solid-js", "solid-js/web", "solid-js/store"],
    exclude: ["@ark-ui/solid", "@corvu/resizable", "@corvu/utils"],
    esbuildOptions: {
      jsx: "automatic",
      jsxImportSource: "solid-js",
    },
  },
  build: {
    outDir: resolve(rootDir, "out/web"),
    emptyOutDir: true,
  },
});
