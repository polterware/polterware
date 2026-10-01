import { Show, type JSX } from "solid-js";
import {
  Button,
  Book03Icon,
  DetailPanel,
  Select,
  StatusChip,
} from "@polterware/frontend-design-system";
import type { WorkspaceAdr } from "@polterware/feature-shell";
import { KnowledgeDocumentShell } from "./knowledge-document-shell";
import { decisionStateTone } from "./library-row";

const STATE_LABEL: Record<WorkspaceAdr["state"], string> = {
  proposed: "Proposed",
  active: "Active",
  challenged: "Challenged",
  superseded: "Superseded",
};

/** A lifecycle move the detail panel asks the host to write. */
export type DecisionAction =
  | { kind: "activate" }
  | { kind: "challenge" }
  | { kind: "reactivate" }
  | { kind: "supersede"; byId: string };

export function DecisionDetail(props: {
  adr: WorkspaceAdr;
  /** Every ADR in the project; superseding picks among the ones still in force. */
  allAdrs: WorkspaceAdr[];
  bodyMarkdown: string;
  onBodyChange: (md: string) => void;
  onAction?: (action: DecisionAction) => void;
  links?: JSX.Element;
}) {
  const successors = () =>
    props.allAdrs.filter((other) => other.id !== props.adr.id && other.state !== "superseded");
  const canSupersede = () =>
    Boolean(props.onAction) && (props.adr.state === "active" || props.adr.state === "challenged") && successors().length > 0;
  return (
    <DetailPanel.Content>
      <DetailPanel.Header />
      <KnowledgeDocumentShell
        title={props.adr.title}
        icon={<Book03Icon size={16} />}
        bodyMarkdown={props.bodyMarkdown}
        onBodyChange={props.onBodyChange}
        properties={
          <DetailPanel.Properties variant="inline">
            <DetailPanel.Property
              label="Status"
              value={
                <div class="flex flex-wrap items-center gap-2">
                  <StatusChip tone={decisionStateTone(props.adr.state)}>{STATE_LABEL[props.adr.state]}</StatusChip>
                  <Show when={props.onAction}>
                    <Show when={props.adr.state === "proposed"}>
                      <Button variant="secondary" size="sm" onClick={() => props.onAction?.({ kind: "activate" })}>Activate</Button>
                    </Show>
                    <Show when={props.adr.state === "active"}>
                      <Button variant="ghost" size="sm" onClick={() => props.onAction?.({ kind: "challenge" })}>Open review</Button>
                    </Show>
                    <Show when={props.adr.state === "challenged"}>
                      <Button variant="ghost" size="sm" onClick={() => props.onAction?.({ kind: "reactivate" })}>Keep active</Button>
                    </Show>
                  </Show>
                </div>
              }
            />
            <DetailPanel.Property
              label="Scope"
              value={props.adr.scope === "principle" ? "Principle (whole project)" : "Decision"}
            />
            <DetailPanel.Property label="Review when" value={props.adr.reviewTrigger || null} />
            <DetailPanel.Property label="Reason" value={props.adr.statusReason || null} />
            <DetailPanel.Property label="Superseded by" value={props.adr.supersededBy || null} />
            <DetailPanel.Property
              label="Supersedes"
              value={props.adr.supersedes.length > 0 ? props.adr.supersedes.join(", ") : null}
            />
            <DetailPanel.Property
              label="Governs"
              value={props.adr.governs.length > 0 ? props.adr.governs.join(", ") : null}
              mono
            />
            <Show when={canSupersede()}>
              <DetailPanel.Property
                label="Supersede with"
                value={
                  <Select
                    aria-label="Supersede with"
                    size="sm"
                    class="w-56"
                    placeholder="Choose a decision…"
                    options={successors().map((other) => ({ value: other.id, label: other.title }))}
                    value=""
                    onChange={(next) => {
                      if (next) props.onAction?.({ kind: "supersede", byId: next });
                    }}
                  />
                }
              />
            </Show>
          </DetailPanel.Properties>
        }
        links={props.links}
      />
    </DetailPanel.Content>
  );
}
