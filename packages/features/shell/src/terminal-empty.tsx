import { Button, Laptop, PageEmpty, Server, TemplateCardGrid } from "@polterware/frontend-design-system";

export function TerminalNoTargetsEmpty(props: {
  onOpenLocal?: () => void;
  onAddSshHost?: () => void;
}) {
  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <TemplateCardGrid
        items={[
          {
            id: "terminal-local",
            title: "This machine",
            description: "Open a local interactive shell on this computer.",
            tone: "execution",
            icon: <Laptop size={16} />,
            onSelect: () => props.onOpenLocal?.(),
          },
          {
            id: "terminal-ssh",
            title: "Add SSH host",
            description: "Save a remote connection profile, then open it in Terminal.",
            tone: "infrastructure",
            icon: <Server size={16} />,
            onSelect: () => props.onAddSshHost?.(),
          },
        ]}
      />
    </div>
  );
}

export function TerminalNoSessionsEmpty(props: {
  onNewSession?: () => void;
  showClosedHint?: boolean;
}) {
  return (
    <PageEmpty
      title="No terminal sessions."
      description={
        props.showClosedHint
          ? "Open a new local or SSH session from a usable target. Closed sessions were removed from the switcher."
          : "Open a new local or SSH session from a usable target."
      }
      action={
        props.onNewSession ? (
          <Button size="sm" variant="primary" onClick={() => props.onNewSession?.()}>
            New session
          </Button>
        ) : undefined
      }
    />
  );
}
