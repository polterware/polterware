# Getting Started

Install Node.js 24 or newer and the pnpm version declared by the workspace. From the monorepo root, run `pnpm install`, then `pnpm dev:web`. Open `http://127.0.0.1:5175`.

Use `pnpm typecheck:web` for TypeScript validation, `pnpm design:lint` for shared visual rules, and `pnpm build:web` followed by `pnpm preview:web` to inspect static production output.

Edit product summaries, status and install details in `apps/web/src/data/products.ts`. When adding a route, also add it to the prerender list in `apps/web/vite.config.ts`.
