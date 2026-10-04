import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "electron-vite";
import solid from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";

const rootDir = dirname(fileURLToPath(import.meta.url));
const tailwindBase = resolve(rootDir, "../../packages/configuration/tailwind/base.css");

export default defineConfig({
  main: {
    build: {
      outDir: "out/main",
      rollupOptions: {
        input: { index: resolve(rootDir, "src/main/index.ts") },
      },
    },
  },
  preload: {
    build: {
      outDir: "out/preload",
      rollupOptions: {
        input: { index: resolve(rootDir, "src/preload/index.ts") },
        output: { entryFileNames: "[name].cjs", format: "cjs" },
      },
    },
  },
  renderer: {
    root: resolve(rootDir, "src/renderer"),
    plugins: [solid(), tailwindcss()],
    resolve: {
      dedupe: ["solid-js", "solid-js/web"],
      alias: {
        "@": resolve(rootDir, "src/renderer"),
        "@tailwind-base": tailwindBase,
      },
    },
    esbuild: {
      jsx: "automatic",
      jsxImportSource: "solid-js",
    },
    optimizeDeps: {
      include: ["solid-js", "solid-js/web", "maplibre-gl", "leaflet"],
    },
    build: {
      outDir: resolve(rootDir, "out/renderer"),
      emptyOutDir: true,
      rollupOptions: {
        input: { index: resolve(rootDir, "src/renderer/index.html") },
      },
    },
  },
});
