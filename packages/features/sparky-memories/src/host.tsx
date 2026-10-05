import { Show } from "solid-js";
import type { DetailSectionDef } from "@polterware/feature-sparky-shell/feature";
import { MemoryDetail } from "./memory-detail";
import { trySparkyShell } from "./shell-access";
import type { MemoriesStore } from "./store";

export function MemoryDetailHost(props: { memoryId?: string; sections?: DetailSectionDef[]; store?: MemoriesStore }) {
  const shell = trySparkyShell();
  const id = () => props.memoryId || shell?.selectedMemoryId() || shell?.params().id || "";
  return <Show when={id()}>{value => <MemoryDetail memoryId={value()} sections={props.sections ?? shell?.detailSections() ?? []} store={props.store} />}</Show>;
}
