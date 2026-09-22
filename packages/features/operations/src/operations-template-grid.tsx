import {
  CalendarClock,
  GitBranch,
  SquareTerminal,
  TemplateCardGrid,
  Workflow,
  type TemplateCardGridItem,
} from "@polterware/frontend-design-system";
import type {
  OperationsRunTemplate,
  OperationsScheduleTemplate,
  OperationsWorkflowTemplate,
} from "./create-templates";
import {
  OPERATIONS_RUN_TEMPLATES,
  OPERATIONS_SCHEDULE_TEMPLATES,
  OPERATIONS_WORKFLOW_TEMPLATES,
} from "./create-templates";

function runIcon(template: OperationsRunTemplate) {
  if (template.id === "run-git-status") return <GitBranch size={16} />;
  return <SquareTerminal size={16} />;
}

export function OperationsRunTemplateGrid(props: {
  label?: string;
  showLabel?: boolean;
  onSelect: (template: OperationsRunTemplate) => void;
}) {
  const items = (): TemplateCardGridItem[] =>
    OPERATIONS_RUN_TEMPLATES.map((template) => ({
      id: template.id,
      title: template.title,
      description: template.description,
      tone: template.tone,
      icon: runIcon(template),
      onSelect: () => props.onSelect(template),
    }));

  return (
    <TemplateCardGrid
      label={props.showLabel === false ? undefined : (props.label ?? "Start from a template")}
      items={items()}
    />
  );
}

export function OperationsWorkflowTemplateGrid(props: {
  label?: string;
  showLabel?: boolean;
  onSelect: (template: OperationsWorkflowTemplate) => void;
}) {
  const items = (): TemplateCardGridItem[] =>
    OPERATIONS_WORKFLOW_TEMPLATES.map((template) => ({
      id: template.id,
      title: template.title,
      description: template.description,
      tone: template.tone,
      icon: <Workflow size={16} />,
      onSelect: () => props.onSelect(template),
    }));

  return (
    <TemplateCardGrid
      label={props.showLabel === false ? undefined : (props.label ?? "Start from a template")}
      items={items()}
    />
  );
}

export function OperationsScheduleTemplateGrid(props: {
  label?: string;
  showLabel?: boolean;
  onSelect: (template: OperationsScheduleTemplate) => void;
}) {
  const items = (): TemplateCardGridItem[] =>
    OPERATIONS_SCHEDULE_TEMPLATES.map((template) => ({
      id: template.id,
      title: template.title,
      description: template.description,
      tone: template.tone,
      icon: <CalendarClock size={16} />,
      onSelect: () => props.onSelect(template),
    }));

  return (
    <TemplateCardGrid
      label={props.showLabel === false ? undefined : (props.label ?? "Start from a template")}
      items={items()}
    />
  );
}
