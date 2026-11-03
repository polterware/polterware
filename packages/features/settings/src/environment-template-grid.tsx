import {
  Server,
  TemplateCardGrid,
  type TemplateCardGridItem,
} from "@polterware/frontend-design-system";
import type { EnvironmentCreateTemplate } from "./create-templates";
import { ENVIRONMENT_CREATE_TEMPLATES } from "./create-templates";

export function EnvironmentTemplateGrid(props: {
  label?: string;
  showLabel?: boolean;
  onSelect: (template: EnvironmentCreateTemplate) => void;
}) {
  const items = (): TemplateCardGridItem[] =>
    ENVIRONMENT_CREATE_TEMPLATES.map((template) => ({
      id: template.id,
      title: template.title,
      description: template.description,
      tone: template.tone,
      icon: <Server size={16} />,
      onSelect: () => props.onSelect(template),
    }));

  return (
    <TemplateCardGrid
      label={props.showLabel === false ? undefined : (props.label ?? "Start from a template")}
      items={items()}
    />
  );
}
