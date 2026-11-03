import { defineConfig } from "vitest/config";
import solid from "vite-plugin-solid";

// Components are Solid JSX, so tests need the Solid transform and the browser build of solid-js.
export default defineConfig({
  plugins: [solid()],
  resolve: { conditions: ["development", "browser"] },
  test: {
    environment: "jsdom",
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
    setupFiles: ["./test/setup.ts"],
    server: { deps: { inline: [/solid-js/, /@ark-ui/, /@corvu/] } },
  },
});
