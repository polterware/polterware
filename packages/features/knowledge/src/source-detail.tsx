import { Show, type JSX } from "solid-js";
import {
  DetailPanel,
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
          <DetailPanel.Header
            icon={<FileText size={16} />}
            title={source().title}
            subtitle={["Source", source().owner, `Updated ${source().updatedAt}`]
              .filter(Boolean)
              .join(" · ")}
          >
            <Show when={props.onArchive}>
              <DetailPanel.Actions>
                <DetailPanel.Overflow>
                  <DetailPanel.OverflowItem value="archive" danger onSelect={() => props.onArchive?.()}>
                    Archive
                  </DetailPanel.OverflowItem>
                </DetailPanel.Overflow>
              </DetailPanel.Actions>
            </Show>
          </DetailPanel.Header>
          <KnowledgeDocumentShell
            title={source().title}
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
            links={props.links}
          />
        </DetailPanel.Content>
      )}
    </Show>
  );
}
