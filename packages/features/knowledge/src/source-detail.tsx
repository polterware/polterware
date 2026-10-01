import { Show, type JSX } from "solid-js";
import {
  DetailPanel,
  Button,
  FileText,
} from "@polterware/frontend-design-system";
import type { WorkspaceSource } from "@polterware/feature-shell";
import { EditablePath } from "./editable-path";
import { KnowledgeDocumentNotFound, KnowledgeDocumentShell } from "./knowledge-document-shell";

export function SourceDetail(props: {
  source: WorkspaceSource | null | undefined;
  bodyMarkdown: string;
  pathValue: string;
  onBodyChange: (markdown: string) => void;
  onPathChange?: (path: string) => void;
  onArchive?: () => void;
  links?: JSX.Element;
}) {
  return (
    <Show
      when={props.source}
      fallback={<KnowledgeDocumentNotFound kindLabel="Source" />}
    >
      {(source) => (
        <DetailPanel.Content>
          <DetailPanel.Header />
          <KnowledgeDocumentShell
            title={source().title}
            icon={<FileText size={16} />}
            bodyMarkdown={props.bodyMarkdown}
            onBodyChange={props.onBodyChange}
            properties={
              <DetailPanel.Properties variant="inline">
                <DetailPanel.Property
                  label="Path"
                  value={<EditablePath value={props.pathValue} onChange={props.onPathChange} />}
                />
              </DetailPanel.Properties>
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
