import {
  Boxes,
  Cog,
  Database,
  HardDrive,
  LayoutDashboard,
  TemplateCardGrid,
  Webhook,
  type TemplateCardGridItem,
} from "@polterware/frontend-design-system";
import type {
  ArchitectureComponentTemplate,
  ArchitectureViewTemplate,
} from "./create-templates";
import {
  ARCHITECTURE_COMPONENT_TEMPLATES,
  ARCHITECTURE_VIEW_TEMPLATES,
} from "./create-templates";

function componentIcon(template: ArchitectureComponentTemplate) {
  if (template.draft.subtype === "postgres") return <Database size={16} />;
  if (template.draft.subtype === "redis") return <HardDrive size={16} />;
  if (template.draft.typeId === "worker" || template.draft.type === "Worker") return <Cog size={16} />;
  if (template.draft.typeId === "api" || template.draft.type === "API") return <Webhook size={16} />;
  return <Boxes size={16} />;
}

function viewIcon(template: ArchitectureViewTemplate) {
  if (template.id === "view-data-plane") return <Database size={16} />;
  if (template.id === "view-applications") return <Boxes size={16} />;
  return <LayoutDashboard size={16} />;
}

export function ArchitectureComponentTemplateGrid(props: {
  label?: string;
  showLabel?: boolean;
  onSelect: (template: ArchitectureComponentTemplate) => void;
}) {
  const items = (): TemplateCardGridItem[] =>
    ARCHITECTURE_COMPONENT_TEMPLATES.map((template) => ({
      id: template.id,
      title: template.title,
      description: template.description,
      tone: template.tone,
      brand: template.brand,
      icon: componentIcon(template),
      onSelect: () => props.onSelect(template),
    }));

  return (
    <TemplateCardGrid
      label={props.showLabel === false ? undefined : (props.label ?? "Start from a template")}
      items={items()}
    />
  );
}

export function ArchitectureViewTemplateGrid(props: {
  label?: string;
  showLabel?: boolean;
  onSelect: (template: ArchitectureViewTemplate) => void;
}) {
  const items = (): TemplateCardGridItem[] =>
    ARCHITECTURE_VIEW_TEMPLATES.map((template) => ({
      id: template.id,
      title: template.title,
      description: template.description,
      tone: template.tone,
      icon: viewIcon(template),
      onSelect: () => props.onSelect(template),
    }));

  return (
    <TemplateCardGrid
      label={props.showLabel === false ? undefined : (props.label ?? "Start from a template")}
      items={items()}
    />
  );
}
