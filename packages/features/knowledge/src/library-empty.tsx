import { Button, PageEmpty } from "@polterware/frontend-design-system";
import { Show } from "solid-js";
import type { KnowledgeCreateTemplate } from "./create-templates";
import { KnowledgeTemplateGrid } from "./knowledge-template-grid";
import type { LibraryCategory } from "./library-row";

export function LibraryEmpty(props: {
  category: "all" | LibraryCategory;
  onCreate?: () => void;
  onCreateFromTemplate?: (template: KnowledgeCreateTemplate) => void;
}) {
  const blankTitle = () => {
    if (props.category === "adr") return "New ADR";
    if (props.category === "runbook") return "New runbook";
    if (props.category === "doc") return "New document";
    return "New library item";
  };

  const emptyTitle = () => {
    if (props.category === "adr") return "No ADRs yet.";
    if (props.category === "runbook") return "No runbooks yet.";
    if (props.category === "doc") return "No documents yet.";
    return "No library items yet.";
  };

  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <Show
        when={props.onCreateFromTemplate && props.category !== "all"}
        fallback={
          <PageEmpty
            title={emptyTitle()}
            description={
              props.category === "all"
                ? "Docs, ADRs, and runbooks live here. Create one to start the local library."
                : undefined
            }
            action={
              props.onCreate ? (
                <Button size="sm" variant="primary" onClick={() => props.onCreate?.()}>
                  {blankTitle()}
                </Button>
              ) : undefined
            }
          />
        }
      >
        <KnowledgeTemplateGrid
          kind={props.category as LibraryCategory}
          showLabel={false}
          onSelect={(template) => props.onCreateFromTemplate?.(template)}
        />
      </Show>
    </div>
  );
}

export function libraryFilterEmpty() {
  return (
    <PageEmpty
      title="No library items match"
      description="Try another search or clear active filters."
    />
  );
}
