import type { JSX } from "solid-js";

export function TerminalTranscript(props: {
  lines: readonly string[];
  label: string;
  class?: string;
}): JSX.Element {
  return (
    <pre
      class={`m-0 min-h-0 overflow-auto bg-[var(--color-surface-sunken)] p-3 font-mono text-[12px] leading-relaxed text-[var(--color-text-primary)] ${props.class ?? ""}`}
      role="log"
      aria-label={props.label}
      tabIndex={0}
    >
      {props.lines.join("\n")}
    </pre>
  );
}
