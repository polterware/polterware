import { useSparkyShell, type SparkyShellContext } from "@polterware/feature-sparky-shell/feature";

/** Registry and tests call this during render. Missing host context is a fixture, not a crash. */
export function trySparkyShell(): SparkyShellContext | undefined {
  try { return useSparkyShell(); } catch { return undefined; }
}
