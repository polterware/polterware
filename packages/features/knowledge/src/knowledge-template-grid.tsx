import {
  BookOpen02Icon,
  FileText,
  ListChecks,
  ScrollText,
  Shield,
  TemplateCardGrid,
  type TemplateCardGridItem,
} from "@polterware/frontend-design-system";
import type { KnowledgeCreateTemplate, KnowledgeTemplateKind } from "./create-templates";
import { knowledgeTemplatesForKind } from "./create-templates";

function templateIcon(kind: KnowledgeTemplateKind) {
  if (kind === "runbook") return <ScrollText size={16} />;
  if (kind === "adr") return <Shield size={16} />;
  if (kind === "card") return <ListChecks size={16} />;
  if (kind === "source") return <FileText size={16} />;
  return <BookOpen02Icon size={16} />;
}

export function KnowledgeTemplateGrid(props: {
  kind: KnowledgeTemplateKind;
  label?: string;
  showLabel?: boolean;
  onSelect: (template: KnowledgeCreateTemplate) => void;
}) {
  const items = (): TemplateCardGridItem[] =>
    knowledgeTemplatesForKind(props.kind).map((template) => ({
      id: template.id,
      title: template.title,
      description: template.description,
      tone: template.tone,
      icon: templateIcon(template.kind),
      onSelect: () => props.onSelect(template),
    }));

  return (
    <TemplateCardGrid
      label={props.showLabel === false ? undefined : (props.label ?? "Start from a template")}
      items={items()}
    />
  );
}
