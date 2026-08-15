import { ipcMain, type IpcMain, type IpcMainInvokeEvent } from "electron";
import type { AppIconOption, AppIconSelection } from "../shared/app-icons.js";
import { IPC_CHANNELS } from "../shared/ipc.js";

type Handler = (event: IpcMainInvokeEvent, payload?: unknown) => unknown | Promise<unknown>;

export interface PolterwareIpcControllers {
  appInfo?: {
    getInfo(): { name: string; version: string };
  };
  appIcons?: {
    getIcon(): AppIconSelection;
    listIcons(): AppIconOption[];
    setIcon(id: string): AppIconSelection;
  };
  commander?: {
    hideOverlay(): void;
    showMainWindow(): void;
  };
}

function collectChannels(value: unknown): string[] {
  if (typeof value === "string") {
    return [value];
  }

  if (!value || typeof value !== "object") {
    return [];
  }

  return Object.values(value).flatMap((entry) => collectChannels(entry));
}

function createUiOnlyHandler(channel: string): Handler {
  return async () => {
    throw new Error(`Polterware Desktop is running in UI-only mode. ${channel} is not connected.`);
  };
}

function parseAppIconSelectionPayload(payload: unknown): string {
  if (!payload || typeof payload !== "object") {
    throw new Error("App icon id is required.");
  }

  const input = payload as { id?: unknown };
  if (typeof input.id !== "string" || !input.id.trim()) {
    throw new Error("App icon id is required.");
  }

  return input.id;
}

export function createPolterwareIpcHandlers(
  controllers: PolterwareIpcControllers = {},
): Record<string, Handler> {
  const handlers = Object.fromEntries(
    collectChannels(IPC_CHANNELS).map((channel) => [channel, createUiOnlyHandler(channel)]),
  );

  if (controllers.appInfo) {
    handlers[IPC_CHANNELS.app.getInfo] = async () => controllers.appInfo?.getInfo();
  }

  if (controllers.commander) {
    handlers[IPC_CHANNELS.commander.hideOverlay] = async () => {
      controllers.commander?.hideOverlay();
    };
    handlers[IPC_CHANNELS.commander.showMainWindow] = async () => {
      controllers.commander?.showMainWindow();
    };
  }

  if (controllers.appIcons) {
    const { appIcons } = controllers;
    handlers[IPC_CHANNELS.app.listIcons] = async () => appIcons.listIcons();
    handlers[IPC_CHANNELS.app.getIcon] = async () => appIcons.getIcon();
    handlers[IPC_CHANNELS.app.setIcon] = async (_event, payload) =>
      appIcons.setIcon(parseAppIconSelectionPayload(payload));
  }

  return handlers;
}

export function registerPolterwareIpcHandlers(
  controllers: PolterwareIpcControllers = {},
  target: Pick<IpcMain, "handle"> = ipcMain,
) {
  const handlers = createPolterwareIpcHandlers(controllers);
  for (const [channel, handler] of Object.entries(handlers)) {
    target.handle(channel, handler);
  }
  return handlers;
}
