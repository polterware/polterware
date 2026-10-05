import { ErrorBoundary } from "solid-js";
import { PageError } from "@polterware/frontend-design-system";
import { SparkyShell, createShellBridge } from "@polterware/feature-sparky-shell";
import { hideBootSplash } from "./boot-splash";
import { features } from "./features";

export function App() {
  return <ErrorBoundary fallback={(error, reset) => { hideBootSplash(); return <PageError title="Sparky failed to start" message={error.message} onRetry={reset} />; }}>
    <SparkyShell features={features} bridge={window.sparky ? createShellBridge(window.sparky) : undefined} onReady={() => hideBootSplash()} />
  </ErrorBoundary>;
}
