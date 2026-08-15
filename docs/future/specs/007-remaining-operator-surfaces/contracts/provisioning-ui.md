# Contract: Provisioning & Node Registration UI

**Owner**: `packages/features/resources`  
**Consumers**: App when `section=resources` and `tab=provisioning` (or node wizard)

## Resources tabs (extended)

`inventory` | `nodes` | `processes` | **`provisioning`**

## Provisioning surface

Sub-views via `filter` (or equivalent): `catalog` | `executions` | `drift`.

### Catalog

```ts
type ProvisioningCatalogProps = {
  templates: ProvisioningTemplate[];
  onSelect: (templateId: string) => void;
  onProvisionResource: () => void;
};
```

### Configure + plan review (page wizards)

```ts
type ProvisionConfigureProps = {
  template: ProvisioningTemplate;
  values: Record<string, unknown>;
  onChange: (values: Record<string, unknown>) => void;
  estimatedCostLabel?: string;
  step: number;
  stepCount: number;
  onBack: () => void;
  onReview: () => void;
};

type ProvisionPlanReviewProps = {
  plan: ProvisioningPlan;
  onBack: () => void;
  onSaveDraft: () => void;
  onRequestApprovalOrProvision: () => void; // label depends on policy
};
```

Rules:

1. Plan review shows resources to create + policy checks.  
2. Production approval required → CTA requests approval; no silent apply.  
3. Estimated cost optional; omit when unknown (do not invent).

### Execution detail

```ts
type ProvisionExecutionProps = {
  execution: ProvisioningExecution;
  onCancel?: () => void;
  onOpenProviderOperation?: () => void;
};
```

### Drift

```ts
type DriftProps = {
  items: DriftItem[];
  selectedId?: string;
  onSelect: (id: string) => void;
  onScan: () => void;
  onAcceptActual: (id: string) => void;
  onRestoreDesired: (id: string) => void;
  onIgnore: (id: string) => void;
};
```

Rules: each resolution action is explicit and audited when core is wired.

## Node registration wizard

```ts
type NodeRegistrationProps = {
  step: number;
  targetType?: "local" | "remote" | "docker" | "kubernetes";
  session?: NodeInstallSession;
  onSelectType: (t: NonNullable<NodeRegistrationProps["targetType"]>) => void;
  onCopyCommand: () => void;
  onRegenerateToken: () => void;
  onConfirmInstalled: () => void;
  onCancel: () => void;
};
```

Rules:

1. Token expiry visible.  
2. Heartbeat waiting state honest.  
3. Command copy does not log full token to fixtures as “live secret dump” after expiry.
