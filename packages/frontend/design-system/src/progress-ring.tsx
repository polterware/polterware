import { Progress, type ProgressRootProps } from "@ark-ui/solid/progress";
import { splitProps } from "solid-js";
import "./progress-ring.css";
export type ProgressRingProps = Omit<ProgressRootProps, "children"> & { label: string; size?: number; thickness?: number };
export function ProgressRing(props: ProgressRingProps) {
  const [local, rest] = splitProps(props, ["label", "size", "thickness", "class"]);
  return <Progress.Root {...rest} class={`pw-progress-ring ${local.class ?? ""}`}>
    <Progress.Label class="pw-progress-ring__label">{local.label}</Progress.Label>
    <Progress.Circle style={{ "--size": `${local.size ?? 64}px`, "--thickness": `${local.thickness ?? 4}px` }}>
      <Progress.CircleTrack class="pw-progress-ring__track" /><Progress.CircleRange class="pw-progress-ring__range" />
    </Progress.Circle>
    <Progress.ValueText class="pw-progress-ring__value" />
  </Progress.Root>;
}
