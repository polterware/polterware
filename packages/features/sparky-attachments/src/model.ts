import type { DescMessage, DescMethodUnary, MessageShape as ContractMessage } from "@bufbuild/protobuf";
import type { AttachmentSchema, AttachmentService, MemoryService, SettingsService } from "@polterware/sparky-contracts";

/** Protobuf JSON: int64 is a decimal string, timestamps are RFC 3339. */
export type Wire<T> = T extends { $typeName: "google.protobuf.Timestamp" } ? string
  : T extends bigint ? string
  : T extends readonly (infer V)[] ? Wire<V>[]
  : T extends object ? { [K in keyof T as K extends "$typeName" ? never : K]?: Wire<T[K]> }
  : T;
export type JsonOf<S extends DescMessage> = Wire<ContractMessage<S>>;
type Namespace<S extends { method: object }> = {
  [K in keyof S["method"]]: S["method"][K] extends DescMethodUnary
    ? (input?: Wire<ContractMessage<S["method"][K]["input"]>>) => Promise<Wire<ContractMessage<S["method"][K]["output"]>>> : never;
};

export type AttachmentsBridge = {
  attachments: Namespace<typeof AttachmentService>;
  memories: Namespace<typeof MemoryService>;
  settings: Namespace<typeof SettingsService>;
  onChange(callback: (event: { resync: boolean; changes: { entity: string; id: string; op: string }[] }) => void): () => void;
  app: {
    pickFiles(): Promise<string[]>;
    revealInFinder(attachmentUrl: string): Promise<void>;
    startMicrophone(): Promise<boolean>;
  };
};

export type AttachmentJSON = JsonOf<typeof AttachmentSchema>;
export type SaveState = "live" | "queued" | "failed" | "unavailable";
export type MicrophoneState = "idle" | "granted" | "denied" | "unavailable";
export type AttachmentKind = "photo" | "audio" | "file" | "link";

const MEMORY_ID = /^[0-7][0-9A-HJKMNP-TV-Z]{25}$/;
const ATTACHMENT_FILE = /^[0-7][0-9A-HJKMNP-TV-Z]{25}\.[a-zA-Z0-9]{1,10}$/;

export function revisionString(rev: string | number | bigint | undefined): string {
  if (typeof rev === "string" && rev !== "") return rev;
  if (typeof rev === "bigint") return rev.toString();
  if (typeof rev === "number" && Number.isFinite(rev)) return Math.trunc(rev).toString();
  return "0";
}

/** Preview URLs must be the Main-served scheme. Never file://, never OpenInfo absolutePath. */
export function previewUrl(relPath: string | undefined): string | undefined {
  if (!relPath) return undefined;
  const parts = relPath.replaceAll("\\", "/").split("/").filter(Boolean);
  if (parts.length !== 2 || !MEMORY_ID.test(parts[0]!) || !ATTACHMENT_FILE.test(parts[1]!)) return undefined;
  return `sparky-file://attachment/${parts[0]}/${parts[1]}`;
}

export function linkHost(url: string | undefined): string {
  if (!url) return "";
  try { return new URL(url).host || url; } catch { return url; }
}

export function droppedFilePath(file: File): string | undefined {
  const path = (file as File & { path?: string }).path;
  return typeof path === "string" && path.startsWith("/") ? path : undefined;
}

export function reorderIds(ids: readonly string[], index: number, direction: -1 | 1): string[] {
  const target = index + direction;
  if (target < 0 || target >= ids.length) return [...ids];
  const next = [...ids];
  [next[index], next[target]] = [next[target]!, next[index]!];
  return next;
}

export function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export const FETCH_TITLES_ON = "Title fetching is on. Sparky will request this URL to read the page title (5 second timeout, no cookies). Private and loopback addresses are refused. Turn this off in Settings (links.fetchTitles).";
export const FETCH_TITLES_OFF = "Title fetching is off. Sparky will not contact this site. Add a title yourself, or enable links.fetchTitles in Settings.";
export const DROP_PATH_UNAVAILABLE = "Dropped files have no filesystem path in the renderer. Use Add photo or Add file so Main can grant the path, then import.";
export const RECORDING_IMPORT_UNAVAILABLE = "Recording finished, but Main cannot write a granted temp file yet. Save the clip and import it with Add file.";
export const MICROPHONE_DENIED = "Microphone permission denied. Grant access in System Settings to record audio.";
export const MICROPHONE_UNAVAILABLE = "Microphone is unavailable on this device.";
