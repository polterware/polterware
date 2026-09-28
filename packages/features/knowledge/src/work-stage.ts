import type { StatusTone } from "@polterware/frontend-design-system";
import type { WorkStage, WorkspaceSpecFeature } from "@polterware/feature-shell";
import { checklistProgress } from "./spec-templates";

export const WORK_STAGES: readonly { id: WorkStage; label: string }[] = [
  { id: "shape", label: "Shape" },
  { id: "plan", label: "Plan" },
  { id: "build", label: "Build" },
  { id: "verify", label: "Verify" },
  { id: "delivered", label: "Delivered" },
];

export function isWorkStage(value: string): value is WorkStage {
  return WORK_STAGES.some((stage) => stage.id === value);
}

export function workStageLabel(stage: WorkStage): string {
  return WORK_STAGES.find((item) => item.id === stage)?.label ?? stage;
}

export function workStageTone(stage: WorkStage): StatusTone {
  if (stage === "delivered") return "healthy";
  if (stage === "verify") return "syncing";
  if (stage === "build") return "ok";
  return "muted";
}

/** Checked vs total `- [ ]` items in the Tasks artifact. */
export function featureTaskProgress(feature: WorkspaceSpecFeature): { done: number; total: number } {
  return checklistProgress(feature.tasks.bodyMarkdown);
}

/** Unchecked `- [ ]` items in the Clarify artifact. */
export function featureOpenQuestions(feature: WorkspaceSpecFeature): number {
  const progress = checklistProgress(feature.clarify.bodyMarkdown);
  return progress.total - progress.done;
}

/**
 * Stored stage wins. Without one, draft features are still being shaped, and
 * ready features are in Build until every task is checked, then in Verify.
 */
export function resolveFeatureStage(feature: WorkspaceSpecFeature): WorkStage {
  if (feature.stage) return feature.stage;
  if (feature.status === "draft") return "shape";
  const progress = featureTaskProgress(feature);
  return progress.total > 0 && progress.done === progress.total ? "verify" : "build";
}
