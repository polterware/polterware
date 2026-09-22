import { Button, PageEmpty } from "@polterware/frontend-design-system";
import { Show } from "solid-js";
import type { KnowledgeCreateTemplate } from "./create-templates";
import { KnowledgeTemplateGrid } from "./knowledge-template-grid";

export function KnowledgeSpecsEmpty(props: {
  onCreate?: () => void;
  onCreateFromTemplate?: (template: KnowledgeCreateTemplate) => void;
}) {
  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <Show
        when={props.onCreateFromTemplate}
        fallback={
          <PageEmpty
            title="No specs yet."
            action={
              props.onCreate ? (
                <Button size="sm" variant="primary" onClick={() => props.onCreate?.()}>
                  New feature
                </Button>
              ) : undefined
            }
          />
        }
      >
        <KnowledgeTemplateGrid
          kind="feature"
          showLabel={false}
          onSelect={(template) => props.onCreateFromTemplate?.(template)}
        />
      </Show>
    </div>
  );
}

export function knowledgeFilterEmpty(kind: "specs") {
  if (kind === "specs") {
    return <PageEmpty title="No features match" description="Try another search or clear active filters." />;
  }
  return <PageEmpty title="No items match" description="Try another search or filter." />;
}
