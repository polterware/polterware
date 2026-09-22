import { Show, createEffect, createSignal } from "solid-js";
import { A11yDialog, Button, Field, Input, Select } from "@polterware/frontend-design-system";
import type { KnowledgeCreateTemplate } from "./create-templates";
import { KnowledgeTemplateGrid } from "./knowledge-template-grid";

export type CreateKnowledgeKind = "doc" | "runbook" | "feature" | "adr";

const LIBRARY_KIND_OPTIONS = [
  { value: "doc", label: "Document" },
  { value: "adr", label: "ADR" },
  { value: "runbook", label: "Runbook" },
] as const;

function isLibraryKind(kind: CreateKnowledgeKind): kind is "doc" | "adr" | "runbook" {
  return kind === "doc" || kind === "adr" || kind === "runbook";
}

export function CreateKnowledgeDialog(props: {
  open: boolean;
  kind: CreateKnowledgeKind;
  /** When true, show a category select for doc / ADR / runbook. */
  pickLibraryKind?: boolean;
  onClose: () => void;
  onSubmit: (title: string, kind: CreateKnowledgeKind) => void;
  onCreateFromTemplate?: (template: KnowledgeCreateTemplate) => void;
}) {
  const [title, setTitle] = createSignal("");
  const [pickedKind, setPickedKind] = createSignal<"doc" | "adr" | "runbook">("doc");

  createEffect(() => {
    if (!props.open) return;
    if (isLibraryKind(props.kind)) setPickedKind(props.kind);
    else setPickedKind("doc");
  });

  const activeKind = (): CreateKnowledgeKind => {
    if (props.kind === "feature") return "feature";
    if (props.pickLibraryKind) return pickedKind();
    return props.kind;
  };

  const copy = () => {
    const kind = activeKind();
    if (kind === "runbook") {
      return {
        title: "New runbook",
        description: "Pick a template or create a blank runbook stored in the local SQLite database.",
        placeholder: "Runbook title",
        blank: "Create blank",
      };
    }
    if (kind === "feature") {
      return {
        title: "New feature",
        description: "Pick a template or create a blank Spec, Plan, and Tasks draft linked by one feature key.",
        placeholder: "Feature title",
        blank: "Create blank",
      };
    }
    if (kind === "adr") {
      return {
        title: "New ADR",
        description: "Pick a template or create a blank architecture decision record.",
        placeholder: "ADR title",
        blank: "Create blank",
      };
    }
    return {
      title: props.pickLibraryKind ? "New library item" : "New document",
      description: props.pickLibraryKind
        ? "Choose a category, then pick a template or create a blank item stored in the local SQLite database."
        : "Pick a template or create a blank document stored in the local SQLite database.",
      placeholder: "Document title",
      blank: "Create blank",
    };
  };
  const canSubmit = () => title().trim().length > 0;

  function close() {
    setTitle("");
    props.onClose();
  }

  function submit() {
    const next = title().trim();
    if (!next) return;
    props.onSubmit(next, activeKind());
    setTitle("");
  }

  function selectTemplate(template: KnowledgeCreateTemplate) {
    props.onCreateFromTemplate?.(template);
    setTitle("");
  }

  return (
    <A11yDialog
      open={props.open}
      title={copy().title}
      description={copy().description}
      size="xl"
      onClose={close}
      footer={
        <>
          <Button size="sm" variant="ghost" onClick={close}>
            Cancel
          </Button>
          <Button size="sm" variant="primary" disabled={!canSubmit()} onClick={submit}>
            {copy().blank}
          </Button>
        </>
      }
    >
      <div class="grid gap-4">
        <Show when={props.pickLibraryKind}>
          <Field label="Category">
            <Select
              size="sm"
              value={pickedKind()}
              options={[...LIBRARY_KIND_OPTIONS]}
              onChange={(value) => {
                if (value === "doc" || value === "adr" || value === "runbook") {
                  setPickedKind(value);
                }
              }}
            />
          </Field>
        </Show>
        {props.onCreateFromTemplate ? (
          <KnowledgeTemplateGrid kind={activeKind()} onSelect={selectTemplate} />
        ) : null}
        <Field label="Title">
          <Input
            size="sm"
            value={title()}
            placeholder={copy().placeholder}
            onInput={(event) => setTitle(event.currentTarget.value)}
          />
        </Field>
      </div>
    </A11yDialog>
  );
}
