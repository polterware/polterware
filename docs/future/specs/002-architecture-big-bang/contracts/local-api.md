# Contract: Local API (polterware-core)

**Transport**: ConnectRPC `127.0.0.1`  
**Auth**: FR-067 Bearer token  
**Package**: `polterware.local.v1`

## HealthService

`Check` → status, version, uptime

## IdentityService

| RPC | Notes |
| --- | --- |
| GetCurrentPrincipal | role + capabilities |
| ListPrincipals | local |
| UpsertPrincipal | Owner only |
| RegisterAgentPrincipal | restricted caps; role=agent |

## ProjectService

Systems, Environments, Workspace get/save (local model — **no Organization RPCs**)

## ArchitectureService

Layout get/update (revision), Resources, Relationships, Evidence, Discovery run/list/associate

## KnowledgeService

Create/Get/Update/Delete/ListBySubject — light knowledge only

## IntegrationService

ListIntegrations, GetToolStatus, ResolveHandoff

## OperationService

See [operations.md](./operations.md)

## WorkflowService

List/Save/Delete/Run pipelines; GetPipelineRun

## ProcessService

List/Start/Stop/Get; SubscribeProcessOutput stream

## LogService

SubscribeLogs, QueryLogs (bounded)

## PolicyService / ApprovalService

Evaluate (dry-run); ListApprovals; ResolveApproval (human only; agent self-approve denied)

## AuditService

List/Get by correlation_id, operation_id — returns FR-027a/b shaped records

## MaintenanceService

ResetLocalStore, GetStoreHealth

## Non-goals

- Org/membership admin  
- Node fleet  
- Generic unrestricted shell from renderer  
- Skills setup/preview product RPCs  
