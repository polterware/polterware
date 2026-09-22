import { Button, PageEmpty } from "@polterware/frontend-design-system";
import { Show } from "solid-js";
import type { KnowledgeCreateTemplate } from "./create-templates";
import { KnowledgeTemplateGrid } from "./knowledge-template-grid";

export function SourcesEmpty(props: {
  onCreate?: () => void;
  onImport?: () => void;
  importBusy?: boolean;
  onCreateFromTemplate?: (template: KnowledgeCreateTemplate) => void;
}) {
  const importAction = () =>
    props.onImport ? (
      <Button
        size="sm"
        variant="primary"
        disabled={props.importBusy}
        onClick={() => props.onImport?.()}
      >
        Import from project root
      </Button>
    ) : undefined;

  const createAction = () =>
    props.onCreate ? (
      <Button size="sm" variant="secondary" onClick={() => props.onCreate?.()}>
        Add source
      </Button>
    ) : undefined;

  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <Show
        when={props.onCreateFromTemplate}
        fallback={
          <PageEmpty
            title="No sources yet."
            description="Import MVP manifests and contracts from the project root, or register a path manually."
            action={
              props.onImport || props.onCreate ? (
                <div class="flex flex-wrap items-center justify-center gap-2">
                  {importAction()}
                  {createAction()}
                </div>
              ) : undefined
            }
          />
        }
      >
        <div class="flex w-full max-w-3xl flex-col items-center gap-6">
          <Show when={props.onImport}>
            <PageEmpty
              title="Import known project files"
              description="Scans the project root for go.mod, package.json, agent docs, Docker, OpenAPI, and Terraform."
              action={importAction()}
            />
          </Show>
          <KnowledgeTemplateGrid
            kind="source"
            showLabel={false}
            onSelect={(template) => props.onCreateFromTemplate?.(template)}
          />
        </div>
      </Show>
    </div>
  );
}

export function sourcesFilterEmpty() {
  return (
    <PageEmpty
      title="No sources match"
      description="Try another search or clear active filters."
    />
  );
}
