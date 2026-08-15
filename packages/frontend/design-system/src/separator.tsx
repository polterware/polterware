export function Separator(props: { class?: string; orientation?: "horizontal" | "vertical" }) {
  if (props.orientation === "vertical") {
    return (
      <div
        role="separator"
        aria-orientation="vertical"
        class={`w-px self-stretch bg-[var(--color-border-subtle)] ${props.class ?? ""}`}
      />
    );
  }
  return <hr class={`border-0 border-t border-[var(--color-border-subtle)] my-2 ${props.class ?? ""}`} />;
}
