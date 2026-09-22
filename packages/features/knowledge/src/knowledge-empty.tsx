import { Button, PageEmpty } from "@polterware/frontend-design-system";
import { Show } from "solid-js";
import type { KnowledgeCreateTemplate } from "./create-templates";
import { KnowledgeTemplateGrid } from "./knowledge-template-grid";

function EmptyTemplates(props: {
  kind: "doc" | "adr" | "runbook" | "feature";
  blankTitle: string;
  onCreate?: () => void;
  onCreateFromTemplate?: (template: KnowledgeCreateTemplate) => void;
}) {
  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <Show
        when={props.onCreateFromTemplate}
        fallback={
          <PageEmpty
            title={
              props.kind === "runbook"
                ? "No runbooks yet."
                : props.kind === "adr"
                  ? "No ADRs yet."
                  : props.kind === "feature"
                    ? "No specs yet."
                    : "No documents yet."
            }
            action={
              props.onCreate ? (
                <Button size="sm" variant="primary" onClick={() => props.onCreate?.()}>
                  {props.blankTitle}
                </Button>
              ) : undefined
            }
          />
        }
      >
        <KnowledgeTemplateGrid
          kind={props.kind}
          showLabel={false}
          onSelect={(template) => props.onCreateFromTemplate?.(template)}
        />
      </Show>
    </div>
  );
}

export function KnowledgeDocsEmpty(props: {
  onCreate?: () => void;
  onCreateFromTemplate?: (template: KnowledgeCreateTemplate) => void;
}) {
  return (
    <EmptyTemplates
      kind="doc"
      blankTitle="New document"
      onCreate={props.onCreate}
      onCreateFromTemplate={props.onCreateFromTemplate}
    />
  );
}

export function KnowledgeAdrsEmpty(props: {
  onCreate?: () => void;
  onCreateFromTemplate?: (template: KnowledgeCreateTemplate) => void;
}) {
  return (
    <EmptyTemplates
      kind="adr"
      blankTitle="New ADR"
      onCreate={props.onCreate}
      onCreateFromTemplate={props.onCreateFromTemplate}
    />
  );
}

export function KnowledgeRunbooksEmpty(props: {
  onCreate?: () => void;
  onCreateFromTemplate?: (template: KnowledgeCreateTemplate) => void;
}) {
  return (
    <EmptyTemplates
      kind="runbook"
      blankTitle="New runbook"
      onCreate={props.onCreate}
      onCreateFromTemplate={props.onCreateFromTemplate}
    />
  );
}

export function KnowledgeSpecsEmpty(props: {
  onCreate?: () => void;
  onCreateFromTemplate?: (template: KnowledgeCreateTemplate) => void;
}) {
  return (
    <EmptyTemplates
      kind="feature"
      blankTitle="New feature"
      onCreate={props.onCreate}
      onCreateFromTemplate={props.onCreateFromTemplate}
    />
  );
}

export function KnowledgeConstitutionEmpty(props: { onCreate?: () => void }) {
  return (
    <div class="border-b border-[var(--color-border)] px-[var(--space-4)] py-[var(--space-3)]">
      <div class="flex items-center justify-between gap-3">
        <div>
          <p class="m-0 text-[13px] font-medium text-[var(--color-text)]">No constitution yet</p>
          <p class="m-0 mt-1 text-[12px] text-[var(--color-text-muted)]">
            Add project principles that anchor Specs for this local project.
          </p>
        </div>
        <Show when={props.onCreate}>
          <Button size="sm" variant="secondary" onClick={() => props.onCreate?.()}>
            Create constitution
          </Button>
        </Show>
      </div>
    </div>
  );
}

export function knowledgeFilterEmpty(kind: "docs" | "adrs" | "runbooks" | "specs") {
  if (kind === "runbooks") {
    return <PageEmpty title="No runbooks match" description="Try another search or severity filter." />;
  }
  if (kind === "specs") {
    return <PageEmpty title="No features match" description="Try another search or status filter." />;
  }
  if (kind === "adrs") {
    return <PageEmpty title="No ADRs match" description="Try another search or status filter." />;
  }
  return <PageEmpty title="No documents match" description="Try another search or status filter." />;
}
