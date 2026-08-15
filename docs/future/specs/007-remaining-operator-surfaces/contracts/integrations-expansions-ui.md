# Contract: Integrations Expansions UI

**Owner**: `packages/features/integrations`  
**Extends**: 004 connected/catalog/credentials/sync

## Tabs

`connected` | `catalog` | `credentials` | `sync` | **`webhooks`**

## Setup wizard

```ts
type IntegrationSetupWizardProps = {
  providerId: string;
  providerName: string;
  step: number;
  stepCount: number;
  method: "assume-role" | "access-key" | string;
  values: Record<string, string>;
  permissions: { id: string; label: string; granted: boolean }[];
  onBack: () => void;
  onNext: () => void;
  onTestConnection: () => Promise<{ ok: boolean; message?: string }>;
  onFinish: () => void;
};
```

Rules:

1. Secret fields write-only after set.  
2. Test connection before finish when network available; offline shows honest failure.  
3. Permissions requested listed explicitly.

## Webhooks

```ts
type WebhooksProps = {
  items: WebhookView[];
  selectedId?: string;
  onSelect: (id: string) => void;
  onCreate: () => void;
  onSendTest: (id: string) => void;
  onViewDeliveries: (id: string) => void;
  onEdit: (id: string) => void;
};
```

Rules:

1. Signing secret: configured boolean only after create; rotate replaces.  
2. Failed webhooks show status without leaking secrets.  
3. Deliveries list is secondary detail (page or dialog).

## Sync conflicts

Use shared [ConflictResolveProps](./global-chrome.md) from Integrations Sync tab rows.
