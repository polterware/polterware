import {
  Database,
  HardDrive,
  Package,
  TemplateCardGrid,
  type TemplateCardGridItem,
} from "@polterware/frontend-design-system";
import type { ResourceInventoryTemplate } from "./create-templates";
import { RESOURCE_INVENTORY_TEMPLATES } from "./create-templates";

function inventoryIcon(template: ResourceInventoryTemplate) {
  if (template.draft.type === "Database") return <Database size={16} />;
  if (template.draft.type === "Cache") return <HardDrive size={16} />;
  return <Package size={16} />;
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
