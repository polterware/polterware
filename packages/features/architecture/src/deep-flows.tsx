import { createSignal, For } from "solid-js";
import { Button, CheckboxRow, Input, PageContent, PageHeader, pushToast } from "@polterware/frontend-design-system";

export function ViewEditor(props: { name?: string; onClose?: () => void; onSave?: () => void }) {
  const [name, setName] = createSignal(props.name ?? "Payment flow");
  const components = [
    "API Gateway",
    "Payment Service",
    "Worker",
    "PostgreSQL",
    "Redis",
    "Event Queue",
    "Auth Service",
  ];
  const [included, setIncluded] = createSignal(new Set(["API Gateway", "Payment Service", "Worker", "PostgreSQL", "Redis", "Event Queue"]));

  return (
    <div class="flex h-full min-h-0 flex-col">
      <PageHeader
        title={`Edit view: ${name()}`}
        actions={
          <>
            <Button size="sm" variant="ghost" onClick={() => props.onClose?.()}>
              Cancel
            </Button>
            <Button
              size="sm"
              variant="primary"
              onClick={() => {
                pushToast("View saved", "success");
                props.onSave?.();
                props.onClose?.();
              }}
            >
              Save
            </Button>
          </>
        }
      />
      <PageContent class="space-y-3 p-4 text-[12.5px]">
        <label class="block text-[12px] text-[var(--color-text-secondary)]">
          Name
          <Input size="sm" class="mt-1 w-full max-w-md" value={name()} onInput={(e) => setName(e.currentTarget.value)} />
        </label>
        <div>
          <p class="m-0 mb-2 text-[12px] font-medium">Included components</p>
          <For each={components}>
            {(c) => (
              <CheckboxRow
                checked={included().has(c)}
                label={c}
                onCheckedChange={(checked) => {
                  setIncluded((set) => {
                    const next = new Set(set);
                    if (checked) next.add(c);
                    else next.delete(c);
                    return next;
                  });
                }}
              />
            )}
          </For>
        </div>
      </PageContent>
    </div>
  );
}
