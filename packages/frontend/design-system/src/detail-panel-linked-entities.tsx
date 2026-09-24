import { For, Show, createMemo, createSignal, type JSX } from "solid-js";
import { Button } from "./button";
import { Plus, X } from "./hugeicons";
import { Input } from "./input";
import { MenuItem } from "./menu";
import { Popover } from "./popover";

export type DetailPanelLinkedEntity = {
  id: string;
  title: string;
  /** Muted secondary text, e.g. relation predicate and kind. */
  caption?: string;
  icon?: JSX.Element;
};

export type DetailPanelLinkCandidate = {
  id: string;
  title: string;
  caption?: string;
};

export type DetailPanelLinkedEntitiesProps = {
  items: readonly DetailPanelLinkedEntity[];
  candidates?: readonly DetailPanelLinkCandidate[];
  linkLabel?: string;
  onOpen?: (id: string) => void;
  onRemove?: (id: string) => void;
  onLink?: (candidateId: string) => void;
  class?: string;
};

export function DetailPanelLinkedEntities(props: DetailPanelLinkedEntitiesProps) {
  const [open, setOpen] = createSignal(false);
  const [query, setQuery] = createSignal("");
  let anchor: HTMLButtonElement | undefined;

  const filtered = createMemo(() => {
    const needle = query().trim().toLowerCase();
    const candidates = props.candidates ?? [];
    if (!needle) return candidates;
    return candidates.filter((candidate) =>
      `${candidate.title} ${candidate.caption ?? ""}`.toLowerCase().includes(needle),
    );
  });

  const canLink = () => Boolean(props.onLink) && (props.candidates?.length ?? 0) > 0;

  function close() {
    setOpen(false);
    setQuery("");
  }

  return (
    <Show when={props.items.length > 0 || canLink()}>
      <div class={`pw-detail-panel__links ${props.class ?? ""}`}>
        <For each={props.items}>
          {(item) => (
            <span class="pw-detail-panel__link-chip">
              <button
                type="button"
                class="pw-detail-panel__link-open"
                disabled={!props.onOpen}
                onClick={() => props.onOpen?.(item.id)}
              >
                <Show when={item.icon}>
                  <span class="pw-detail-panel__link-icon" aria-hidden>
                    {item.icon}
                  </span>
                </Show>
                <span class="pw-detail-panel__link-title">{item.title}</span>
                <Show when={item.caption}>
                  <span class="pw-detail-panel__link-caption">{item.caption}</span>
                </Show>
              </button>
              <Show when={props.onRemove}>
                <button
                  type="button"
                  class="pw-detail-panel__link-remove"
                  aria-label={`Unlink ${item.title}`}
                  onClick={() => props.onRemove?.(item.id)}
                >
                  <X size={12} aria-hidden />
                </button>
              </Show>
            </span>
          )}
        </For>
        <Show when={canLink()}>
          <Button
            ref={anchor}
            variant="ghost"
            size="sm"
            class="pw-detail-panel__link-add"
            aria-expanded={open()}
            onClick={() => (open() ? close() : setOpen(true))}
          >
            <Plus size={12} aria-hidden />
            {props.linkLabel ?? "Link"}
          </Button>
          <Popover
            open={open()}
            onClose={close}
            anchorRef={() => anchor}
            title={props.linkLabel ?? "Link entity"}
            width={280}
          >
            <div class="p-2">
              <Input
                size="sm"
                placeholder="Search entities…"
                value={query()}
                onInput={(event) => setQuery(event.currentTarget.value)}
                aria-label="Search entities"
                autofocus
              />
            </div>
            <div class="max-h-[240px] overflow-auto px-1 pb-1">
              <Show
                when={filtered().length > 0}
                fallback={
                  <p class="m-0 px-2 py-1.5 text-[12px] text-[var(--color-text-muted)]">
                    No matching entities.
                  </p>
                }
              >
                <For each={filtered()}>
                  {(candidate) => (
                    <MenuItem
                      onClick={() => {
                        props.onLink?.(candidate.id);
                        close();
                      }}
                    >
                      <span class="min-w-0 flex-1 truncate">{candidate.title}</span>
                      <Show when={candidate.caption}>
                        <span class="shrink-0 text-[11px] text-[var(--color-text-muted)]">
                          {candidate.caption}
                        </span>
                      </Show>
                    </MenuItem>
                  )}
                </For>
              </Show>
            </div>
          </Popover>
        </Show>
      </div>
    </Show>
  );
}
