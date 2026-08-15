import { Show } from "solid-js";
import {
  DetailPanel,
  BookOpen02Icon,
  Copy,
  Play,
  SquareTerminal,
} from "@polterware/frontend-design-system";
import type { WorkspaceRunbook } from "@polterware/feature-shell";
import { KnowledgeDocumentNotFound, KnowledgeDocumentShell } from "./knowledge-document-shell";

export function RunbookDetail(props: {
  runbook: WorkspaceRunbook | null | undefined;
  notFound?: boolean;
  bodyMarkdown: string;
  onBodyChange: (md: string) => void;
  onRun?: () => void;
  onDuplicate?: () => void;
  onOpenTerminal?: () => void;
  onOpenComponent?: (componentId: string) => void;
  onOpenOperation?: (operationId: string) => void;
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
              <Show when={props.onRun}>
                <DetailPanel.Action label="Run" tone="primary" onClick={() => props.onRun?.()}>
                  <Play />
                </DetailPanel.Action>
              </Show>
              <Show when={props.onDuplicate}>
                <DetailPanel.Action label="Duplicate" onClick={() => props.onDuplicate?.()}>
                  <Copy />
                </DetailPanel.Action>
              </Show>
              <Show when={props.onOpenTerminal}>
                <DetailPanel.Action
                  label="Open terminal"
                  onClick={() => props.onOpenTerminal?.()}
                >
                  <SquareTerminal />
                </DetailPanel.Action>
              </Show>
            </DetailPanel.Actions>
          </DetailPanel.Header>
          <KnowledgeDocumentShell
            title={runbook().title}
            bodyMarkdown={props.bodyMarkdown}
            onBodyChange={props.onBodyChange}
          />
        </DetailPanel.Content>
      )}
    </Show>
  );
}
