import { Show, createMemo } from "solid-js";
import {
  Archive,
  Check,
  DetailPanel,
  FileText,
  StatusChip,
  Tabs,
} from "@polterware/frontend-design-system";
import type {
  WorkspaceSpecArtifactKind,
  WorkspaceSpecFeature,
} from "@polterware/feature-shell";
import { KnowledgeDocumentNotFound, KnowledgeDocumentShell } from "./knowledge-document-shell";
import { specStatusTone } from "./knowledge-document-helpers";
import { checklistHasOpenItems } from "./spec-templates";

const ARTIFACT_TABS = [
  { id: "spec", label: "Spec" },
  { id: "plan", label: "Plan" },
  { id: "tasks", label: "Tasks" },
  { id: "clarify", label: "Clarify" },
  { id: "checklist", label: "Checklist" },
] as const;

function normalizeArtifact(raw?: string): WorkspaceSpecArtifactKind {
  if (raw === "plan" || raw === "tasks" || raw === "clarify" || raw === "checklist") return raw;
  return "spec";
}

export function SpecFeatureDetail(props: {
  feature: WorkspaceSpecFeature | null | undefined;
  notFound?: boolean;
  artifact?: WorkspaceSpecArtifactKind;
  bodyMarkdown: string;
  checklistBody?: string;
  onArtifactChange?: (artifact: WorkspaceSpecArtifactKind) => void;
  onBodyChange: (md: string) => void;
  onArchive?: () => void;
  onSetStatus?: (status: "draft" | "ready") => void;
}) {
  const artifact = createMemo(() => normalizeArtifact(props.artifact));
  const artifactLabel = createMemo(
    () => ARTIFACT_TABS.find((tab) => tab.id === artifact())?.label ?? "Spec",
  );
  const checklistIncomplete = createMemo(() => {
    const feature = props.feature;
    if (!feature) return false;
    return checklistHasOpenItems(props.checklistBody ?? feature.checklist.bodyMarkdown);
  });

  return (
    <Show
      when={props.feature}
      fallback={
        <Show when={props.notFound}>
          <KnowledgeDocumentNotFound kindLabel="Feature" />
        </Show>
      }
    >
      {(feature) => (
        <DetailPanel.Content>
          <DetailPanel.Header icon={<FileText size={16} />} title={feature().title}>
            <DetailPanel.Actions>
              <Show when={props.onSetStatus}>
                <DetailPanel.Action
                  label={feature().status === "ready" ? "Mark draft" : "Mark ready"}
                  onClick={() =>
                    props.onSetStatus?.(feature().status === "ready" ? "draft" : "ready")
                  }
                >
                  <Check />
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
            title={`${feature().title} · ${artifactLabel()}`}
            bodyMarkdown={props.bodyMarkdown}
            onBodyChange={props.onBodyChange}
            preamble={
              <div class="flex flex-col gap-3 px-4 py-3">
                <div class="flex flex-wrap items-center gap-2">
                  <StatusChip tone={specStatusTone(feature().status)} label={feature().status} />
                  <Show when={checklistIncomplete() && feature().status !== "ready"}>
                    <StatusChip tone="warn" label="checklist open" />
                  </Show>
                  <span class="text-[12px] text-[var(--color-text-muted)] font-mono">
                    {feature().featureKey}
                  </span>
                </div>
                <Tabs
                  items={[...ARTIFACT_TABS]}
                  value={artifact()}
                  onChange={(next) => props.onArtifactChange?.(normalizeArtifact(next))}
                  aria-label="Feature artifacts"
                />
              </div>
            }
          />
        </DetailPanel.Content>
      )}
    </Show>
  );
}
