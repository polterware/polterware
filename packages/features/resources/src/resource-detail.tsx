import { Show } from "solid-js";
import {
  DetailPanel,
  Network,
  Package,
  StatusChip,
} from "@polterware/frontend-design-system";
import type { Resource } from "./types";
import { healthTone } from "./resource-status";

export function ResourceDetail(props: {
  resource: Resource;
  onOpenArchitecture?: (id: string) => void;
}) {
  const resource = () => props.resource;
  return (
    <DetailPanel.Content>
      <DetailPanel.Header
        icon={<Package size={16} />}
        title={resource().name}
        status={<StatusChip tone={healthTone(resource().status)} label={resource().status} />}
        subtitle={[resource().type, resource().provider, resource().environment]
          .filter(Boolean)
          .join(" · ")}
      >
        <Show when={props.onOpenArchitecture}>
          <DetailPanel.Actions>
            <DetailPanel.PrimaryAction
              label="Open architecture"
              icon={<Network size={14} aria-hidden />}
              onClick={() => props.onOpenArchitecture?.(resource().id)}
            />
          </DetailPanel.Actions>
        </Show>
      </DetailPanel.Header>
      <DetailPanel.Body>
        <DetailPanel.Main>
          <DetailPanel.Section title="Overview">
            <Show
              when={resource().description.trim()}
              fallback={
                <p class="pw-detail-panel__text" data-empty>
                  No description yet.
                </p>
              }
            >
              <p class="pw-detail-panel__text">{resource().description}</p>
            </Show>
          </DetailPanel.Section>
          <Show when={resource().tags.length > 0}>
            <DetailPanel.Section title="Tags">
              <DetailPanel.LinkedEntities
                items={resource().tags.map((tag) => ({ id: tag, title: tag }))}
              />
            </DetailPanel.Section>
          </Show>
        </DetailPanel.Main>
        <DetailPanel.Rail>
          <DetailPanel.Section title="Properties">
            <DetailPanel.Properties>
              <DetailPanel.Property label="Type" value={resource().type} />
              <DetailPanel.Property label="Provider" value={resource().provider} />
              <DetailPanel.Property label="Environment" value={resource().environment} />
              <DetailPanel.Property label="Region" value={resource().region} />
              <DetailPanel.Property label="Address" value={resource().address} mono />
              <DetailPanel.Property
                label="Source"
                value={resource().dataSource === "core" ? "Local store" : "Sample data"}
              />
            </DetailPanel.Properties>
          </DetailPanel.Section>
        </DetailPanel.Rail>
      </DetailPanel.Body>
    </DetailPanel.Content>
  );
}
