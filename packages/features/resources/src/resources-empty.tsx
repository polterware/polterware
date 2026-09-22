import { Button, PageEmpty } from "@polterware/frontend-design-system";
import { Show } from "solid-js";
import type { ResourceInventoryTemplate } from "./create-templates";
import { ResourceInventoryTemplateGrid } from "./resources-template-grid";

export function ResourcesInventoryEmpty(props: {
  onCreate?: () => void;
  onCreateFromTemplate?: (template: ResourceInventoryTemplate) => void;
}) {
  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <Show
        when={props.onCreateFromTemplate}
        fallback={
          <PageEmpty
            title="No resources yet."
            action={
              props.onCreate ? (
                <Button size="sm" variant="primary" onClick={() => props.onCreate?.()}>
                  Add resource
                </Button>
              ) : undefined
            }
          />
        }
      >
        <ResourceInventoryTemplateGrid
          showLabel={false}
          onSelect={(template) => props.onCreateFromTemplate?.(template)}
        />
      </Show>
    </div>
  );
}

export function resourcesFilterEmpty() {
  return (
    <PageEmpty
      title="No resources match"
      description="Try another search or clear active filters."
    />
  );
}
