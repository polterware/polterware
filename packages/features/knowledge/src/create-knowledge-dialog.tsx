import { createSignal } from "solid-js";
import { A11yDialog, Button, Field, Input } from "@polterware/frontend-design-system";
import type { KnowledgeCreateTemplate } from "./create-templates";
import { KnowledgeTemplateGrid } from "./knowledge-template-grid";

export type CreateKnowledgeKind = "doc" | "runbook" | "feature" | "adr";

export function CreateKnowledgeDialog(props: {
  open: boolean;
  kind: CreateKnowledgeKind;
  onClose: () => void;
  onSubmit: (title: string) => void;
  onCreateFromTemplate?: (template: KnowledgeCreateTemplate) => void;
}) {
  const [title, setTitle] = createSignal("");
  const copy = () => {
    if (props.kind === "runbook") {
      return {
        title: "New runbook",
        description: "Pick a template or create a blank runbook stored in the local SQLite database.",
        placeholder: "Runbook title",
        blank: "Create blank",
      };
    }
    if (props.kind === "feature") {
      return {
        title: "New feature",
        description: "Pick a template or create a blank Spec, Plan, and Tasks draft linked by one feature key.",
        placeholder: "Feature title",
        blank: "Create blank",
      };
    }
    if (props.kind === "adr") {
      return {
        title: "New ADR",
        description: "Pick a template or create a blank architecture decision record.",
        placeholder: "ADR title",
        blank: "Create blank",
      };
    }
    return {
      title: "New document",
      description: "Pick a template or create a blank document stored in the local SQLite database.",
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
    props.onSubmit(next);
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
        {props.onCreateFromTemplate ? (
          <KnowledgeTemplateGrid kind={props.kind} onSelect={selectTemplate} />
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
