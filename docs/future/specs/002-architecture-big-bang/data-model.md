# Data Model: Architecture Big Bang (Local Completion Set)

**Feature**: `002-architecture-big-bang`  
**Authority**: Local SQLite in polterware-core  
**Migration**: Greenfield DISCARD for legacy stores (see migration-map). No CONVERT this feature.

---

## Normative terminology (CHK039)

| Term | Definition | Domain home (completion) |
| --- | --- | --- |
| **Provisioning** | Creation or declarative configuration of an **external** resource by a provider | **DEFERRED** as separate domain; express via Operation + Integration today |
| **Operation** | Auditable unit of execution | `operations` |
| **Workflow** | Ordered composition of operations | `workflows` |
| **Integration** | Connection and capabilities of a provider (`execute` \| `status` \| `handoff`) | `integrations` |
| **Resource** | Real entity discovered, created, or managed | `resources` |
| **Command** | Concrete instruction executed by an operation | payload of Operation / process provider |

Do **not** create an empty `provisioning` package or domain.

---

## Conventions

- IDs: UUID strings  
- Timestamps: RFC3339 UTC  
- Validation: fail-closed writes (FR-065)  
- No cloud Organization/Membership/Team tables  
- Tenant isolation / organization isolation: **NOT APPLICABLE** this feature  
- Isolation required: workspace / system / environment / resource + human vs agent principals  

---

## Principal

| Field | Rules |
| --- | --- |
| id | required unique |
| kind | `human` \| `agent` |
| display_name | required |
| role | `owner` \| `operator` \| `viewer` \| `agent` |
| capabilities | optional explicit grants (string set) |
| created_at | required |

Default install: one human **Owner**. Agents always distinct ids; role `agent`.

---

## Workspace

Local operational context (single default per install for completion).

| Field | Rules |
| --- | --- |
| id | required |
| name | required |
| created_at | required |

---

## System

Domain root (UI may label “Project”).

| Field | Rules |
| --- | --- |
| id | required |
| workspace_id | FK Workspace |
| name | required |
| root_path | optional |
| created_at / updated_at | required |

---

## Environment

| Field | Rules |
| --- | --- |
| id | required |
| system_id | FK System |
| name | required |
| kind | local \| development \| staging \| production \| custom |

---

## Target

| Field | Rules |
| --- | --- |
| id | required |
| system_id | FK |
| environment_id | optional |
| name | required |
| kind | `local_machine` (completion); remote kinds deferred |
| status | multi-facet metadata |

---

## Resource

| Field | Rules |
| --- | --- |
| id | required |
| system_id | FK |
| environment_id | optional |
| name | required |
| type | extensible |
| external_key | unique per (system_id, source) when set |
| source | user \| discovery \| import |
| ownership_text | optional |
| status facets | lifecycle/link/management/health/drift |

---

## Relationship / Evidence / ArchitectureLayout / WorkspaceLayout

As prior plan: relationship lifecycle honesty; evidence observations; **user-authored layout revision** optimistic concurrency; dock layouts for operator panels.

---

## Integration

| Field | Rules |
| --- | --- |
| tool_id | git \| pkg \| process \| gh \| vercel \| supabase \| … |
| mode | execute \| status \| handoff |

---

## Operation

Governance fields FR-027; states queued → awaiting_approval → running → terminal; idempotency_key optional.

| Extra | Rules |
| --- | --- |
| actor_principal_id | required |
| actor_type | human \| agent |
| requested_capability | required string |
| workspace_id | required |
| policy_decision | allow/deny + reason + policy_id |

---

## Approval / Policy

Approval: pending \| approved \| denied \| expired.  
Agent MUST NOT self-approve.  
Policy evaluation inputs: principal role, capabilities, resource context, operation type.

---

## AuditEvent

Must support FR-027a (executed) and FR-027b (denied). Payload redacted — no secrets/tokens.

Queryable by correlation_id, operation_id.

---

## Workflow / ProcessSession / LogStream

Unchanged intent: pipelines of ops; process sessions with bounded output; log chunks with sequence and backpressure policy.

---

## KnowledgeItem (light — in scope)

| Field | Rules |
| --- | --- |
| id | required |
| system_id | FK |
| subject_type | system \| environment \| resource \| component |
| subject_id | required |
| title | required |
| body | text size-limited |
| external_url | optional |
| kind | note \| description \| link \| runbook_ref \| adr_ref |
| created_at / updated_at | required |

**Out of model:** embeddings, vector stores, RAG pipelines.

Operations: create, read, update, delete, associate (via subject fields).

---

## SecretReference

| Field | Rules |
| --- | --- |
| id | required unique |
| name | required |
| backend | `none` \| `os` \| `external` |
| locator | required opaque locator — **never** raw secret value |
| created_at / updated_at | required |

### Lifecycle (completion — CHK086)

| Action | In scope |
| --- | --- |
| Create reference | Yes |
| List / get metadata | Yes |
| Delete reference | Yes |
| Resolve value for op execution in core only | Yes (never to renderer/audit) |
| Rotate / revoke workflows / TTL expiry product | **DEFERRED** |
| Multi-tenant vaults | **DEFERRED** |

Redaction: secrets MUST NOT enter audit, logs, stream buffers, or UI.

---

## SessionDescriptor (runtime file, not SQLite)

Written by Main/core process: endpoint, token, pid, started_at. Ephemeral. Not a durable entity. Bootstrap: [contracts/session.md](./contracts/session.md).

---

## SQLite migrations (planned names — CHK079)

Implement under `backend/platform/db/sqlite/migrations/` as ordered SQL files:

| File | Entities |
| --- | --- |
| `0001_init_principals.sql` | principal, role bindings |
| `0002_workspace_system_env.sql` | workspace, system, environment, target |
| `0003_resources_arch.sql` | resource, relationship, architecture_layout, evidence |
| `0004_integrations_ops.sql` | integration, operation, approval, policy |
| `0005_audit_logs.sql` | audit_event, process_session, log metadata |
| `0006_workflows_knowledge.sql` | workflow, knowledge_item |
| `0007_secret_refs.sql` | secret_reference |
| `0008_workspace_layout.sql` | dock/workspace layout prefs |

Exact SQL authored at implement time; **names above are the normative plan list**. Versioned migrate up required; dual-read of legacy conf **forbidden**.

---

## Explicitly out of model (this feature)

Organization · Membership · Team · Node · SyncConflict product · Entitlement · Billing · Provisioning-as-domain · PostgreSQL tables · NATS

---

## Validation globals

1. No terminal success without policy allow recorded.  
2. Agent ≠ human id.  
3. external_key uniqueness when set.  
4. Parameters schema-validated per op type.  
5. Audit redaction.  
6. Layout revision concurrency.  
7. Role cannot escalate via client claim alone (server evaluates).  
8. Role catalog is exactly Owner / Operator / Viewer / Agent (FR-024a).  
