/**
 * Polterware's link to Polterware Dispatch: a run's Route pill (`./run-route`), the agent
 * avatars, and the history Activity lists. Running and following runs belongs to the
 * Dispatch app; Polterware starts them and opens them there.
 */
import {
  Button,
  DetailPanel,
  ExternalLink,
  FileText,
  Route,
  Settings,
  StatusChip,
} from "@polterware/frontend-design-system";
import { dispatchStatusTone } from "./dispatch-bridge";
import type { DispatchRunEventView } from "./dispatch-types";

export * from "./dispatch-types";
export * from "./dispatch-bridge";
export { DispatchAgentAvatar } from "./agent-avatar";
export { createRunRoute, createRunRouteSources, loadAutoAvailable, type RunRouteCard, type RunRouteRepository, type RunRouteSelection } from "./run-route";

function formatWhen(value: string): string {
  if (!value) return "—";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

/** Short summary panel for a dispatch event on Activity (no live output). */
export function DispatchEventSummaryDetail(props: {
  event: DispatchRunEventView;
  /** Opens the run in Polterware Dispatch. */
  onOpenRun?: () => void;
}) {
  const error = () => (props.event.kind === "failed" ? props.event.text : "");
  return (
    <DetailPanel.Content>
      <DetailPanel.Header />
      <DetailPanel.Body>
        <DetailPanel.Main>
          <DetailPanel.Title
            icon={<Route size={16} />}
            title={props.event.text || `Dispatch ${props.event.kind}`}
          />
          <DetailPanel.Section title="Run summary" icon={<FileText size={14} />} aside={props.onOpenRun ? (
            <Button size="icon" variant="ghost" aria-label="Open in Dispatch" onClick={() => props.onOpenRun?.()}>
              <ExternalLink size={14} />
            </Button>
          ) : undefined}>
            <DetailPanel.Properties>
              <DetailPanel.Property
                label="Status"
                value={<StatusChip tone={dispatchStatusTone(props.event.kind)} label={props.event.kind} />}
              />
              <DetailPanel.Property label="Tier" value={props.event.tier || "—"} />
              <DetailPanel.Property label="Agent" value={props.event.agent || "—"} />
              <DetailPanel.Property label="Error" value={error() || "—"} />
            </DetailPanel.Properties>
          </DetailPanel.Section>
        </DetailPanel.Main>
        <DetailPanel.Rail>
          <DetailPanel.Section title="Event" icon={<Settings size={14} />}>
            <DetailPanel.Properties>
              <DetailPanel.Property
                label="Kind"
                value={<StatusChip tone={dispatchStatusTone(props.event.kind)} label={props.event.kind} />}
              />
              <DetailPanel.Property label="Run id" value={props.event.runId || "—"} mono />
              <DetailPanel.Property label="When" value={formatWhen(props.event.at)} />
            </DetailPanel.Properties>
          </DetailPanel.Section>
        </DetailPanel.Rail>
      </DetailPanel.Body>
    </DetailPanel.Content>
  );
}
