import { Show, createEffect, createSignal } from "solid-js";
import {
  A11yDialog,
  Button,
  SwitchRow,
  Field,
  Input,
  Select,
  Textarea,
} from "@polterware/frontend-design-system";
import {
  PPM_EVIDENCE_KIND_OPTIONS,
  type PpmEvidenceDraft,
} from "./ppm-facts-types";

export function PpmEvidenceFormDialog(props: {
  open: boolean;
  canLinkToFact: boolean;
  onClose: () => void;
  onSubmit: (draft: PpmEvidenceDraft) => void;
}) {
  const [kind, setKind] = createSignal("file");
  const [summary, setSummary] = createSignal("");
  const [path, setPath] = createSignal("");
  const [linkToSelectedFact, setLinkToSelectedFact] = createSignal(true);

  createEffect(() => {
    if (!props.open) return;
    setKind("file");
    setSummary("");
    setPath("");
    setLinkToSelectedFact(props.canLinkToFact);
  });

  const canSubmit = () => summary().trim().length > 0;

  function submit() {
    const next = summary().trim();
    if (!next) return;
    props.onSubmit({
      kind: kind(),
      summary: next,
      path: path().trim(),
      linkToSelectedFact: props.canLinkToFact && linkToSelectedFact(),
    });
  }

  return (
    <A11yDialog
      open={props.open}
      title="Add evidence"
      description="Evidence supports PPM facts. Optionally link it to the selected fact after create."
      size="md"
      onClose={() => props.onClose()}
      footer={
        <>
          <Button size="sm" variant="ghost" onClick={() => props.onClose()}>
            Cancel
          </Button>
          <Button size="sm" variant="primary" disabled={!canSubmit()} onClick={submit}>
            Create
          </Button>
        </>
      }
    >
      <div class="grid gap-3">
        <Field label="Kind">
          <Select
            options={[...PPM_EVIDENCE_KIND_OPTIONS]}
            value={kind()}
            onChange={(value) => setKind(value)}
          />
        </Field>
        <Field label="Summary">
          <Textarea
            rows={3}
            value={summary()}
            placeholder="package.json declares left-pad"
            onInput={(event) => setSummary(event.currentTarget.value)}
          />
        </Field>
        <Field label="Path">
          <Input
            value={path()}
            placeholder="optional project-relative path"
            onInput={(event) => setPath(event.currentTarget.value)}
          />
        </Field>
        <Show when={props.canLinkToFact}>
          <SwitchRow
            checked={linkToSelectedFact()}
            label="Link to selected fact"
            onCheckedChange={(checked) => setLinkToSelectedFact(checked)}
          />
        </Show>
      </div>
    </A11yDialog>
  );
}
