import { For, Show, createMemo, createSignal } from "solid-js";
import {
  Button,
  Dialog,
  Field,
  FilterChip,
  Input,
  PageContent,
  PageEmpty,
  PageToolbar,
  Select,
  StatusChip,
  type StatusTone,
} from "@polterware/frontend-design-system";
import type {
  WorkspaceInbox,
  WorkspaceInboxAction,
  WorkspaceInboxItem,
  WorkspaceInboxKind,
} from "./workspace-mocks";
import "./inbox-page.css";

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

/** The first action of a kind is its primary one. */
const PRIMARY_ACTIONS = new Set<WorkspaceInboxAction>(["decide", "fix_code", "verify"]);

function kindTone(item: WorkspaceInboxItem): StatusTone {
  if (item.state === "deferred") return "muted";
  if (item.blocking) return "danger";
  return item.kind === "drift" ? "warn" : "muted";
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
      <li class="pw-inbox-row" data-state={item().state}>
        <div class="pw-inbox-row__head">
          <StatusChip tone={kindTone(item())}>
            {item().blocking && item().state === "open" ? `Blocking ${KIND_LABEL[item().kind].toLowerCase()}` : KIND_LABEL[item().kind]}
          </StatusChip>
          <span class="pw-inbox-row__title">{item().title}</span>
          <Show when={item().where}>
            <span class="pw-inbox-row__meta font-mono">{item().where}</span>
          </Show>
          <Show when={item().source}>
            <span class="pw-inbox-row__meta">{item().source}</span>
          </Show>
        </div>
        <Show when={item().options.length > 0}>
          <p class="pw-inbox-row__detail">Options: {item().options.join(" · ")}</p>
        </Show>
        <div class="pw-inbox-row__actions">
          <For each={item().actions}>
            {(action) => (
              <Button
                size="sm"
                variant={PRIMARY_ACTIONS.has(action) ? "primary" : "ghost"}
                disabled={pending() !== "" || !props.onResolve}
                onClick={() => void run(item(), action)}
              >
                {ACTION_LABEL[action]}
              </Button>
            )}
          </For>
          <span class="pw-inbox-row__id font-mono">{item().id}</span>
        </div>
      </li>
    );
  }

  return (
    <div class="flex h-full min-h-0 flex-col">
      <PageToolbar.Root>
        <PageToolbar.Leading>
          <span class="pw-page-toolbar-title">Inbox</span>
        </PageToolbar.Leading>
        <PageToolbar.Actions>
          <Show when={(props.inbox?.blockingCount ?? 0) > 0}>
            <StatusChip tone="danger">{props.inbox?.blockingCount} blocking</StatusChip>
          </Show>
        </PageToolbar.Actions>
      </PageToolbar.Root>
      <div class="pw-inbox-filters">
        <For each={FILTERS}>
          {(entry) => (
            <FilterChip
              selected={filter() === entry.id}
              count={countOf(entry.id) || undefined}
              onClick={() => setFilter(entry.id)}
            >
              {entry.label}
            </FilterChip>
          )}
        </For>
      </div>
      <PageContent class="p-0">
        <Show
          when={visible().length > 0}
          fallback={
            <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
              <PageEmpty
                title={props.loading ? "Loading the Inbox…" : "Nothing needs you right now."}
                description="Open questions, drift findings, and facts to verify collect here."
              />
            </div>
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
