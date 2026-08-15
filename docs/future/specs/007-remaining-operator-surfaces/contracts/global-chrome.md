# Contract: Global Chrome Surfaces

**Owner**: `features/shell` (overview, switcher, notifications), `features/commander` (search results), `features/integrations` (sync center page)  
**Consumers**: App composition

## Project Overview

```ts
type ProjectOverviewProps = {
  snapshot: ProjectOverviewSnapshot | null;
  dataSource: DataSource;
  onQuickAction: (id: "architecture" | "create-resource" | "run-command" | "connect-integration") => void;
  onAttention: (item: AttentionItem) => void;
  onActivity: (item: ActivityItem) => void;
};
```

Rules:

1. Full main surface page (`section=overview`).  
2. Empty/offline states honest.  
3. Quick actions navigate or open dialogs; never silent mutations.

## Project switcher popover

```ts
type ProjectSwitcherProps = {
  open: boolean;
  onClose: () => void;
  projects: ProjectSwitcherItem[];
  query: string;
  onQuery: (q: string) => void;
  onSelect: (projectId: string) => void;
  onCreateProject: () => void;
  onManageOrganizations: () => void;
};
```

Rules:

1. Not a page.  
2. Group by organization.  
3. Current project marked.  
4. Escape / outside click closes.

## Notification center drawer

```ts
type NotificationDrawerProps = {
  open: boolean;
  onClose: () => void;
  items: NotificationItem[];
  filter: "all" | "unread" | "approvals";
  onFilter: (f: NotificationDrawerProps["filter"]) => void;
  onMarkAllRead: () => void;
  onAction: (item: NotificationItem) => void;
};
```

Rules:

1. Temporary drawer only; not permanent chrome.  
2. Closing clears `route.drawer` when type is notifications.  
3. Does not replace Approvals destination; deep-links into Approvals when needed.  
4. Mark all read is non-destructive.

## Search results page

```ts
type SearchResultsProps = {
  query: string;
  onQuery: (q: string) => void;
  category: string; // all | architecture | resources | operations | knowledge | …
  counts: Record<string, number>;
  hits: SearchHit[];
  dataSource: DataSource;
  onOpenHit: (hit: SearchHit) => void;
};
```

Rules:

1. `section=search`.  
2. Category chips filter; All shows grouped or flat list.  
3. Zero results empty state with refine guidance.

## Synchronization center

```ts
type SyncCenterProps = {
  tab: "status" | "offline-queue" | "conflicts" | "history";
  onTab: (t: SyncCenterProps["tab"]) => void;
  endpoints: SyncEndpointStatus[];
  pendingUploadCount: number;
  conflictCount: number;
  queue: OfflineQueueItem[];
  conflicts: SyncConflict[];
  history: { id: string; summary: string; at: string; outcome: string }[];
  dataSource: DataSource;
  onSyncNow: () => void;
  onOpenSettings: () => void;
  onOpenConflict: (id: string) => void;
};
```

## Conflict resolve (shared)

```ts
type ConflictResolveProps = {
  conflict: SyncConflict;
  onResolve: (choice: "keep-polter" | "accept-provider" | "ignore") => void;
  onCancel: () => void;
  submitting?: boolean;
};
```

Rules:

1. Side-by-side Polter vs provider.  
2. Explicit choice required.  
3. Resolution audited (core when wired; fixture records outcome label in UI).  
4. Same component for Sync center and Integrations Sync.

## Onboarding

```ts
type OnboardingProps = {
  state: OnboardingState;
  onCreateOrganization: () => void;
  onJoinInvite: () => void;
  onCreateProject: (input: { name: string; description: string; environments: string[]; storage: "local-first" | "cloud" }) => void;
  onBack: () => void;
};
```

Rules:

1. Local-first is default storage choice.  
2. Cloud signup not mandatory.  
3. Completing project exits to overview/architecture.
