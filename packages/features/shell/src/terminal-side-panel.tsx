import { Show, createMemo } from "solid-js";
import { ExternalLink, X } from "@polterware/frontend-design-system";
import { Button, Drawer, StatusChip } from "@polterware/frontend-design-system";
import type { TerminalTargetItem } from "./types";
import {
  terminalTargetStatusLabel,
  terminalTargetStatusTone,
} from "./types";
import { TERMINAL_OUTPUT_FIXTURES, TERMINAL_TARGET_FIXTURES } from "./fixtures";
import { TERMINAL_SESSION_MOCKS } from "./workspace-mocks";
import type { TerminalSessionMock } from "./workspace-mocks";

export function TerminalSidePanel(props: {
  open: boolean;
  targetId: string | null;
  targets?: TerminalTargetItem[];
  sessions?: TerminalSessionMock[];
  onClose: () => void;
  onExpand: (targetId: string) => void;
}) {
  const targets = () => props.targets ?? TERMINAL_TARGET_FIXTURES;
  const sessions = () => props.sessions ?? TERMINAL_SESSION_MOCKS;
  const target = createMemo(() =>
    props.targetId ? targets().find((t) => t.id === props.targetId) ?? null : null,
  );
  const session = createMemo(() => {
    const id = props.targetId;
    if (!id) return null;
    return (
      sessions().find((s) => s.nodeId === id && s.status !== "closed") ??
      sessions().find((s) => s.nodeId === id) ??
      null
    );
  });
  const lines = createMemo(() => {
    const id = props.targetId;
    if (!id) return ["$ _"];
    const s = session();
    if (s?.lines?.length) return s.lines;
    const fixtureLines = Object.entries(TERMINAL_OUTPUT_FIXTURES)
      .find(([targetId]) => targetId === id)?.[1];
    return fixtureLines ?? [`${target()?.name ?? "target"} $ _`];
  });

  const visible = () => props.open && Boolean(props.targetId);
  const title = () => target()?.name ?? "Terminal";
  const subtitle = () => {
    const t = target();
    if (!t) return "No target selected";
    return t.shell;
  };
  const healthLabel = () => {
    const t = target();
    if (!t) return "Unknown";
    return terminalTargetStatusLabel(t.status);
  };
  const healthTone = () => {
    const t = target();
    if (!t) return "unknown" as const;
    return terminalTargetStatusTone(t.status);
  };

  return (
    <Drawer
      open={visible()}
      onClose={() => props.onClose()}
      title={title()}
      subtitle={subtitle()}
      width={480}
      footer={
        <div class="flex w-full flex-wrap items-center justify-between gap-2">
          <div class="flex min-w-0 flex-wrap items-center gap-2">
            <StatusChip tone={healthTone()} label={healthLabel()} />
            <span class="text-[11px] text-[var(--color-text-muted)]">
              {target()?.shell ?? "—"}
            </span>
          </div>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => {
              if (props.targetId) props.onExpand(props.targetId);
            }}
          >
            <span class="inline-flex items-center gap-1.5">
              <ExternalLink size={13} style={{ color: "var(--color-text-secondary)" }} />
              Expand
            </span>
          </Button>
        </div>
      }
    >
      <div class="flex min-h-0 flex-1 flex-col">
        <div class="mb-2 flex items-start justify-end gap-2">
          <div class="flex shrink-0 gap-1">
            <Button variant="ghost" size="icon" onClick={() => props.onClose()} aria-label="Close panel">
              <X size={15} style={{ color: "var(--color-text-secondary)" }} />
            </Button>
          </div>
        </div>
        <Show
          when={target()}
          fallback={
            <p class="text-[12px] text-[var(--color-text-muted)]">Select a target to open a terminal.</p>
          }
        >
          <pre
            class="m-0 min-h-0 flex-1 overflow-auto rounded-[var(--radius-item)] border border-[var(--color-border-subtle)] bg-[var(--color-app-background)] p-3 font-mono text-[12px] leading-relaxed text-[var(--color-text-secondary)]"
            role="log"
            aria-label={`Terminal output for ${target()?.name ?? "target"}`}
          >
            {lines().join("\n")}
          </pre>
        </Show>
      </div>
    </Drawer>
  );
}
