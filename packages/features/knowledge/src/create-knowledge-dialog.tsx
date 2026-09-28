import { Show, createEffect, createSignal } from "solid-js";
import { A11yDialog, Button, Field, Input, Select } from "@polterware/frontend-design-system";
import type { KnowledgeCreateTemplate } from "./create-templates";
import { KnowledgeTemplateGrid } from "./knowledge-template-grid";

export type CreateKnowledgeKind = "doc" | "runbook" | "card" | "adr";

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
    if (props.kind === "card") return "card";
    if (props.pickLibraryKind) return pickedKind();
    return props.kind;
  };

  const copy = () => {
    const kind = activeKind();
    if (kind === "runbook") {
      return {
        title: "New runbook",
        placeholder: "Runbook title",
      };
    }
    if (kind === "card") {
      return {
        title: "New card",
        placeholder: "Card title",
      };
    }
    if (kind === "adr") {
      return {
        title: "New ADR",
        placeholder: "ADR title",
      };
    }
    return {
      title: props.pickLibraryKind ? "New library item" : "New document",
      placeholder: "Document title",
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
      size="lg"
      dividers={false}
      onClose={close}
      footer={
        <>
          <Button size="sm" variant="ghost" class="mr-auto" onClick={close}>
            Cancel
          </Button>
          <Button size="sm" variant="primary" disabled={!canSubmit()} onClick={submit}>
            Create
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
        <Field label="Title">
          <Input
            size="sm"
            value={title()}
            placeholder={copy().placeholder}
            onInput={(event) => setTitle(event.currentTarget.value)}
          />
        </Field>
        {props.onCreateFromTemplate ? (
          <KnowledgeTemplateGrid kind={activeKind()} onSelect={selectTemplate} />
        ) : null}
      </div>
    </A11yDialog>
  );
}
