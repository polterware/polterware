import { Show, createSignal } from "solid-js";
import { DetailPanel, DesignSystemProvider } from "@polterware/frontend-design-system";
import type { DetailSectionDef } from "@polterware/feature-sparky-shell/feature";
import { MemoryCollection } from "./memory-list";
import { MemoryDetail } from "./memory-detail";
import type { MemoriesStore } from "./store";

/** Isolated host for page behavior tests and W3's 10k Playwright performance fixture. */
export function MemoryFixtureWorkspace(props: { store: MemoriesStore; sections?: DetailSectionDef[]; virtualized?: boolean; mode?: "today" | "inbox" | "mind"; mindId?: string }) {
  const [selected, setSelected] = createSignal<string>();
  const [open, setOpen] = createSignal(false);
  const [expanded, setExpanded] = createSignal(false);
  return <DesignSystemProvider storagePrefix="sparky-fixture">
    <DetailPanel.Root open={open()} expanded={expanded()} activeEntityId={selected() ?? null} onOpenChange={setOpen} onExpandedChange={setExpanded}>
      <DetailPanel.Collection><MemoryCollection store={props.store} mode={props.mode} mindId={props.mindId} virtualized={props.virtualized} onOpen={id => { setSelected(id); setOpen(true); }} /></DetailPanel.Collection>
      <DetailPanel.Content><DetailPanel.Header /><Show when={selected()}>{id => <MemoryDetail memoryId={id()} sections={props.sections} store={props.store} />}</Show></DetailPanel.Content>
    </DetailPanel.Root>
  </DesignSystemProvider>;
}
