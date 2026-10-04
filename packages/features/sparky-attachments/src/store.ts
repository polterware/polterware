import { batch, createSignal } from "solid-js";
import { browserBridge } from "./bridge";
import {
  DROP_PATH_UNAVAILABLE, MICROPHONE_DENIED, MICROPHONE_UNAVAILABLE, RECORDING_IMPORT_UNAVAILABLE,
  droppedFilePath, errorText, previewUrl, reorderIds, revisionString,
  type AttachmentJSON, type AttachmentsBridge, type MicrophoneState, type SaveState,
} from "./model";

export type AttachmentsStore = ReturnType<typeof createAttachmentsStore>;

export function createAttachmentsStore(getBridge: () => AttachmentsBridge = browserBridge) {
  const [items, setItems] = createSignal<AttachmentJSON[]>([]);
  const [state, setState] = createSignal<SaveState>("live");
  const [error, setError] = createSignal("");
  const [microphone, setMicrophone] = createSignal<MicrophoneState>("idle");
  const [fetchTitles, setFetchTitles] = createSignal(false);
  const [memoryRev, setMemoryRev] = createSignal("0");
  const [recording, setRecording] = createSignal(false);
  let unsubscribe: (() => void) | undefined;
  let users = 0;
  let activeMemory = "";

  function fail(value: unknown) {
    batch(() => { setState("failed"); setError(errorText(value)); });
  }
  async function refreshRev(memoryId: string) {
    const result = await getBridge().memories.get({ id: memoryId });
    setMemoryRev(revisionString(result.memory?.rev));
  }
  async function load(memoryId: string) {
    if (!memoryId) {
      batch(() => { setItems([]); setState("unavailable"); setError("Select a memory to attach files."); });
      return;
    }
    activeMemory = memoryId;
    setState("queued");
    try {
      const [listed, settings] = await Promise.all([
        getBridge().attachments.list({ memoryId }),
        getBridge().settings.get({}),
        refreshRev(memoryId),
      ]);
      if (activeMemory !== memoryId) return;
      batch(() => {
        setItems(listed.attachments ?? []);
        setFetchTitles(settings.settings?.linksFetchTitles ?? false);
        setState("live");
        setError("");
      });
    } catch (value) {
      if (activeMemory !== memoryId) return;
      batch(() => { setState("unavailable"); setError(errorText(value)); setItems([]); });
    }
  }
  async function importPaths(memoryId: string, paths: readonly string[]) {
    if (!paths.length) return;
    setState("queued");
    const messages: string[] = [];
    try {
      await refreshRev(memoryId);
      const expectedRev = memoryRev();
      for (const sourcePath of paths) {
        try {
          await getBridge().attachments.importFromPath({ memoryId, sourcePath, name: sourcePath.split("/").pop() ?? "", expectedRev });
        } catch (value) { messages.push(errorText(value)); }
      }
      await load(memoryId);
      if (messages.length) batch(() => { setState("failed"); setError(messages.join(" ")); });
    } catch (value) { fail(value); }
  }
  async function addLink(memoryId: string, url: string, title: string) {
    setState("queued");
    try {
      await refreshRev(memoryId);
      await getBridge().attachments.addLink({ memoryId, url, title, expectedRev: memoryRev() });
      await load(memoryId);
    } catch (value) { fail(value); }
  }
  async function remove(memoryId: string, id: string) {
    setState("queued");
    try {
      await getBridge().attachments.delete({ id });
      await load(memoryId);
    } catch (value) { fail(value); }
  }
  async function move(memoryId: string, index: number, direction: -1 | 1) {
    const orderedIds = reorderIds(items().map(item => item.id ?? ""), index, direction);
    setState("queued");
    try {
      const result = await getBridge().attachments.reorder({ memoryId, orderedIds });
      batch(() => { setItems(result.attachments ?? []); setState("live"); setError(""); });
    } catch (value) { fail(value); }
  }
  async function reveal(item: AttachmentJSON) {
    const url = previewUrl(item.relPath);
    if (!url) { fail(new Error("Preview is unavailable for this attachment.")); return; }
    try { await getBridge().app.revealInFinder(url); }
    catch (value) { fail(value); }
  }
  async function pickAndImport(memoryId: string) {
    try {
      const paths = await getBridge().app.pickFiles();
      await importPaths(memoryId, paths);
    } catch (value) { fail(value); }
  }
  function importDropped(memoryId: string, files: FileList | File[]) {
    const paths = [...files].map(droppedFilePath).filter((path): path is string => Boolean(path));
    if (!paths.length) {
      batch(() => { setState("failed"); setError(DROP_PATH_UNAVAILABLE); });
      return;
    }
    void importPaths(memoryId, paths);
  }
  async function requestMicrophone(): Promise<MicrophoneState> {
    if (typeof MediaRecorder === "undefined" || !navigator.mediaDevices?.getUserMedia) {
      setMicrophone("unavailable"); return "unavailable";
    }
    try {
      const granted = await getBridge().app.startMicrophone();
      const next: MicrophoneState = granted ? "granted" : "denied";
      setMicrophone(next);
      return next;
    } catch {
      setMicrophone("unavailable");
      return "unavailable";
    }
  }
  function microphoneCopy(value: MicrophoneState) {
    if (value === "denied") return MICROPHONE_DENIED;
    if (value === "unavailable") return MICROPHONE_UNAVAILABLE;
    return "";
  }
  function recordingImportUnavailable() {
    batch(() => { setRecording(false); setState("failed"); setError(RECORDING_IMPORT_UNAVAILABLE); });
  }
  function connect() {
    users += 1;
    if (users === 1) {
      try { unsubscribe = getBridge().onChange(event => {
        if (event.resync || event.changes.some(change => change.entity === "attachment" || change.entity === "memory" || change.entity === "setting")) {
          if (activeMemory) void load(activeMemory);
        }
      }); } catch { /* Core offline: load() reports unavailable. */ }
    }
    return () => {
      users -= 1;
      if (users <= 0) { users = 0; unsubscribe?.(); unsubscribe = undefined; }
    };
  }
  function dispose() {
    users = 0; unsubscribe?.(); unsubscribe = undefined;
    batch(() => { setItems([]); setState("live"); setError(""); setMicrophone("idle"); setRecording(false); });
  }
  return {
    items, state, error, microphone, fetchTitles, memoryRev, recording, setRecording,
    load, importPaths, addLink, remove, move, reveal, pickAndImport, importDropped,
    requestMicrophone, microphoneCopy, recordingImportUnavailable, connect, dispose, fail,
  };
}

export const attachmentsStore = createAttachmentsStore();
