import { Show, createEffect, createSignal, onCleanup, onMount } from "solid-js";
import {
  A11yDialog,
  Button,
  Field,
  Input,
  Textarea,
} from "@polterware/frontend-design-system";
import {
  resolveChangeEnvelope,
  subscribeChangeEnvelopeRequests,
  type ChangeEnvelopeRequest,
} from "./change-envelope-request";

/** Host dialog for justified Knowledge / PPM mutations (B9). */
export function ChangeEnvelopeDialogHost() {
  const [open, setOpen] = createSignal(false);
  const [request, setRequest] = createSignal<ChangeEnvelopeRequest | null>(null);
  const [summary, setSummary] = createSignal("");
  const [rationale, setRationale] = createSignal("");

  onMount(() => {
    const unsubscribe = subscribeChangeEnvelopeRequests((pending) => {
      if (!pending) {
        setOpen(false);
        setRequest(null);
        return;
      }
      setRequest(pending.request);
      setSummary(pending.request.defaultSummary ?? "");
      setRationale("");
      setOpen(true);
    });
    onCleanup(unsubscribe);
  });

  createEffect(() => {
    if (open()) return;
    setSummary("");
    setRationale("");
  });

  const canSubmit = () =>
    summary().trim().length > 0 && rationale().trim().length > 0;

  function cancel() {
    resolveChangeEnvelope(null);
  }

  function submit() {
    if (!canSubmit()) return;
    resolveChangeEnvelope({
      summary: summary().trim(),
      rationale: rationale().trim(),
    });
  }

  return (
    <A11yDialog
      open={open()}
      size="sm"
      title={request()?.title ?? "Justify change"}
      description={
        request()?.description
        ?? "Every project-model change needs a short summary and rationale."
      }
      dismissible
      onClose={cancel}
      footer={
        <>
          <Button size="sm" variant="ghost" class="mr-auto" onClick={cancel}>
            Cancel
          </Button>
          <Button size="sm" variant="brand" disabled={!canSubmit()} onClick={submit}>
            Save change
          </Button>
        </>
      }
    >
      <Show when={request()}>
        <div class="flex flex-col gap-3">
          <Field label="Summary">
            <Input
              value={summary()}
              placeholder="What changed?"
              onInput={(event) => setSummary(event.currentTarget.value)}
            />
          </Field>
          <Field label="Rationale">
            <Textarea
              value={rationale()}
              placeholder="Why is this change justified?"
              rows={4}
              onInput={(event) => setRationale(event.currentTarget.value)}
            />
          </Field>
        </div>
      </Show>
    </A11yDialog>
  );
}
