export type ShellChromeMode = "vibrancy" | "solid";

export type AppearancePreferences = {
  version: number;
  reduceTransparency: boolean;
};

type AppearancePreferencesBridge = {
  getAppearancePreferences: () => Promise<AppearancePreferences>;
  setAppearancePreferences: (preferences: {
    reduceTransparency: boolean;
  }) => Promise<AppearancePreferences>;
};

function appearancePreferencesBridge(): AppearancePreferencesBridge | undefined {
  const root = globalThis as {
    polterware?: { app?: Partial<AppearancePreferencesBridge> };
  };
  const app = root.polterware?.app;
  if (!app?.getAppearancePreferences || !app?.setAppearancePreferences) return undefined;
  return app as AppearancePreferencesBridge;
}

export function applyShellChromeMode(reduceTransparency: boolean): ShellChromeMode {
  const mode: ShellChromeMode = reduceTransparency ? "solid" : "vibrancy";
  if (typeof document !== "undefined") {
    document.documentElement.dataset.shellChrome = mode;
  }
  return mode;
}

export function readShellChromeMode(): ShellChromeMode {
  if (typeof document === "undefined") return "vibrancy";
  return document.documentElement.dataset.shellChrome === "solid" ? "solid" : "vibrancy";
}

export async function hydrateAppearancePreferences(): Promise<AppearancePreferences> {
  const bridge = appearancePreferencesBridge();
  if (!bridge) {
    applyShellChromeMode(false);
    return { version: 1, reduceTransparency: false };
  }
  const preferences = await bridge.getAppearancePreferences();
  applyShellChromeMode(preferences.reduceTransparency);
  return preferences;
}

export async function persistReduceTransparency(
  reduceTransparency: boolean,
): Promise<AppearancePreferences> {
  const bridge = appearancePreferencesBridge();
  if (!bridge) {
    applyShellChromeMode(reduceTransparency);
    return { version: 1, reduceTransparency };
  }
  const preferences = await bridge.setAppearancePreferences({ reduceTransparency });
  applyShellChromeMode(preferences.reduceTransparency);
  return preferences;
}
