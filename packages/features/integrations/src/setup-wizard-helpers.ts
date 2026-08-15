import type { CliConnectionState, WorkspaceCatalogProvider } from "@polterware/feature-shell";

export function cliStateLabel(state: CliConnectionState): string {
  switch (state) {
    case "cli-unavailable": return "CLI unavailable";
    case "ready": return "Ready";
    case "authenticating": return "Authenticating";
    case "connected": return "Connected";
    case "failed": return "Failed";
  }
}

export function cliStateTone(
  state: CliConnectionState,
): "muted" | "unknown" | "syncing" | "healthy" | "danger" {
  switch (state) {
    case "cli-unavailable": return "unknown";
    case "ready": return "muted";
    case "authenticating": return "syncing";
    case "connected": return "healthy";
    case "failed": return "danger";
  }
}

export function providerByName(
  providers: readonly WorkspaceCatalogProvider[],
  value?: string,
): WorkspaceCatalogProvider | undefined {
  const needle = value?.trim().toLowerCase();
  if (!needle) return providers[0];
  return providers.find((provider) =>
    provider.id === needle || provider.name.toLowerCase() === needle,
  );
}

export function matchConnectedIntegrationId(
  providerName: string,
  integrations: readonly { id: string; name: string; provider: string }[],
): string | null {
  const needle = providerName.trim().toLowerCase();
  const match = integrations.find((item) =>
    item.name.toLowerCase() === needle || item.provider.toLowerCase() === needle,
  );
  return match?.id ?? null;
}
