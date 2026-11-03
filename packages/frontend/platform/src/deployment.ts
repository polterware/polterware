export type DeploymentMode = "local" | "self-hosted" | "cloud";

export type PlatformCapabilities = {
  organizations: boolean;
  teams: boolean;
  billing: boolean;
  usage: boolean;
  managedBackups: boolean;
  cloudSync: boolean;
};

export type PlatformCapability = keyof PlatformCapabilities;

export type PlatformContext = {
  mode: DeploymentMode;
  capabilities: PlatformCapabilities;
};

export const LOCAL_CAPABILITIES: PlatformCapabilities = Object.freeze({
  organizations: false,
  teams: false,
  billing: false,
  usage: false,
  managedBackups: false,
  cloudSync: false,
});

export function capabilityEnabled(
  platform: PlatformContext | undefined,
  capability?: PlatformCapability,
): boolean {
  if (!capability) return true;
  return platform?.capabilities[capability] === true;
}
