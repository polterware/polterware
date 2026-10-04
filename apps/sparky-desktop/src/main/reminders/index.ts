import { app } from "electron";
import { existsSync, mkdirSync, readFileSync, watch, writeFileSync } from "node:fs";
import { isAbsolute, join } from "node:path";
import type { CoreClients } from "../core/client.js";
import { createReminderController } from "./controller";
import { loadNativeNotify } from "./native";
import { createNotifyLog } from "./log";
import { createReminderCore } from "./rpc";

export { isBackgroundReminderLaunch } from "./launch";
import { isBackgroundReminderLaunch } from "./launch";

export interface InstallOptions {
  ensureCore(): Promise<Pick<CoreClients, "reminders" | "schedule" | "settings">>;
  openMemory(url: string): void | Promise<void>;
  hasWindow(): boolean;
  quit(): void;
  dataDirectory: string;
  background?: boolean;
}
function resolveNativeModule(): string {
  const override = process.env.SPARKY_NOTIFY_MODULE?.trim();
  if (override) {
    if (!isAbsolute(override)) throw new Error("SPARKY_NOTIFY_MODULE must be absolute");
    return override;
  }
  return app.isPackaged ? join(process.resourcesPath, "native/notify.node") : join(app.getAppPath(), "native/build/notify.node");
}
function attachE2eHooks(native: { emitSystemEvent(reason: "timezone" | "wake"): Promise<void> }, dataDirectory: string): () => void {
  if (process.env.SPARKY_NOTIFY_E2E !== "1") return () => {};
  const hookPath = join(dataDirectory, "run/e2e-hook");
  mkdirSync(join(dataDirectory, "run"), { recursive: true, mode: 0o700 });
  writeFileSync(hookPath, "", { mode: 0o600 });
  const watcher = watch(hookPath, () => {
    try {
      const reason = readFileSync(hookPath, "utf8").trim();
      if (reason === "timezone" || reason === "wake") void native.emitSystemEvent(reason as "timezone" | "wake");
    } catch { /* Isolated e2e only. */ }
  });
  return () => watcher.close();
}
// Call immediately after the single-instance lock, BEFORE app.whenReady().
export function installReminders(options: InstallOptions) {
  const log = createNotifyLog(options.dataDirectory);
  const background = options.background ?? isBackgroundReminderLaunch(process.argv);
  app.setActivationPolicy("accessory");
  const nativePath = resolveNativeModule();
  if (!existsSync(nativePath)) {
    log("native-unavailable");
    throw new Error("Native reminders unavailable; build the macOS notification addon");
  }
  try {
    const native = loadNativeNotify(nativePath);
    log("delegate-installed");
    const stopHooks = attachE2eHooks(native, options.dataDirectory);
    const controller = createReminderController({ ...options, background, native, log, ensureCore: async () => createReminderCore(await options.ensureCore()) });
    return { ...controller,
      prepareLaunch(launchInfo?: unknown) {
        const hidden = controller.shouldStartHidden() || isBackgroundReminderLaunch(process.argv, launchInfo);
        controller.setBackgroundLaunch(hidden);
        app.setActivationPolicy(hidden ? "accessory" : "regular");
        return hidden;
      },
      close() { stopHooks(); controller.close(); },
    };
  } catch {
    log("native-unavailable");
    throw new Error("Native reminders unavailable; build the macOS notification addon");
  }
}
