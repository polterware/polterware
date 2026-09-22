import { Button, PageEmpty } from "@polterware/frontend-design-system";
import { Show } from "solid-js";
import type {
  ResourceInventoryTemplate,
  ResourceNodeTemplate,
  ResourceProcessTemplate,
} from "./create-templates";
import {
  ResourceInventoryTemplateGrid,
  ResourceNodeTemplateGrid,
  ResourceProcessTemplateGrid,
} from "./resources-template-grid";

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

export function ResourcesNodesEmpty(props: {
  onCreateNode?: () => void;
  onCreateFromTemplate?: (template: ResourceNodeTemplate) => void;
}) {
  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <Show
        when={props.onCreateFromTemplate}
        fallback={
          <PageEmpty
            title="No SSH hosts yet."
            action={
              props.onCreateNode ? (
                <Button size="sm" variant="primary" onClick={() => props.onCreateNode?.()}>
                  Add SSH host
                </Button>
              ) : undefined
            }
          />
        }
      >
        <ResourceNodeTemplateGrid
          showLabel={false}
          onSelect={(template) => props.onCreateFromTemplate?.(template)}
        />
      </Show>
    </div>
  );
}

export function ResourcesProcessesEmpty(props: {
  onCreateProcess?: () => void;
  onCreateFromTemplate?: (template: ResourceProcessTemplate) => void;
}) {
  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <Show
        when={props.onCreateFromTemplate}
        fallback={
          <PageEmpty
            title="No local processes."
            action={
              props.onCreateProcess ? (
                <Button size="sm" variant="primary" onClick={() => props.onCreateProcess?.()}>
                  Start process
                </Button>
              ) : undefined
            }
          />
        }
      >
        <ResourceProcessTemplateGrid
          showLabel={false}
          onSelect={(template) => props.onCreateFromTemplate?.(template)}
        />
      </Show>
    </div>
  );
}

export function resourcesFilterEmpty(kind: "inventory" | "nodes" | "processes") {
  if (kind === "nodes") {
    return <PageEmpty title="No nodes match" description="Try another search or status filter." />;
  }
  if (kind === "processes") {
    return <PageEmpty title="No processes match" description="Try another search or status filter." />;
  }
  return <PageEmpty title="No resources match" description="Try another search or status filter." />;
}
