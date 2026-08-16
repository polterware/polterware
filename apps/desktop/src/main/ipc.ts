import { ipcMain, type IpcMain, type IpcMainInvokeEvent } from "electron";
import type { AppIconOption, AppIconSelection } from "../shared/app-icons.js";
import { IPC_CHANNELS, type AppInfo, type AppIpcRequests, type AppIpcResponses, type JsonObject, type JsonValue } from "../shared/ipc.js";

type Handler<K extends keyof AppIpcRequests> = (event?: IpcMainInvokeEvent, payload?: AppIpcRequests[K]) => AppIpcResponses[K] | Promise<AppIpcResponses[K]>;

export interface PolterwareIpcControllers {
  appInfo?: {
    getInfo(): AppInfo;
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

function createUiOnlyHandler<K extends keyof AppIpcRequests>(channel: K): Handler<K> {
  return async () => {
    throw new Error(`Polterware Desktop is running in UI-only mode. ${channel} is not connected.`);
  };
}

function isJsonObject(value: JsonValue): value is JsonObject {
  return value !== null && !Array.isArray(value) && Object(value) === value;
}

function parseAppIconSelectionPayload(payload: JsonValue | undefined): string {
  if (!payload || !isJsonObject(payload)) {
    throw new Error("App icon id is required.");
  }

  const id = payload.id;
  if (Object.prototype.toString.call(id) !== "[object String]") {
    throw new Error("App icon id is required.");
  }

  const parsedId = String(id);
  if (!parsedId.trim()) {
    throw new Error("App icon id is required.");
  }

  return parsedId;
}

export function createPolterwareIpcHandlers(
  controllers: PolterwareIpcControllers = {},
) {
  const handlers = {
    [IPC_CHANNELS.app.getInfo]: createUiOnlyHandler(IPC_CHANNELS.app.getInfo),
    [IPC_CHANNELS.app.listIcons]: createUiOnlyHandler(IPC_CHANNELS.app.listIcons),
    [IPC_CHANNELS.app.getIcon]: createUiOnlyHandler(IPC_CHANNELS.app.getIcon),
    [IPC_CHANNELS.app.setIcon]: createUiOnlyHandler(IPC_CHANNELS.app.setIcon),
    [IPC_CHANNELS.commander.hideOverlay]: createUiOnlyHandler(IPC_CHANNELS.commander.hideOverlay),
    [IPC_CHANNELS.commander.showMainWindow]: createUiOnlyHandler(IPC_CHANNELS.commander.showMainWindow),
  } satisfies { [K in keyof AppIpcRequests]: Handler<K> };

  if (controllers.appInfo) {
    const { appInfo } = controllers;
    handlers[IPC_CHANNELS.app.getInfo] = async () => appInfo.getInfo();
  }

  if (controllers.commander) {
    const { commander } = controllers;
    handlers[IPC_CHANNELS.commander.hideOverlay] = async () => {
      commander.hideOverlay();
    };
    handlers[IPC_CHANNELS.commander.showMainWindow] = async () => {
      commander.showMainWindow();
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

  target.handle(IPC_CHANNELS.app.getInfo, (event, payload: AppIpcRequests[typeof IPC_CHANNELS.app.getInfo]) => handlers[IPC_CHANNELS.app.getInfo](event, payload));
  target.handle(IPC_CHANNELS.app.listIcons, (event, payload: AppIpcRequests[typeof IPC_CHANNELS.app.listIcons]) => handlers[IPC_CHANNELS.app.listIcons](event, payload));
  target.handle(IPC_CHANNELS.app.getIcon, (event, payload: AppIpcRequests[typeof IPC_CHANNELS.app.getIcon]) => handlers[IPC_CHANNELS.app.getIcon](event, payload));
  target.handle(IPC_CHANNELS.app.setIcon, (event, payload: AppIpcRequests[typeof IPC_CHANNELS.app.setIcon]) => handlers[IPC_CHANNELS.app.setIcon](event, payload));
  target.handle(IPC_CHANNELS.commander.hideOverlay, (event, payload: AppIpcRequests[typeof IPC_CHANNELS.commander.hideOverlay]) => handlers[IPC_CHANNELS.commander.hideOverlay](event, payload));
  target.handle(IPC_CHANNELS.commander.showMainWindow, (event, payload: AppIpcRequests[typeof IPC_CHANNELS.commander.showMainWindow]) => handlers[IPC_CHANNELS.commander.showMainWindow](event, payload));

  return handlers;
}
