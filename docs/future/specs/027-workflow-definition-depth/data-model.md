# Data Model: Workflow Definition Depth

## WorkflowDef

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| id | string | yes | Stable fixture/session id |
| name | string | yes | Operator title |
| description | string | no | Purpose summary |
| trigger | string | yes | Manual / Scheduled / Health event / Webhook |
| lastRun | string | yes | Relative or honest “Never” / “—” |
| status | WorkflowStatus \| string | yes | Definition lifecycle / last-outcome display |
| version | number | yes | Monotonic definition version |
| steps | WorkflowStep[] | no | Ordered sequential pipeline |
| failureStrategy | WorkflowFailureStrategy | no | Default step-failure behavior |
| inputSummary | string[] | no | Operator-facing inputs |
| outputSummary | string[] | no | Operator-facing outputs |
| artifactsSummary | string[] | no | Artifact labels (no secrets) |
| notes | string | no | Honesty / governance notes |
| dataSource | `"core"` \| `"fixture"` | yes | Source honesty |

### WorkflowStatus

`draft` | `ready` | `running` | `success` | `failed` | `archived`

Display labels: Draft, Ready, Running, Success, Failed, Archived. Fixtures may use Title Case; helpers normalize.

### WorkflowFailureStrategy

| Value | Operator label | Meaning |
| --- | --- | --- |
| stop | Stop on failure | Halt pipeline at failed step |
| continue | Continue on failure | Proceed to next step (mark failure) |
| fail_pipeline | Fail pipeline | Mark whole definition run failed and stop |

## WorkflowStep

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| id | string | yes | Stable within definition |
| label | string | yes | Node title |
| type | WorkflowStepType | yes | validate, approval, build, deploy, health_check, command, notify |
| config | Record\<string, string\> | no | Type-specific flat mock fields |
| condition | string | no | Optional run condition |
| retryPolicy | WorkflowRetryPolicy | no | Explicit per-step retry |
| failureStrategy | WorkflowFailureStrategy | no | Override definition default |
| approval | WorkflowApprovalSemantics | no | Required shape for approval type fixtures |

### WorkflowRetryPolicy

| Field | Type | Notes |
| --- | --- | --- |
| maxAttempts | number | ≥ 1 |
| backoff | `"none"` \| `"fixed"` \| `"exponential"` | |
| delaySeconds | number | optional |
| respectsPolicy | `true` | Always true — invariant documented in UI |

### WorkflowApprovalSemantics

| Field | Type | Notes |
| --- | --- | --- |
| condition | string | When gate applies |
| reviewers | string | Reviewer set label |
| expiration | string | Human duration |
| decisionIsNotExecution | `true` | Always true in product copy |
| separationOfDuties | boolean | optional |

### Type-specific config fields (minimum distinct sets)

| Type | Distinct config keys |
| --- | --- |
| validate | target, environment |
| approval | name, condition, reviewers, expiration (+ approval object) |
| command | name, command, target |
| deploy | name, environment, notes |
| build | name, notes |
| health_check | name, target, timeout |
| notify | name, channel |

## Relationships

- **WorkflowDef 1—\* WorkflowStep** (ordered)
- **WorkflowDef 1—\* OperationRun** via `run.workflowId` + `run.workflowVersion` (immutable at run start)
- **Schedule → WorkflowDef** optional binding (unchanged by this feature)

## Invariants

1. Editing definition does not mutate historical runs.
2. Retry never skips policy/approval.
3. Approval decision is not execution.
4. Secrets are references/labels only.
5. Version increments on save of existing definition; create starts at 1.
