import { createSignal } from "solid-js";
import { A11yDialog, Button, Field, Input } from "@polterware/frontend-design-system";
import type { KnowledgeCreateTemplate } from "./create-templates";
import { KnowledgeTemplateGrid } from "./knowledge-template-grid";

export function CreateSourceDialog(props: {
  open: boolean;
  onClose: () => void;
  onSubmit: (input: { title: string; path: string }) => void;
  onCreateFromTemplate?: (template: KnowledgeCreateTemplate) => void;
}) {
  const [title, setTitle] = createSignal("");
  const [path, setPath] = createSignal("");
  const canSubmit = () => title().trim().length > 0 && path().trim().length > 0;

  function close() {
    setTitle("");
    setPath("");
    props.onClose();
  }

  function submit() {
    const nextTitle = title().trim();
    const nextPath = path().trim();
    if (!nextTitle || !nextPath) return;
    props.onSubmit({ title: nextTitle, path: nextPath });
    setTitle("");
    setPath("");
  }

  return (
    <A11yDialog
      open={props.open}
      title="Add source"
      size="lg"
      onClose={close}
      footer={
        <>
          <Button size="sm" variant="ghost" onClick={close}>
            Cancel
          </Button>
          <Button size="sm" variant="primary" disabled={!canSubmit()} onClick={submit}>
            Create
          </Button>
        </>
      }
    >
      <div class="grid gap-4">
        <Field label="Title">
          <Input
            size="sm"
            value={title()}
            placeholder="README"
            onInput={(event) => setTitle(event.currentTarget.value)}
          />
        </Field>
        <Field label="Path" hint="Relative to the project root, e.g. README.md or infra/main.tf">
          <Input
            size="sm"
            value={path()}
            placeholder="README.md"
            onInput={(event) => setPath(event.currentTarget.value)}
          />
        </Field>
        {props.onCreateFromTemplate ? (
          <KnowledgeTemplateGrid
            kind="source"
            onSelect={(template) => {
              props.onCreateFromTemplate?.(template);
              setTitle("");
              setPath("");
            }}
          />
        ) : null}
      </div>
    </A11yDialog>
  );
}
