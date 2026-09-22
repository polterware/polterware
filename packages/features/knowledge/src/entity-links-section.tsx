import { For, Show } from "solid-js";
import { Button, DetailPanel } from "@polterware/frontend-design-system";
import type { WorkspaceEntityLink, WorkspacePpmEntity } from "@polterware/feature-shell";

export function EntityLinksSection(props: {
  links: WorkspaceEntityLink[];
  entities?: WorkspacePpmEntity[];
  onOpenEntity?: (entityId: string, refId: string, kind: string) => void;
  onLinkEntity?: (entityId: string) => void;
  onUnlink?: (linkId: string) => void;
}) {
  const available = () => {
    const linked = new Set(props.links.map((link) => link.entityId));
    return (props.entities ?? []).filter((entity) => !linked.has(entity.id));
  };

  return (
    <DetailPanel.Section title="Linked entities">
      <Show
        when={props.links.length > 0}
        fallback={<p class="text-sm text-[var(--color-text-muted)]">No linked project entities.</p>}
      >
        <ul class="flex flex-col gap-2">
          <For each={props.links}>
            {(link) => (
              <li class="flex items-center justify-between gap-2 text-sm">
                <button
                  type="button"
                  class="text-left text-[var(--color-text-primary)] underline-offset-2 hover:underline"
                  onClick={() => props.onOpenEntity?.(link.entityId, link.entityRefId, link.entityKind)}
                >
                  {link.entityTitle}
                  <span class="ml-2 text-[var(--color-text-muted)]">
                    {link.predicate} · {link.entityKind}
                  </span>
                </button>
                <Show when={props.onUnlink}>
                  <Button size="sm" variant="ghost" onClick={() => props.onUnlink?.(link.id)}>
                    Unlink
                  </Button>
                </Show>
              </li>
            )}
          </For>
        </ul>
      </Show>
      <Show when={available().length > 0 && props.onLinkEntity}>
        <label class="mt-3 flex flex-col gap-1 text-sm">
          <span class="text-[var(--color-text-muted)]">Link entity</span>
          <select
            class="rounded-[var(--radius-control)] border border-[var(--color-border-subtle)] bg-[var(--color-surface)] px-2 py-1"
            value=""
            onChange={(event) => {
              const value = event.currentTarget.value;
              event.currentTarget.value = "";
              if (value) props.onLinkEntity?.(value);
            }}
          >
            <option value="">Select…</option>
            <For each={available()}>
              {(entity) => (
                <option value={entity.id}>
                  {entity.title} ({entity.kind})
                </option>
              )}
            </For>
          </select>
        </label>
      </Show>
    </DetailPanel.Section>
  );
}
