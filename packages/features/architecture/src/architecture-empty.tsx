import type { JSX } from "solid-js";
import { Button, PageEmpty } from "@polterware/frontend-design-system";
import { Show } from "solid-js";
import type {
  ArchitectureComponentTemplate,
  ArchitectureViewTemplate,
} from "./create-templates";
import {
  ArchitectureComponentTemplateGrid,
  ArchitectureViewTemplateGrid,
} from "./architecture-template-grid";

export function ArchitectureCanvasEmpty(props: {
  onAddComponent?: () => void;
  onCreateFromTemplate?: (template: ArchitectureComponentTemplate) => void;
}) {
  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <Show
        when={props.onCreateFromTemplate}
        fallback={
          <PageEmpty
            title="No components on the canvas yet."
            action={
              props.onAddComponent ? (
                <Button size="sm" variant="primary" onClick={() => props.onAddComponent?.()}>
                  Add component
                </Button>
              ) : undefined
            }
          />
        }
      >
        <ArchitectureComponentTemplateGrid
          showLabel={false}
          onSelect={(template) => props.onCreateFromTemplate?.(template)}
        />
      </Show>
    </div>
  );
}

export function ArchitectureComponentsEmpty(props: {
  onAddComponent?: () => void;
  onCreateFromTemplate?: (template: ArchitectureComponentTemplate) => void;
}) {
  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <Show
        when={props.onCreateFromTemplate}
        fallback={
          <PageEmpty
            title="No architecture components yet."
            action={
              props.onAddComponent ? (
                <Button size="sm" variant="primary" onClick={() => props.onAddComponent?.()}>
                  Add component
                </Button>
              ) : undefined
            }
          />
        }
      >
        <ArchitectureComponentTemplateGrid
          showLabel={false}
          onSelect={(template) => props.onCreateFromTemplate?.(template)}
        />
      </Show>
    </div>
  );
}

export function ArchitectureViewsEmpty(props: {
  onCreateView?: () => void;
  onCreateFromTemplate?: (template: ArchitectureViewTemplate) => void;
}) {
  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <Show
        when={props.onCreateFromTemplate}
        fallback={
          <PageEmpty
            title="No saved views yet."
            action={
              props.onCreateView ? (
                <Button size="sm" variant="primary" onClick={() => props.onCreateView?.()}>
                  Create view
                </Button>
              ) : undefined
            }
          />
        }
      >
        <ArchitectureViewTemplateGrid
          showLabel={false}
          onSelect={(template) => props.onCreateFromTemplate?.(template)}
        />
      </Show>
    </div>
  );
}

export function architectureFilterEmpty(kind: "components" | "views"): JSX.Element {
  if (kind === "views") {
    return <PageEmpty title="No views match" description="Try another search." />;
  }
  return <PageEmpty title="No components match" description="Try another search or filter." />;
}
