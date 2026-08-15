import { For, Show, createMemo, createSignal } from "solid-js";
import { Field, Input } from "@polterware/frontend-design-system";
import type { ResourceLinkPort } from "../domain/fixture-resource-link-port";
import type { LinkableResource } from "../domain/place-link";

export function ResourceLinkPicker(props: {
  port: ResourceLinkPort;
  environmentId: string;
  componentType?: string;
  selectedId: string | undefined;
  onSelect: (resource: LinkableResource | null) => void;
}) {
  const [query, setQuery] = createSignal("");

  const candidates = createMemo(() =>
    props.port.listCandidates({
      environmentId: props.environmentId,
      componentType: props.componentType,
      query: query(),
      typeFilterStrict: true,
    }),
  );

  return (
    <div class="flex flex-col gap-3">
      <Field label="Search resources" for="place-link-search">
        <Input
          id="place-link-search"
          size="md"
          value={query()}
          placeholder="Filter by name…"
          autocomplete="off"
          onInput={(e) => setQuery(e.currentTarget.value)}
        />
      </Field>

      <Show
        when={candidates().length > 0}
        fallback={
          <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
            No matching resources for this type and environment. Discover or import
            resources later, or place as planned without a link.
          </p>
        }
      >
        <ul class="m-0 max-h-40 list-none overflow-y-auto p-0" role="listbox" aria-label="Linkable resources">
          <For each={candidates()}>
            {(r) => {
              const selected = () => props.selectedId === r.id;
              return (
                <li class="mb-1">
                  <button
                    type="button"
                    role="option"
                    aria-selected={selected()}
                    class={`flex w-full flex-col items-start gap-0.5 rounded-[var(--radius-item)] border-0 px-2.5 py-2 text-left ${
                      selected()
                        ? "bg-[var(--color-surface-selected)] text-[var(--color-text-primary)]"
                        : "bg-transparent text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]"
                    }`}
                    onClick={() => props.onSelect(selected() ? null : r)}
                  >
                    <span class="text-[13px] font-medium text-[var(--color-text-primary)]">
                      {r.name}
                    </span>
                    <span class="text-[11px] text-[var(--color-text-muted)]">
                      {r.type}
                      {r.provider ? ` · ${r.provider}` : ""}
                      {r.environmentLabel ? ` · ${r.environmentLabel}` : ""}
                      {r.dataSource === "fixture" ? " · fixture" : ""}
                    </span>
                  </button>
                </li>
              );
            }}
          </For>
        </ul>
      </Show>

      <Show when={props.selectedId}>
        <button
          type="button"
          class="self-start border-0 bg-transparent p-0 text-[12px] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
          onClick={() => props.onSelect(null)}
        >
          Clear link
        </button>
      </Show>
    </div>
  );
}
