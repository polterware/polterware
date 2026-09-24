import { Show, type JSX } from "solid-js";
import {
  DetailPanel,
  Book03Icon,
  StatusChip,
  type StatusTone,
} from "@polterware/frontend-design-system";
import type { WorkspaceDocument } from "@polterware/feature-shell";
import { KnowledgeDocumentNotFound, KnowledgeDocumentShell } from "./knowledge-document-shell";
import { documentStatusTone } from "./knowledge-document-helpers";

export function DocumentDetail(props: {
  document: WorkspaceDocument | null | undefined;
  notFound?: boolean;
  /** Library kind shown in the header subtitle (Doc, ADR). */
  kindLabel?: string;
  /** Overrides the document status chip (ADRs use their own lifecycle). */
  status?: { label: string; tone: StatusTone };
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
      {(doc) => {
        const status = () =>
          props.status ?? { label: doc().status, tone: documentStatusTone(doc().status) };
        return (
          <DetailPanel.Content>
            <DetailPanel.Header
              icon={<Book03Icon size={16} />}
              title={doc().title}
              status={<StatusChip tone={status().tone} label={status().label} />}
              subtitle={[props.kindLabel ?? "Doc", doc().owner, `Updated ${doc().updatedAt}`]
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
              title={doc().title}
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
              links={props.links}
            />
          </DetailPanel.Content>
        );
      }}
    </Show>
  );
}
