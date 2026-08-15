import { For, Show } from "solid-js";
import {
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
}) {
  return (
    <DetailPanel.Content>
      <Show when={props.loading}>
        <DetailPanel.Header title="Loading environment" />
        <DetailPanel.Body layout="sections">
          <DetailPanel.Grid>
            <DetailPanel.Section>
              <PageLoading label="Loading environment…" />
            </DetailPanel.Section>
          </DetailPanel.Grid>
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
        <DetailPanel.Body layout="sections">
          <DetailPanel.Grid>
            <DetailPanel.Section>
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
            </DetailPanel.Section>
          </DetailPanel.Grid>
        </DetailPanel.Body>
      </Show>

      <Show when={!props.loading && props.environment}>
        {(environment) => {
          const rules = () => environment().protectionRules ?? [];

          return (
            <>
              <DetailPanel.Header
                icon={<Server size={16} />}
                title={environment().name}
              />

              <DetailPanel.Body layout="sections">
                <DetailPanel.Grid>
                  <DetailPanel.Metric
                    span={3}
                    label="Kind"
                    value={
                      <StatusChip
                        tone={environmentKindTone(environment().kind)}
                        label={environmentKindLabel(environment().kind)}
                      />
                    }
                  />
                  <DetailPanel.Metric
                    span={3}
                    label="Protection"
                    value={
                      <StatusChip
                        tone={protectionTone(environment().protection)}
                        label={protectionLabel(environment().protection)}
                      />
                    }
                  />

                  <DetailPanel.Section title="Environment" span={6}>
                    <DetailPanel.Metadata>
                      <DetailPanel.MetadataItem
                        label="Environment ID"
                        value={environment().id}
                        mono
                      />
                      <Show when={environment().description}>
                        <DetailPanel.MetadataItem
                          label="Summary"
                          value={environment().description!}
                        />
                      </Show>
                    </DetailPanel.Metadata>
                  </DetailPanel.Section>

                  <Show when={rules().length > 0}>
                    <DetailPanel.Section title="Protection rules" span={6}>
                      <ul
                        class="m-0 list-disc space-y-1 pl-4 text-[12.5px] text-[var(--color-text-secondary)]"
                        aria-label="Protection rules"
                      >
                        <For each={rules()}>{(rule) => <li>{rule}</li>}</For>
                      </ul>
                    </DetailPanel.Section>
                  </Show>
                </DetailPanel.Grid>
              </DetailPanel.Body>
            </>
          );
        }}
      </Show>
    </DetailPanel.Content>
  );
}
