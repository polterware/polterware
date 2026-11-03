# Security

## Reporting a vulnerability

Use [GitHub private vulnerability reporting](https://github.com/polterware/polterware/security/advisories/new). Do not open a public issue or discussion for a suspected vulnerability.

Include the affected version or commit, impact, reproduction steps, and any suggested mitigation. Do not include credentials, personal data, customer data, or production secrets.

## Current boundary

The Desktop is a local Electron application backed by the local `polterware` core and SQLite. The renderer receives a narrow typed preload bridge and has no unrestricted Node.js, Electron, filesystem, shell, PTY, credential-store, or core network access.

Electron Main starts and supervises the core, owns Connect clients, and mediates every renderer request. The core binds to loopback, requires a technical bearer session token except for health, and owns local persistence, operations, workflows, processes, terminal sessions, and integration probes. Self-hosted and Cloud functionality is not implemented today.

Any future Cloud request must validate authentication, organization membership, permission, tenant, plan, entitlement, and billing state on the server.

Terminal sessions execute through a core-owned local PTY whose output is forwarded to the renderer. Operation and workflow steps execute through the core runner. Integration setup may inspect supported CLI availability, start a fixed provider authentication command in the core-owned PTY, optionally open the provider documentation URL through Electron Main, and probe connection state. Integration records have no provider-token input or storage field and do not deliberately persist provider credentials in SQLite. Raw CLI or shell output may contain sensitive material and is not guaranteed to be redacted.

## Go core

The core exposes only retained local domain services and requires a bearer session token for every request except health. The token is technical process authentication; it is not an account or person model.

The core must bind to loopback by default, store only the local domain baseline, validate request sizes and identifiers, and avoid deliberately logging its bearer session token. Electron Main passes that token outside process arguments, waits for health before exposing requests, and terminates the child core when the application exits.

The current PTY buffer is in memory and is not size-bounded. Operation logs persist command previews and truncated stdout/stderr without secret-aware redaction. Users and product flows must avoid placing secrets on command lines or in operation output; bounded buffers and structured redaction require a separate security specification before stronger guarantees are documented.

## Electron requirements

- `contextIsolation` enabled
- sandbox enabled where supported
- Node integration disabled in renderer windows
- strict Content Security Policy
- navigation and external window creation denied unless explicitly handled
- no raw `ipcRenderer` exposure
- all IPC channels declared in the shared catalog

## Supported versions

The project is pre-release. Security fixes are applied to the current default branch; no historical release line is currently supported.
