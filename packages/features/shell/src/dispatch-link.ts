// Polterware Dispatch is its own app. Polterware reads a few of its answers
// through the small `window.polterware.dispatch` bridge and opens the rest in
// the Dispatch app through its dispatch:// links.

type DispatchCall = (payload?: Record<string, unknown>) => Promise<unknown>;

export type PolterwareDispatch = {
  available: () => Promise<unknown>;
  listRuns: DispatchCall;
  listHistory: DispatchCall;
  startRun: DispatchCall;
  open: (url: string) => Promise<unknown>;
};

export function polterwareDispatch(): PolterwareDispatch | undefined {
  const d = (globalThis as { polterware?: { dispatch?: Partial<PolterwareDispatch> } }).polterware?.dispatch;
  if (!d?.available || !d.listRuns || !d.listHistory || !d.startRun || !d.open) return undefined;
  return d as PolterwareDispatch;
}

let availability: Promise<boolean> | undefined;

/** Whether Polterware Dispatch is installed; asked once per window. */
export function dispatchAvailable(): Promise<boolean> {
  const d = polterwareDispatch();
  if (!d) return Promise.resolve(false);
  availability ??= d.available()
    .then((raw) => Boolean((raw as { available?: unknown } | undefined)?.available))
    .catch(() => false);
  return availability;
}

export function dispatchRunUrl(runId: string): string {
  return `dispatch://run/${encodeURIComponent(runId)}`;
}

/** Opens a dispatch:// link in the Dispatch app; false when it is not installed. */
export async function openInDispatch(url: string): Promise<boolean> {
  const d = polterwareDispatch();
  if (!d || !(await dispatchAvailable())) return false;
  await d.open(url);
  return true;
}
