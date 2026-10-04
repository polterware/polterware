import { app } from "electron";
import { join } from "node:path";
import type { Transport } from "@connectrpc/connect";
import { createReminderController } from "./controller";
import { loadNativeNotify } from "./native";
import { createNotifyLog } from "./log";
import { createReminderCore } from "./rpc";

export { isBackgroundReminderLaunch } from "./launch";
import { isBackgroundReminderLaunch } from "./launch";

export interface InstallOptions {
  ensureCore(): Promise<Transport>;
  openMemory(url: string): void | Promise<void>;
  hasWindow(): boolean;
  quit(): void;
  background?: boolean;
}
// Call immediately after the single-instance lock, BEFORE app.whenReady().
// Normal window creation, single-instance forwarding and Watch are owned by W1-ELECTRON.
export function installReminders(options: InstallOptions) {
  const log = createNotifyLog(join(app.getPath("appData"), "Sparky"));
  const background = options.background ?? isBackgroundReminderLaunch(process.argv);
  // macOS reports notification cold launches using its launch event; W1-ELECTRON
  // must classify that launch before deciding whether to open a window (request 2).
  app.setActivationPolicy("accessory");
  const nativePath = app.isPackaged ? join(process.resourcesPath, "native/notify.node") : join(app.getAppPath(), "native/build/notify.node");
  try {
    const native = loadNativeNotify(nativePath);
    log("delegate-installed");
    const controller = createReminderController({ ...options, background, native, log, ensureCore: async () => createReminderCore(await options.ensureCore()) });
    return { ...controller,
      // Invoke in app.on("ready", (_event, launchInfo)), before any initial window.
      prepareLaunch(launchInfo?: unknown) {
        const hidden = controller.shouldStartHidden() || isBackgroundReminderLaunch(process.argv, launchInfo);
        controller.setBackgroundLaunch(hidden);
        app.setActivationPolicy(hidden ? "accessory" : "regular");
        return hidden;
      },
    };
  } catch {
    log("native-unavailable");
    throw new Error("Native reminders unavailable; build the macOS notification addon");
  }
}
