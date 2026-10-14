# Troubleshooting

- Installation or build failure: use Node.js 24 or newer and run pnpm from the monorepo root.
- Missing shared styles: import the shared Tailwind base and keep the app's `@source` declaration; do not duplicate control chrome locally.
- A product URL fails on reload: configure the static host to serve directory `index.html` files.
- Unknown paths show the home page: remove the host's SPA rewrite and serve `404.html` with HTTP 404.
- Catalog changes are absent: rebuild the app before deploying its `.output/public` directory.

The web app has no environment configuration or contact database. The original independent checkout is not used at runtime.
