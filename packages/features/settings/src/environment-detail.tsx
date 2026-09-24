import { For, Show } from "solid-js";
import {
  Check,
  DetailPanel,
  PageError,
  PageLoading,
  Server,
  StatusChip,
} from "@polterware/frontend-design-system";
import type { ProjectEnvironment } from "./types";
import {
  environmentKindLabel,
  environmentKindTone,
  protectionLabel,
  protectionTone,
} from "./environment-status";

export function EnvironmentDetail(props: {
  environment: ProjectEnvironment | null | undefined;
  loading?: boolean;
  error?: string | null;
  notFound?: boolean;
  canDelete?: boolean;
  onSetDefault?: (environmentId: string) => void;
  onDelete?: (environmentId: string) => void;
}) {
  return (
    <DetailPanel.Content>
      <Show when={props.loading}>
        <DetailPanel.Header title="Loading environment" />
        <DetailPanel.Body>
          <DetailPanel.Main>
            <PageLoading label="Loading environment…" />
          </DetailPanel.Main>
        </DetailPanel.Body>
      </Show>

      <Show
        when={
          !props.loading && (props.error || props.notFound || !props.environment)
        }
      >
        <DetailPanel.Header
          title={
            props.notFound || !props.environment
              ? "Environment not found"
              : "Failed to load"
          }
        />
        <DetailPanel.Body>
          <DetailPanel.Main>
            <PageError
              title={
                props.notFound || !props.environment
                  ? "Environment not found"
                  : "Failed to load"
              }
              message={
                props.error ??
                "This environment is unavailable or was removed from the project."
              }
            />
          </DetailPanel.Main>
        </DetailPanel.Body>
      </Show>

      <Show when={!props.loading && props.environment}>
        {(environment) => {
          const rules = () => environment().protectionRules ?? [];
          const showActions = () =>
            Boolean(
              (props.onSetDefault && !environment().isDefault) ||
                (props.onDelete && props.canDelete !== false),
            );

          return (
            <>
              <DetailPanel.Header
                icon={<Server size={16} />}
                title={environment().name}
                status={
                  <>
                    <StatusChip
                      tone={environmentKindTone(environment().kind)}
                      label={environmentKindLabel(environment().kind)}
                    />
                    <StatusChip
                      tone={protectionTone(environment().protection)}
                      label={protectionLabel(environment().protection)}
                    />
                  </>
                }
                subtitle={environment().isDefault ? "Default environment" : "Environment"}
              >
                <Show when={showActions()}>
                  <DetailPanel.Actions>
                    <Show when={props.onSetDefault && !environment().isDefault}>
                      <DetailPanel.PrimaryAction
                        label="Set as default"
                        icon={<Check size={14} aria-hidden />}
                        onClick={() => props.onSetDefault?.(environment().id)}
                      />
                    </Show>
                    <Show when={props.onDelete && props.canDelete !== false}>
                      <DetailPanel.Overflow>
                        <DetailPanel.OverflowItem
                          value="delete"
                          danger
                          onSelect={() => props.onDelete?.(environment().id)}
                        >
                          Delete environment
                        </DetailPanel.OverflowItem>
                      </DetailPanel.Overflow>
                    </Show>
                  </DetailPanel.Actions>
                </Show>
              </DetailPanel.Header>

              <DetailPanel.Body>
                <DetailPanel.Main>
                  <DetailPanel.Section title="Summary">
                    <Show
                      when={environment().description}
                      fallback={<p class="pw-detail-panel__text" data-empty>No summary yet.</p>}
                    >
                      <p class="pw-detail-panel__text">{environment().description}</p>
                    </Show>
                  </DetailPanel.Section>
                  <Show when={rules().length > 0}>
                    <DetailPanel.Section title="Protection rules">
                      <ul
                        class="m-0 list-disc space-y-1 pl-4 text-[12.5px] text-[var(--color-text-secondary)]"
                        aria-label="Protection rules"
                      >
                        <For each={rules()}>{(rule) => <li>{rule}</li>}</For>
                      </ul>
                    </DetailPanel.Section>
                  </Show>
                </DetailPanel.Main>
                <DetailPanel.Rail>
                  <DetailPanel.Section title="Properties">
                    <DetailPanel.Properties>
                      <DetailPanel.Property label="ID" value={environment().id} mono />
                      <DetailPanel.Property
                        label="Default"
                        value={environment().isDefault ? "Yes" : "No"}
                      />
                      <DetailPanel.Property
                        label="Rules"
                        value={rules().length > 0 ? String(rules().length) : null}
                      />
                    </DetailPanel.Properties>
                  </DetailPanel.Section>
                </DetailPanel.Rail>
              </DetailPanel.Body>
            </>
          );
        }}
      </Show>
    </DetailPanel.Content>
  );
}
