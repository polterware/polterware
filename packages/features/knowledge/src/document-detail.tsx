import { Show } from "solid-js";
import {
  DetailPanel,
  Archive,
  Book03Icon,
} from "@polterware/frontend-design-system";
import type { WorkspaceDocument } from "@polterware/feature-shell";
import { KnowledgeDocumentNotFound, KnowledgeDocumentShell } from "./knowledge-document-shell";

export function DocumentDetail(props: {
  document: WorkspaceDocument | null | undefined;
  notFound?: boolean;
  bodyMarkdown: string;
  onBodyChange: (md: string) => void;
  onArchive?: () => void;
  onOpenComponent?: (componentId: string) => void;
}) {
  return (
    <Show
      when={props.document}
      fallback={
        <Show when={props.notFound}>
          <KnowledgeDocumentNotFound kindLabel="Document" />
        </Show>
      }
    >
      {(doc) => (
        <DetailPanel.Content>
          <DetailPanel.Header icon={<Book03Icon size={16} />} title={doc().title}>
            <DetailPanel.Actions>
              <Show when={props.onArchive}>
                <DetailPanel.Action label="Archive" onClick={() => props.onArchive?.()}>
                  <Archive />
                </DetailPanel.Action>
              </Show>
            </DetailPanel.Actions>
          </DetailPanel.Header>
          <KnowledgeDocumentShell
            title={doc().title}
            bodyMarkdown={props.bodyMarkdown}
            onBodyChange={props.onBodyChange}
          />
        </DetailPanel.Content>
      )}
    </Show>
  );
}
