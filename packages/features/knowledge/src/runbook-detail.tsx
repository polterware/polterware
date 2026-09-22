import { Show, type JSX } from "solid-js";
import {
  DetailPanel,
  Archive,
  BookOpen02Icon,
  Copy,
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
          <DetailPanel.Header icon={<BookOpen02Icon size={16} />} title={runbook().title}>
            <DetailPanel.Actions>
              <Show when={props.onDuplicate}>
                <DetailPanel.Action label="Duplicate" onClick={() => props.onDuplicate?.()}>
                  <Copy />
                </DetailPanel.Action>
              </Show>
              <Show when={props.onArchive}>
                <DetailPanel.Action label="Archive" tone="danger" onClick={() => props.onArchive?.()}>
                  <Archive />
                </DetailPanel.Action>
              </Show>
            </DetailPanel.Actions>
          </DetailPanel.Header>
          <KnowledgeDocumentShell
            title={runbook().title}
            bodyMarkdown={props.bodyMarkdown}
            onBodyChange={props.onBodyChange}
            preamble={props.links}
          />
        </DetailPanel.Content>
      )}
    </Show>
  );
}
