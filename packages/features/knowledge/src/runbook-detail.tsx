import { Show, type JSX } from "solid-js";
import {
  DetailPanel,
  Button,
  BookOpen02Icon,
} from "@polterware/frontend-design-system";
import type { WorkspaceRunbook } from "@polterware/feature-shell";
import { KnowledgeDocumentNotFound, KnowledgeDocumentShell } from "./knowledge-document-shell";

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
          <DetailPanel.Header />
          <KnowledgeDocumentShell
            title={runbook().title}
            icon={<BookOpen02Icon size={16} />}
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
            footerActions={
              <Show when={props.onArchive}>
                <Button variant="danger" size="sm" onClick={() => props.onArchive?.()}>Archive</Button>
              </Show>
            }
            metadataActions={
              <Show when={props.onDuplicate}>
                <Button variant="ghost" size="sm" onClick={() => props.onDuplicate?.()}>Duplicate</Button>
              </Show>
            }
            links={props.links}
          />
        </DetailPanel.Content>
      )}
    </Show>
  );
}
