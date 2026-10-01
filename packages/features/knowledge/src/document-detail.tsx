import { Show, type JSX } from "solid-js";
import {
  DetailPanel,
  Button,
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
  links?: JSX.Element;
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
          <DetailPanel.Header />
          <KnowledgeDocumentShell
            title={doc().title}
            icon={<Book03Icon size={16} />}
            bodyMarkdown={props.bodyMarkdown}
            onBodyChange={props.onBodyChange}
            properties={
              <Show when={doc().tags.length > 0 || doc().environment}>
                <DetailPanel.Properties variant="inline">
                  <DetailPanel.Property label="Environment" value={doc().environment} />
                  <DetailPanel.Property
                    label="Tags"
                    value={doc().tags.length > 0 ? doc().tags.join(", ") : null}
                  />
                </DetailPanel.Properties>
              </Show>
            }
            footerActions={
              <Show when={props.onArchive}>
                <Button variant="danger" size="sm" onClick={() => props.onArchive?.()}>Archive</Button>
              </Show>
            }
            links={props.links}
          />
        </DetailPanel.Content>
      )}
    </Show>
  );
}
