import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "electron-vite";
import solid from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";

const rootDir = dirname(fileURLToPath(import.meta.url));
const packagesDir = resolve(rootDir, "../../packages");

const sparkyFeatures = [
  "shell",
  "memories",
  "minds",
  "schedule",
  "calendar",
  "attachments",
  "focus",
  "review",
  "map",
  "settings",
  "search",
] as const;

const sparkyFeatureAliases = Object.fromEntries(
  sparkyFeatures.map((name) => [
    `@polterware/feature-sparky-${name}`,
    resolve(rootDir, `../../packages/features/sparky-${name}/src/index.tsx`),
  ]),
);

export default defineConfig({
  main: {
    resolve: {
      preserveSymlinks: false,
      alias: {
        "@polterware/sparky-contracts": resolve(
          rootDir,
          "../../packages/sparky-contracts/src/index.ts",
        ),
      },
    },
    build: {
      outDir: "out/main",
      externalizeDeps: {
        exclude: ["@polterware/sparky-contracts"],
      },
      rollupOptions: {
        input: { index: resolve(rootDir, "src/main/index.ts") },
      },
    },
  },
  preload: {
    resolve: { preserveSymlinks: false },
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
    plugins: [
      solid({
        include: [
          `${resolve(rootDir, "src/renderer")}/**/*.{tsx,jsx}`,
          `${packagesDir}/features/sparky-*/**/*.{tsx,jsx}`,
          `${packagesDir}/frontend/**/*.{tsx,jsx}`,
          /packages[\\/]features[\\/]sparky-[^\\/]+[\\/].*\.tsx$/,
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
        "@polterware/sparky-contracts": resolve(
          rootDir,
          "../../packages/sparky-contracts/src/index.ts",
        ),
        "@polterware/feature-sparky-shell/feature": resolve(
          rootDir,
          "../../packages/features/sparky-shell/src/feature.ts",
        ),
        ...sparkyFeatureAliases,
        "@polterware/frontend-design-system": resolve(
          rootDir,
          "../../packages/frontend/design-system/src/index.ts",
        ),
        "@polterware/frontend-map": resolve(
          rootDir,
          "../../packages/frontend/map/src/index.ts",
        ),
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
    ssr: {
      noExternal: ["@ark-ui/solid", "@corvu/resizable", "@corvu/utils"],
    },
    build: {
      outDir: resolve(rootDir, "out/renderer"),
      emptyOutDir: true,
      commonjsOptions: {
        transformMixedEsModules: true,
      },
      rollupOptions: {
        input: {
          index: resolve(rootDir, "src/renderer/index.html"),
        },
      },
    },
  },
});
