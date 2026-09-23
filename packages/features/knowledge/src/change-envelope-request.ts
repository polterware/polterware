export type ChangeEnvelope = {
  summary: string;
  rationale: string;
  agentRunId?: string;
};

export type ChangeEnvelopeRequest = {
  title: string;
  description?: string;
  defaultSummary?: string;
};

type Pending = {
  request: ChangeEnvelopeRequest;
  resolve: (value: ChangeEnvelope | null) => void;
};

let pending: Pending | null = null;
let notify: ((pending: Pending | null) => void) | null = null;

/** Subscribe the host dialog to pending justification requests. */
export function subscribeChangeEnvelopeRequests(
  listener: (pending: Pending | null) => void,
): () => void {
  notify = listener;
  listener(pending);
  return () => {
    if (notify === listener) notify = null;
  };
}

/**
 * Opens the shared change-envelope dialog and resolves with the justified
 * envelope, or null when the user cancels.
 */
export function requestChangeEnvelope(
  request: ChangeEnvelopeRequest,
): Promise<ChangeEnvelope | null> {
  if (pending) {
    pending.resolve(null);
    pending = null;
  }
  return new Promise((resolve) => {
    pending = { request, resolve };
    notify?.(pending);
  });
}

export function resolveChangeEnvelope(value: ChangeEnvelope | null): void {
  if (!pending) return;
  const current = pending;
  pending = null;
  notify?.(null);
  current.resolve(value);
}
