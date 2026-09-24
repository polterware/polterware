import { Show } from "solid-js";
import {
  DetailPanel,
  Shield,
  StatusChip,
} from "@polterware/frontend-design-system";
import type { WorkspaceConstitution } from "@polterware/feature-shell";
import { KnowledgeDocumentNotFound, KnowledgeDocumentShell } from "./knowledge-document-shell";
import { libraryStatusTone } from "./knowledge-document-helpers";

export function ConstitutionDetail(props: {
  constitution: WorkspaceConstitution | null | undefined;
  notFound?: boolean;
  bodyMarkdown: string;
  onBodyChange: (md: string) => void;
  onArchive?: () => void;
}) {
  return (
    <Show
      when={props.constitution}
      fallback={
        <Show when={props.notFound}>
          <KnowledgeDocumentNotFound kindLabel="Constitution" />
        </Show>
      }
    >
      {(constitution) => (
        <DetailPanel.Content>
          <DetailPanel.Header
            icon={<Shield size={16} />}
            title={constitution().title}
            status={
              constitution().status ? (
                <StatusChip
                  tone={libraryStatusTone(constitution().status)}
                  label={constitution().status}
                />
              ) : undefined
            }
            subtitle={["Constitution", constitution().owner, `Updated ${constitution().updatedAt}`]
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
            title={constitution().title}
            bodyMarkdown={props.bodyMarkdown}
            onBodyChange={props.onBodyChange}
          />
        </DetailPanel.Content>
      )}
    </Show>
  );
}
