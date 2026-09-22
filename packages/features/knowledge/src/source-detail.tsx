import { Show, type JSX } from "solid-js";
import {
  Archive,
  DetailPanel,
  Field,
  FileText,
  Input,
} from "@polterware/frontend-design-system";
import type { WorkspaceSource } from "@polterware/feature-shell";
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
          <DetailPanel.Header icon={<FileText size={16} />} title={source().title}>
            <DetailPanel.Actions>
              <Show when={props.onArchive}>
                <DetailPanel.Action label="Archive" tone="danger" onClick={() => props.onArchive?.()}>
                  <Archive />
                </DetailPanel.Action>
              </Show>
            </DetailPanel.Actions>
          </DetailPanel.Header>
          <KnowledgeDocumentShell
            title={source().title}
            bodyMarkdown={props.bodyMarkdown}
            onBodyChange={props.onBodyChange}
            preamble={
              <>
                <div class="px-4 py-3">
                  <Field label="Path" hint="Relative to the project root">
                    <Input
                      size="sm"
                      class="font-mono"
                      value={props.pathValue}
                      onInput={(event) => props.onPathChange?.(event.currentTarget.value)}
                      disabled={!props.onPathChange}
                    />
                  </Field>
                </div>
                <Show when={props.links}>{props.links}</Show>
              </>
            }
          />
        </DetailPanel.Content>
      )}
    </Show>
  );
}
