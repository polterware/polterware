import type { TerminalTargetItem } from "./types";

/** Unusable targets — picker must block open with honest feedback. */
export const UNUSABLE_TERMINAL_TEST_TARGETS: TerminalTargetItem[] = [
  { id: "node-offline-01", name: "offline-edge-01", shell: "bash", status: "offline", environmentId: "env-polter-prod" },
  { id: "node-revoked-01", name: "revoked-node-01", shell: "bash", status: "revoked", environmentId: "env-polter-prod" },
];
