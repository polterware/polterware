import { defineConfig } from '../../../../../apps/gallery/node_modules/vite/dist/node/index.js';
import solid from '../../../../../apps/gallery/node_modules/vite-plugin-solid/dist/esm/index.mjs';
import tailwindcss from '../../../../../apps/gallery/node_modules/@tailwindcss/vite/dist/index.mjs';
import { fileURLToPath } from 'node:url';
const root = fileURLToPath(new URL('.', import.meta.url));
export default defineConfig({ root, plugins: [solid({ include: [/\.(tsx|jsx)$/] }), tailwindcss()], resolve: { alias: { vitest: root + 'vitest-stub.ts', 'solid-js': fileURLToPath(new URL('../../../../../apps/gallery/node_modules/solid-js', import.meta.url)) }, dedupe: ['solid-js'], conditions: ['solid', 'browser', 'development', 'import'] }, server: { host: '127.0.0.1', port: 5178, fs: { allow: [fileURLToPath(new URL('../../../../..', import.meta.url))] } }, publicDir: fileURLToPath(new URL('../../../../../apps/sparky-desktop/src/renderer/public', import.meta.url)) });
