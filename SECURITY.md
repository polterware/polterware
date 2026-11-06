# Security

## Reporting a vulnerability

Use [GitHub private vulnerability reporting](https://github.com/polterware/polterware/security/advisories/new). Do not open a public issue or discussion for a suspected vulnerability.

Include the affected version or commit, impact, reproduction steps, and any suggested mitigation. Do not include credentials, personal data, customer data, or production secrets.

## Current boundary

The Desktop is a native GPUI application (Rust) backed by the local `polterware` core and SQLite. It renders no web content and loads no remote code: it is a client of the core over Connect-JSON on loopback, and it never opens SQLite.

The Desktop finds the core through `<db dir>/run/core.json`, which holds its address and session token, or starts it detached (`polterware serve`). The core binds to loopback, requires a technical bearer session token except for health, and owns local persistence, operations, workflows, processes, terminal sessions, and integration probes. Self-hosted and Cloud functionality is not implemented today.

Any future Cloud request must validate authentication, organization membership, permission, tenant, plan, entitlement, and billing state on the server.

Terminal sessions execute through a core-owned local PTY whose output is forwarded to the Desktop. Operation and workflow steps execute through the core runner. Integration setup may inspect supported CLI availability, start a fixed provider authentication command in the core-owned PTY, optionally open the provider documentation URL in the default browser, and probe connection state. Integration records have no provider-token input or storage field and do not deliberately persist provider credentials in SQLite. Raw CLI or shell output may contain sensitive material and is not guaranteed to be redacted.

## Go core

The core exposes only retained local domain services and requires a bearer session token for every request except health. The token is technical process authentication; it is not an account or person model.

The core must bind to loopback by default, store only the local domain baseline, validate request sizes and identifiers, and avoid deliberately logging its bearer session token. The token never travels in process arguments: clients read it from `core.json`, which only the user can read, and wait for health before sending requests.

The current PTY buffer is in memory and is not size-bounded. Operation logs persist command previews and truncated stdout/stderr without secret-aware redaction. Users and product flows must avoid placing secrets on command lines or in operation output; bounded buffers and structured redaction require a separate security specification before stronger guarantees are documented.

## Supported versions

The project is pre-release. Security fixes are applied to the current default branch; no historical release line is currently supported.
