/** What Polterware reads from Polterware Dispatch's core, through `window.polterware.dispatch`. */

export type DispatchHistoryEventKind = "started" | "merged" | "failed";

export type DispatchRunEventView = {
  id: string;
  runId: string;
  projectId: string;
  kind: DispatchHistoryEventKind;
  agent: string;
  tier: string;
  text: string;
  seq: number;
  at: string;
};

export type DispatchAgentView = {
  name: string;
  bin: string;
  provider: string;
  usedPercent?: number;
  cooldownUntil: string;
  /** Whether the agent's CLI is on this machine now. */
  installed: boolean;
};

export type DispatchModelView = {
  agent: string;
  model: string;
  provider: string;
  tier: string;
};

export type DispatchTierView = {
  name: string;
  provider: string;
  agent: string;
  model: string;
  speed: number;
  strength: number;
  position: number;
};
