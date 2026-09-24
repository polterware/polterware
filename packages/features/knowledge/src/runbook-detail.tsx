import { Show, type JSX } from "solid-js";
import {
  DetailPanel,
  BookOpen02Icon,
  StatusChip,
} from "@polterware/frontend-design-system";
import type { WorkspaceRunbook } from "@polterware/feature-shell";
import { KnowledgeDocumentNotFound, KnowledgeDocumentShell } from "./knowledge-document-shell";
import { runbookSeverityTone } from "./knowledge-document-helpers";

export function RunbookDetail(props: {
  runbook: WorkspaceRunbook | null | undefined;
  notFound?: boolean;
  bodyMarkdown: string;
  onBodyChange: (md: string) => void;
  onDuplicate?: () => void;
  onArchive?: () => void;
  onOpenComponent?: (componentId: string) => void;
  links?: JSX.Element;
}) {
  return (
    <Show
      when={props.runbook}
      fallback={
        <Show when={props.notFound}>
          <KnowledgeDocumentNotFound kindLabel="Runbook" />
        </Show>
      }
    >
      {(runbook) => (
        <DetailPanel.Content>
          <DetailPanel.Header
            icon={<BookOpen02Icon size={16} />}
            title={runbook().title}
            status={
              <StatusChip
                tone={runbookSeverityTone(runbook().severity)}
                label={runbook().severity}
              />
            }
            subtitle={["Runbook", runbook().service, runbook().owner]
              .filter(Boolean)
              .join(" · ")}
          >
            <Show when={props.onDuplicate || props.onArchive}>
              <DetailPanel.Actions>
                <DetailPanel.Overflow>
                  <Show when={props.onDuplicate}>
                    <DetailPanel.OverflowItem value="duplicate" onSelect={() => props.onDuplicate?.()}>
                      Duplicate
                    </DetailPanel.OverflowItem>
                  </Show>
                  <Show when={props.onArchive}>
                    <DetailPanel.OverflowItem value="archive" danger onSelect={() => props.onArchive?.()}>
                      Archive
                    </DetailPanel.OverflowItem>
                  </Show>
                </DetailPanel.Overflow>
              </DetailPanel.Actions>
            </Show>
          </DetailPanel.Header>
          <KnowledgeDocumentShell
            title={runbook().title}
            bodyMarkdown={props.bodyMarkdown}
            onBodyChange={props.onBodyChange}
            properties={
              <DetailPanel.Properties variant="inline">
                <DetailPanel.Property label="Service" value={runbook().service} />
                <DetailPanel.Property
                  label="Component"
                  value={runbook().componentName}
                />
                <DetailPanel.Property label="Resource" value={runbook().resourceName} />
                <DetailPanel.Property label="Last validated" value={runbook().lastValidated} />
              </DetailPanel.Properties>
            }
            links={props.links}
          />
        </DetailPanel.Content>
      )}
    </Show>
  );
}
