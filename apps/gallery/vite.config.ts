import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";
export default defineConfig({
  plugins: [solid({ extensions: [".tsx", ".jsx"], include: [/\.(tsx|jsx)$/] }), tailwindcss()],
  resolve: { dedupe: ["solid-js"], conditions: ["solid", "browser", "development", "import"] },
});
