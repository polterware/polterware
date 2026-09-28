import { Show, createMemo } from "solid-js";
import {
  Check,
  DetailPanel,
  FileText,
  Select,
  StatusChip,
  Tabs,
} from "@polterware/frontend-design-system";
import type {
  WorkStage,
  WorkspaceSpecArtifactKind,
  WorkspaceSpecFeature,
} from "@polterware/feature-shell";
import { KnowledgeDocumentNotFound, KnowledgeDocumentShell } from "./knowledge-document-shell";
import { checklistHasOpenItems, checklistProgress } from "./spec-templates";
import { WORK_STAGES, isWorkStage, resolveFeatureStage } from "./work-stage";

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
  onSetStage?: (stage: WorkStage) => void;
}) {
  const artifact = createMemo(() => normalizeArtifact(props.artifact));
  const artifactLabel = createMemo(
    () => ARTIFACT_TABS.find((tab) => tab.id === artifact())?.label ?? "Spec",
  );
  const checklistBody = createMemo(
    () => props.checklistBody ?? props.feature?.checklist.bodyMarkdown ?? "",
  );
  const checklistIncomplete = createMemo(() => checklistHasOpenItems(checklistBody()));
  const progress = createMemo(() => checklistProgress(checklistBody()));

  return (
    <Show
      when={props.feature}
      fallback={
        <Show when={props.notFound}>
          <KnowledgeDocumentNotFound kindLabel="Feature" />
        </Show>
      }
    >
      {(feature) => {
        const isReady = () => feature().status === "ready";
        return (
          <DetailPanel.Content>
            <DetailPanel.Header
              icon={<FileText size={16} />}
              title={feature().title}
            >
              <Show when={props.onSetStatus || props.onArchive}>
                <DetailPanel.Actions>
                  <Show when={props.onSetStatus}>
                    <DetailPanel.PrimaryAction
                      label={isReady() ? "Mark draft" : "Mark ready"}
                      icon={isReady() ? undefined : <Check size={14} aria-hidden />}
                      disabled={!isReady() && checklistIncomplete()}
                      disabledReason="Check every Checklist item before marking ready."
                      onClick={() => props.onSetStatus?.(isReady() ? "draft" : "ready")}
                    />
                  </Show>
                  <Show when={props.onArchive}>
                    <DetailPanel.Overflow>
                      <DetailPanel.OverflowItem value="archive" danger onSelect={() => props.onArchive?.()}>
                        Archive
                      </DetailPanel.OverflowItem>
                    </DetailPanel.Overflow>
                  </Show>
                </DetailPanel.Actions>
              </Show>
            </DetailPanel.Header>
            <KnowledgeDocumentShell
              title={`${feature().title} · ${artifactLabel()}`}
              bodyMarkdown={props.bodyMarkdown}
              onBodyChange={props.onBodyChange}
              tabs={
                <Tabs
                  items={[...ARTIFACT_TABS]}
                  value={artifact()}
                  onChange={(next) => props.onArtifactChange?.(normalizeArtifact(next))}
                  aria-label="Feature artifacts"
                />
              }
              properties={
                <DetailPanel.Properties variant="inline">
                  <DetailPanel.Property label="Feature key" value={feature().featureKey} mono />
                  <DetailPanel.Property
                    label="Stage"
                    value={
                      <Select
                        aria-label="Work stage"
                        class="w-32"
                        options={WORK_STAGES.map((stage) => ({ value: stage.id, label: stage.label }))}
                        value={resolveFeatureStage(feature())}
                        disabled={!props.onSetStage}
                        onChange={(next) => {
                          if (isWorkStage(next) && next !== resolveFeatureStage(feature())) {
                            props.onSetStage?.(next);
                          }
                        }}
                      />
                    }
                  />
                  <DetailPanel.Property
                    label="Checklist"
                    value={
                      progress().total > 0 ? (
                        <StatusChip
                          tone={checklistIncomplete() ? "warn" : "healthy"}
                          label={`${progress().done}/${progress().total}`}
                        />
                      ) : null
                    }
                  />
                </DetailPanel.Properties>
              }
            />
          </DetailPanel.Content>
        );
      }}
    </Show>
  );
}
