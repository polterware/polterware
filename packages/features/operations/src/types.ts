export type RunStatus = "queued" | "running" | "success" | "failed" | "cancelled";

export type RunStep = {
  id: string;
  name: string;
  status: "queued" | "running" | "success" | "failed" | "skipped";
  duration?: string;
};

export type OperationRun = {
  id: string;
  title: string;
  target: string;
  status: RunStatus;
  startedAt: string;
  duration?: string;
  environment: string;
  command: string;
  steps: RunStep[];
  outputLines: string[];
  result: string;
  dataSource: "fixture";
};

export type WorkflowStepType = "validate" | "build" | "deploy" | "health_check" | "command" | "notify";

export type WorkflowStep = {
  id: string;
  label: string;
  type: WorkflowStepType;
  command?: string;
};

export type OperationWorkflow = {
  id: string;
  name: string;
  description: string;
  status: "draft" | "ready" | "disabled";
  trigger: "manual" | "schedule";
  steps: WorkflowStep[];
  lastRun: string;
  dataSource: "fixture";
};

export type OperationSchedule = {
  id: string;
  name: string;
  description: string;
  status: "active" | "paused" | "disabled";
  workflowId: string;
  workflowName: string;
  expression: string;
  timezone: string;
  nextRun: string;
  lastRun: string;
  environment: string;
  dataSource: "fixture";
};
