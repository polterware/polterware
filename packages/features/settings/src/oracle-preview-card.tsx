import { Show, createSignal } from "solid-js";
import { Button, Field, Input, Select } from "@polterware/frontend-design-system";

const ZOOMS = [
  { value: "task", label: "Task" },
  { value: "component", label: "Component" },
  { value: "domain", label: "Domain" },
  { value: "system", label: "System" },
];

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

/** The deterministic Oracle text an agent would receive for one target. */
export function OraclePreviewCard(props: {
  projectId: string;
  query?: (payload: { projectId: string; query: string }) => Promise<unknown>;
}) {
  const [target, setTarget] = createSignal("");
  const [zoom, setZoom] = createSignal("task");
  const [budget, setBudget] = createSignal("1200");
  const [text, setText] = createSignal("");
  const [error, setError] = createSignal("");
  const [busy, setBusy] = createSignal(false);

  async function run() {
    const query = props.query;
    const name = target().trim();
    if (!query) {
      setError("Oracle is unavailable in this session.");
      setText("");
      return;
    }
    if (!name) {
      setError("Enter a node path, card slug, or exact title.");
      setText("");
      return;
    }
    const limit = Number.parseInt(budget(), 10);
    if (!Number.isFinite(limit) || limit < 1) {
      setError("Budget must be a positive number of tokens.");
      setText("");
      return;
    }
    setBusy(true);
    setError("");
    try {
      const response = await query({
        projectId: props.projectId,
        query: `oracle ${name} zoom:${zoom()} budget:${limit}`,
      });
      setText(typeof asRecord(response).text === "string" ? String(asRecord(response).text) : "");
    } catch (cause) {
      setText("");
      setError(cause instanceof Error ? cause.message : "Could not read the Oracle.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div class="flex flex-col gap-4">
      <div class="grid gap-3 sm:grid-cols-[minmax(0,1fr)_8rem_6rem_auto] sm:items-end">
        <Field label="Target">
          <Input
            value={target()}
            placeholder="core/import"
            onInput={(event) => setTarget(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") void run();
            }}
          />
        </Field>
        <Field label="Zoom">
          <Select options={ZOOMS} value={zoom()} onChange={setZoom} />
        </Field>
        <Field label="Budget">
          <Input
            value={budget()}
            inputMode="numeric"
            onInput={(event) => setBudget(event.currentTarget.value)}
          />
        </Field>
        <Button size="sm" variant="primary" disabled={busy()} onClick={() => void run()}>
          {busy() ? "Reading…" : "Show"}
        </Button>
      </div>
      <Show when={error()}>
        <p role="alert" class="m-0 text-[12px] text-[var(--color-danger)]">{error()}</p>
      </Show>
      <Show when={text()}>
        <pre class="m-0 max-h-[28rem] overflow-auto whitespace-pre-wrap rounded-[var(--radius-control)] bg-[var(--color-surface-subtle)] p-3 font-mono text-[11.5px] leading-relaxed text-[var(--color-text-primary)] pw-selectable">
          {text()}
        </pre>
      </Show>
    </div>
  );
}
