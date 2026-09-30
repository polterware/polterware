import { For, Show, createEffect, createMemo, createSignal, onCleanup } from "solid-js";
import {
  Button,
  Dialog,
  DitherIconTile,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Field,
  Input,
  MoreHorizontal,
  PageContent,
  PageEmpty,
  PageLoading,
  PageToolbar,
  Select,
  Bot,
  Check,
  Clock,
  Eye,
  GitCompare,
  Hammer,
  Pencil,
  ShieldCheck,
  StatusChip,
  Scale,
  ThumbsDownIcon,
  Tooltip,
  Tabs,
} from "@polterware/frontend-design-system";
import { detailPanelStore } from "@polterware/frontend-state";
import type {
  WorkspaceInbox,
  WorkspaceInboxAction,
  WorkspaceInboxItem,
  WorkspaceInboxKind,
} from "./workspace-mocks";
import "./inbox-page.css";

const INBOX_SCOPE = "inbox:items";

type InboxFilter = "all" | WorkspaceInboxKind;

const FILTERS: { id: InboxFilter; label: string }[] = [
  { id: "all", label: "All" },
  { id: "decision", label: "Decisions" },
  { id: "drift", label: "Drift" },
  { id: "verify", label: "Verify" },
];

const KIND_LABEL: Record<WorkspaceInboxKind, string> = {
  decision: "Decision",
  drift: "Drift",
  verify: "Verify",
};

const ACTION_LABEL: Record<WorkspaceInboxAction, string> = {
  decide: "Decide",
  delegate: "Delegate",
  defer: "Defer",
  fix_code: "Fix code",
  update_intent: "Update intent",
  ignore: "Ignore",
  verify: "Mark verified",
  reject: "Reject",
};

const ACTION_ICON = {
  decide: Check,
  delegate: Bot,
  defer: Clock,
  fix_code: Hammer,
  update_intent: Pencil,
  ignore: Eye,
  verify: ShieldCheck,
  reject: ThumbsDownIcon,
} as const;

const KIND_ICON = { decision: Scale, drift: GitCompare, verify: ShieldCheck } as const;

function statusLabel(item: WorkspaceInboxItem): string {
  const kind = KIND_LABEL[item.kind];
  if (item.state === "deferred") return `Deferred ${kind.toLowerCase()}`;
  return item.blocking ? `Blocking ${kind.toLowerCase()}` : kind;
}

/** Inbox — Decision, Drift, and Verify items derived from the project's data. */
export function InboxPage(props: {
  inbox?: WorkspaceInbox;
  loading?: boolean;
  onResolve?: (item: WorkspaceInboxItem, action: WorkspaceInboxAction, answer?: string) => Promise<void>;
}) {
  const [filter, setFilter] = createSignal<InboxFilter>("all");
  const [pending, setPending] = createSignal("");
  const [deciding, setDeciding] = createSignal<WorkspaceInboxItem | null>(null);
  const [answer, setAnswer] = createSignal("");

  // The Inbox has no detail targets; registering an empty collection keeps the shared
  // panel hidden instead of inheriting another surface's selection.
  createEffect(() => {
    detailPanelStore.getState().registerCollection({
      scopeId: INBOX_SCOPE,
      status: "ready",
      targets: [],
    });
    onCleanup(() => detailPanelStore.getState().unregisterCollection(INBOX_SCOPE));
  });

  const items = () => props.inbox?.items ?? [];
  const countOf = (kind: InboxFilter) =>
    items().filter((item) => item.state === "open" && (kind === "all" || item.kind === kind)).length;
  const visible = createMemo(() =>
    items().filter((item) => filter() === "all" || item.kind === filter()),
  );
  const open = () => visible().filter((item) => item.state === "open");
  const deferred = () => visible().filter((item) => item.state === "deferred");

  async function run(item: WorkspaceInboxItem, action: WorkspaceInboxAction, value?: string) {
    if (action === "decide" && value === undefined) {
      setAnswer(item.options[0] ?? "");
      setDeciding(item);
      return;
    }
    setPending(item.id);
    try {
      await props.onResolve?.(item, action, value);
      if (action === "decide") setDeciding(null);
    } catch {
      // The host reports the failure; the dialog stays open to retry.
    } finally {
      setPending("");
    }
  }

  function Row(rowProps: { item: WorkspaceInboxItem }) {
    const item = () => rowProps.item;
    return (
      <li class="pw-inbox-row" data-state={item().state} title={item().id}>
        <Tooltip.Root openDelay={400}>
          <Tooltip.Trigger
            asChild={(triggerProps) => (
              <span {...triggerProps({ class: "pw-inbox-row__status", role: "img", "aria-label": statusLabel(item()) })}>
                <DitherIconTile size={28} tone={item().blocking && item().state === "open" ? "security" : "neutral"} dimmed={item().state === "deferred"}>
                  {(() => {
                    const Icon = KIND_ICON[item().kind];
                    return <Icon size={14} aria-hidden />;
                  })()}
                </DitherIconTile>
              </span>
            )}
          />
          <Tooltip.Content>{statusLabel(item())}</Tooltip.Content>
        </Tooltip.Root>
        <div class="pw-inbox-row__body">
          <span class="pw-inbox-row__title">{item().title}</span>
          <p class="pw-inbox-row__meta">
            {[item().where, item().source].filter(Boolean).join(" · ")}
          </p>
        </div>
        <DropdownMenu positioning={{ placement: "bottom-end" }}>
          <DropdownMenuTrigger
            aria-label={`Actions for ${item().title}`}
            disabled={pending() !== "" || !props.onResolve}
            class="pw-inbox-row__menu h-7 w-7 shrink-0 justify-center rounded-full text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-primary)]"
          >
            <MoreHorizontal size={14} />
          </DropdownMenuTrigger>
          <DropdownMenuContent class="min-w-40">
            <For each={item().actions}>
              {(action) => (
                <DropdownMenuItem value={action} onSelect={() => void run(item(), action)}>
                  {(() => {
                    const Icon = ACTION_ICON[action];
                    return <Icon size={14} aria-hidden />;
                  })()}
                  {ACTION_LABEL[action]}
                </DropdownMenuItem>
              )}
            </For>
          </DropdownMenuContent>
        </DropdownMenu>
      </li>
    );
  }

  return (
    <div class="flex h-full min-h-0 flex-col">
      <PageToolbar.Root>
        <PageToolbar.Tabs>
          <Tabs
            items={FILTERS.map((entry) => ({
              id: entry.id,
              label: entry.label,
              count: countOf(entry.id) || undefined,
            }))}
            value={filter()}
            onChange={(value) => setFilter(value as InboxFilter)}
            aria-label="Inbox"
          />
        </PageToolbar.Tabs>
        <PageToolbar.Actions>
          <Show when={(props.inbox?.blockingCount ?? 0) > 0}>
            <StatusChip tone="danger">{props.inbox?.blockingCount} blocking</StatusChip>
          </Show>
        </PageToolbar.Actions>
      </PageToolbar.Root>
      <PageContent class="p-0">
        <Show
          when={visible().length > 0}
          fallback={
            <Show
              when={!props.loading}
              fallback={<PageLoading layout="table" label="Loading the Inbox…" />}
            >
              <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
                <PageEmpty
                  title="Nothing needs you right now."
                  description="Open questions, drift findings, and facts to verify collect here."
                />
              </div>
            </Show>
          }
        >
          <ul class="pw-inbox-list">
            <For each={open()}>{(item) => <Row item={item} />}</For>
          </ul>
          <Show when={deferred().length > 0}>
            <p class="pw-inbox-section-label">Deferred</p>
            <ul class="pw-inbox-list">
              <For each={deferred()}>{(item) => <Row item={item} />}</For>
            </ul>
          </Show>
        </Show>
      </PageContent>

      <Dialog
        open={deciding() !== null}
        title="Decide"
        description={deciding()?.title}
        size="sm"
        onClose={() => setDeciding(null)}
        footer={
          <div class="flex justify-end gap-2">
            <Button variant="ghost" onClick={() => setDeciding(null)}>
              Cancel
            </Button>
            <Button
              variant="primary"
              disabled={answer().trim() === "" || pending() !== ""}
              onClick={() => {
                const item = deciding();
                if (item) void run(item, "decide", answer().trim());
              }}
            >
              {pending() !== "" ? "Saving…" : "Record decision"}
            </Button>
          </div>
        }
      >
        <Show
          when={(deciding()?.options.length ?? 0) > 0}
          fallback={
            <Field label="Answer">
              <Input value={answer()} onInput={(event) => setAnswer(event.currentTarget.value)} />
            </Field>
          }
        >
          <Field label="Answer">
            <Select
              aria-label="Answer"
              value={answer()}
              onChange={setAnswer}
              options={(deciding()?.options ?? []).map((option) => ({ value: option, label: option }))}
            />
          </Field>
        </Show>
      </Dialog>
    </div>
  );
}
