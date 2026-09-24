import { Show } from "solid-js";
import {
  DetailPanel,
  Shield,
} from "@polterware/frontend-design-system";
import type { WorkspaceConstitution } from "@polterware/feature-shell";
import { KnowledgeDocumentNotFound, KnowledgeDocumentShell } from "./knowledge-document-shell";

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
