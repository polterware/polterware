# Contract: Settings Expansions UI

**Owner**: `packages/features/settings`  
**Extends**: 004 project/environments/notifications/core/account

## Tabs (target)

`project` | `organization` | `environments` | `notifications` | `core` | `developer` | `billing` | `security` | `data`

Account (004) may remain temporarily; profile/session fields migrate to Security/Project without dual long-term homes.

## Organization

```ts
type OrganizationSettingsProps = {
  org: OrganizationSettings;
  onChange: (patch: Partial<OrganizationSettings>) => void;
  onSave: () => void;
  onChangeLogo?: () => void;
};
```

## Developer

```ts
type DeveloperSettingsProps = {
  status: DeveloperStatus;
  onCopyCliInstall: () => void;
  onCopyMcpConfig: () => void;
  onManageTools?: () => void;
  onRestartMcp?: () => void;
  onOpenCliDocs?: () => void;
};
```

Rules: never mark MCP/CLI running if process not actually up (honesty).

## Billing

```ts
type BillingSettingsProps = {
  snapshot: BillingSnapshot;
  tab: "plan" | "usage" | "invoices";
  onManagePlan?: () => void;
  onUpdatePayment?: () => void;
  onViewInvoices?: () => void;
};
```

When `snapshot.available === false`, show unavailable explanation (local-only / no account)—no fake plan.

## Security

```ts
type SecuritySettingsProps = {
  mfaEnabled: boolean;
  sessions: SecuritySession[];
  policies: { requireMfaAdmins: boolean; requireProdApproval: boolean; restrictIp: boolean };
  onConfigureMfa: () => void;
  onRevokeSession: (id: string) => void;
  onSavePolicies: () => void;
};
```

Current session cannot be revoked via the same control as others (label Current).

## Data / retention

```ts
type DataSettingsProps = {
  retention: RetentionPolicy;
  onChange: (patch: Partial<RetentionPolicy>) => void;
  onExportProject: () => void;
  onDeleteRetainedLogs: () => void; // confirm required
};
```

Destructive actions require explicit confirmation dialog.
