import {
  Database,
  GitBranch,
  HardDrive,
  Laptop,
  Package,
  Server,
  SquareTerminal,
  TemplateCardGrid,
  type TemplateCardGridItem,
} from "@polterware/frontend-design-system";
import type {
  ResourceInventoryTemplate,
  ResourceNodeTemplate,
  ResourceProcessTemplate,
} from "./create-templates";
import {
  RESOURCE_INVENTORY_TEMPLATES,
  RESOURCE_NODE_TEMPLATES,
  RESOURCE_PROCESS_TEMPLATES,
} from "./create-templates";

function inventoryIcon(template: ResourceInventoryTemplate) {
  if (template.draft.type === "Database") return <Database size={16} />;
  if (template.draft.type === "Cache") return <HardDrive size={16} />;
  return <Package size={16} />;
}

function nodeIcon(template: ResourceNodeTemplate) {
  if (template.id === "node-jump-box") return <HardDrive size={16} />;
  if (template.id === "node-vps") return <Server size={16} />;
  return <Laptop size={16} />;
}

function processIcon(template: ResourceProcessTemplate) {
  if (template.id === "process-git-status") return <GitBranch size={16} />;
  return <SquareTerminal size={16} />;
}

export function ResourceInventoryTemplateGrid(props: {
  label?: string;
  showLabel?: boolean;
  onSelect: (template: ResourceInventoryTemplate) => void;
}) {
  const items = (): TemplateCardGridItem[] =>
    RESOURCE_INVENTORY_TEMPLATES.map((template) => ({
      id: template.id,
      title: template.title,
      description: template.description,
      tone: template.tone,
      brand: template.brand,
      icon: inventoryIcon(template),
      onSelect: () => props.onSelect(template),
    }));

  return (
    <TemplateCardGrid
      label={props.showLabel === false ? undefined : (props.label ?? "Start from a template")}
      items={items()}
    />
  );
}

export function ResourceNodeTemplateGrid(props: {
  label?: string;
  showLabel?: boolean;
  onSelect: (template: ResourceNodeTemplate) => void;
}) {
  const items = (): TemplateCardGridItem[] =>
    RESOURCE_NODE_TEMPLATES.map((template) => ({
      id: template.id,
      title: template.title,
      description: template.description,
      tone: template.tone,
      icon: nodeIcon(template),
      onSelect: () => props.onSelect(template),
    }));

  return (
    <TemplateCardGrid
      label={props.showLabel === false ? undefined : (props.label ?? "Start from a template")}
      items={items()}
    />
  );
}

export function ResourceProcessTemplateGrid(props: {
  label?: string;
  showLabel?: boolean;
  onSelect: (template: ResourceProcessTemplate) => void;
}) {
  const items = (): TemplateCardGridItem[] =>
    RESOURCE_PROCESS_TEMPLATES.map((template) => ({
      id: template.id,
      title: template.title,
      description: template.description,
      tone: template.tone,
      icon: processIcon(template),
      onSelect: () => props.onSelect(template),
    }));

  return (
    <TemplateCardGrid
      label={props.showLabel === false ? undefined : (props.label ?? "Start from a template")}
      items={items()}
    />
  );
}
