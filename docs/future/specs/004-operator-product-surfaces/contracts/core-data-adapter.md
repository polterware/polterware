# Contract: Core Data Adapter (Desktop)

**Owner**: `apps/desktop/src/renderer/App.tsx` + `@polterware/frontend-data`  
**Consumers**: Feature views via props (features stay free of bridge secrets)

## Responsibilities

| Layer | Owns |
| --- | --- |
| App | Session endpoint/token via `window.polterware`, health, createCoreClient, fixture fallback, route state |
| frontend-data | Connect clients for existing services |
| Features | Pure presentation + view-model mapping helpers |

## Preferred core services (when present)

| UI need | Proto / service (local/v1) |
| --- | --- |
| Health | HealthService |
| Resources | project/architecture/resource RPCs as already used in 003 |
| Operations | OperationService |
| Approvals | ApprovalService |
| Workflows | WorkflowService |
| Processes | ProcessService |
| Knowledge | KnowledgeService |
| Identity/members | IdentityService |
| Policy | PolicyService |
| Audit | AuditService |
| Integrations | IntegrationService |
| Secrets | SecretService (references only) |
| Project/settings | ProjectService / maintenance |

## Fallback policy

```text
try core → on failure/empty → fixture (label) → empty state
```

Never:

- Write secrets to renderer logs  
- Call Node/OS from feature packages  
- Claim fixture rows are live multi-provider sync  

## Mutation adapter shape

```ts
type RequestOperationInput = {
  targetId: string;
  command: string;
  cwd?: string;
  environmentId?: string;
};

// Returns operation + optional approval required flag
```

UI maps approval-required to Approvals tab or inline pending messaging.
