import type { DitherIconTone } from "@polterware/frontend-design-system";
import type { CreateScheduleDraft } from "./create-schedule-dialog";
import type { CreateWorkflowDraft } from "./create-workflow-dialog";

export type OperationsRunDraft = {
  target: string;
  command: string;
  cwd: string;
  environment: string;
};

export type OperationsRunTemplate = {
  id: string;
  title: string;
  description: string;
  tone: DitherIconTone;
  draft: OperationsRunDraft;
};

export type OperationsWorkflowTemplate = {
  id: string;
  title: string;
  description: string;
  tone: DitherIconTone;
  draft: CreateWorkflowDraft;
};

export type OperationsScheduleTemplate = {
  id: string;
  title: string;
  description: string;
  tone: DitherIconTone;
  name: string;
  expression: string;
  timezone: string;
};

export const OPERATIONS_RUN_TEMPLATES: OperationsRunTemplate[] = [
  {
    id: "run-echo-ok",
    title: "Echo check",
    description: "Print a quick confirmation and exit.",
    tone: "execution",
    draft: {
      target: "local",
      command: "echo ok",
      cwd: "",
      environment: "Development",
    },
  },
  {
    id: "run-sleep-probe",
    title: "Sleep probe",
    description: "Safe short-lived command to verify local execution.",
    tone: "execution",
    draft: {
      target: "local",
      command: "sleep 5",
      cwd: "",
      environment: "Development",
    },
  },
  {
    id: "run-git-status",
    title: "Git status",
    description: "Inspect the working tree in the current directory.",
    tone: "integration",
    draft: {
      target: "local",
      command: "git status",
      cwd: "",
      environment: "Development",
    },
  },
];

export const OPERATIONS_WORKFLOW_TEMPLATES: OperationsWorkflowTemplate[] = [
  {
    id: "workflow-health-check",
    title: "Health check",
    description: "Local steps to verify core services are reachable.",
    tone: "observability",
    draft: {
      name: "Health check",
      description: "Verify local services and report status through polterware-core.",
    },
  },
  {
    id: "workflow-deploy-preview",
    title: "Deploy preview",
    description: "Local preview build and smoke checks before promote.",
    tone: "execution",
    draft: {
      name: "Deploy preview",
      description: "Build a preview locally and run smoke checks before promotion.",
    },
  },
  {
    id: "workflow-backup-verify",
    title: "Backup verify",
    description: "Confirm local backup artifacts exist and are readable.",
    tone: "security",
    draft: {
      name: "Backup verify",
      description: "Confirm backup artifacts exist locally and are readable.",
    },
  },
];

export const OPERATIONS_SCHEDULE_TEMPLATES: OperationsScheduleTemplate[] = [
  {
    id: "schedule-hourly",
    title: "Hourly",
    description: "Run at the top of every hour (UTC).",
    tone: "execution",
    name: "Hourly run",
    expression: "0 * * * *",
    timezone: "UTC",
  },
  {
    id: "schedule-daily",
    title: "Daily morning",
    description: "Run every day at 09:00 America/Sao_Paulo.",
    tone: "execution",
    name: "Daily morning",
    expression: "0 9 * * *",
    timezone: "America/Sao_Paulo",
  },
  {
    id: "schedule-weekdays",
    title: "Weekday standup",
    description: "Run Monday–Friday at 08:00 UTC.",
    tone: "execution",
    name: "Weekday standup",
    expression: "0 8 * * 1-5",
    timezone: "UTC",
  },
];

export function scheduleDraftFromTemplate(
  template: OperationsScheduleTemplate,
  workflowId: string,
): CreateScheduleDraft {
  return {
    name: template.name,
    workflowId,
    expression: template.expression,
    timezone: template.timezone,
  };
}
