import { Button, PageEmpty } from "@polterware/frontend-design-system";
import { Show } from "solid-js";
import type { EnvironmentCreateTemplate } from "./create-templates";
import { EnvironmentTemplateGrid } from "./environment-template-grid";

export function EnvironmentsEmpty(props: {
  onCreate?: () => void;
  onCreateFromTemplate?: (template: EnvironmentCreateTemplate) => void;
}) {
  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <Show
        when={props.onCreateFromTemplate}
        fallback={
          <PageEmpty
            title="No environments"
            description="This project has no environments yet."
            action={
              props.onCreate ? (
                <Button size="sm" variant="primary" onClick={() => props.onCreate?.()}>
                  Add environment
                </Button>
              ) : undefined
            }
          />
        }
      >
        <EnvironmentTemplateGrid
          showLabel={false}
          onSelect={(template) => props.onCreateFromTemplate?.(template)}
        />
      </Show>
    </div>
  );
}
