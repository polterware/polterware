# Security

## Current boundary

The Desktop is a fixture-only Electron application. The renderer receives a narrow typed preload bridge for local application chrome and Commander window controls. It has no unrestricted Node.js, Electron, filesystem, shell, PTY, privileged network bridge, or core API access.

Build and runtime configuration select the deployment composition. Backend capabilities select the available product surfaces. Backend authorization decides whether an operation is allowed. Only local fixture-backed behavior is active today; self-hosted and Cloud functionality is not implemented.

Any future Cloud request must validate authentication, organization membership, permission, tenant, plan, entitlement, and billing state on the server.

The terminal and integration setup surfaces are visual transcripts. They must not execute commands, open browsers, inspect installed CLIs, accept credentials, or persist secrets. Fixtures must contain synthetic values only.

## Go core

The retained core is a separate future-facing local runtime. It exposes only retained domain services and requires a bearer session token for every request except health. The token is technical process authentication; it is not an account or person model.

The core must bind to loopback by default, store only the reduced local domain baseline, validate request sizes and identifiers, and avoid logging bearer tokens or sensitive provider material. The Desktop must not spawn, supervise, discover, or call the core in this version.

## Electron requirements

- `contextIsolation` enabled
- sandbox enabled where supported
- Node integration disabled in renderer windows
- strict Content Security Policy
- navigation and external window creation denied unless explicitly handled
- no raw `ipcRenderer` exposure
- all IPC channels declared in the shared catalog

## Reporting

Do not open a public issue for a suspected vulnerability. Send a minimal reproduction and impact assessment to the repository maintainer through a private channel.
