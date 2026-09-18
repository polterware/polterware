import { asRecord, asString } from "./format";

function core() {
  return window.polterware.core;
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const value of bytes) binary += String.fromCharCode(value);
  return btoa(binary);
}

function utf8ToBase64(value: string): string {
  return bytesToBase64(new TextEncoder().encode(value));
}

function base64ToUtf8(value: string): string {
  if (!value) return "";
  const binary = atob(value);
  const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));
  return new TextDecoder().decode(bytes);
}

function coerceBase64(data: unknown): string {
  if (typeof data === "string") return data;
  if (data && typeof data === "object" && "data" in (data as object)) {
    return coerceBase64((data as { data: unknown }).data);
  }
  return "";
}

export async function createLiveTerminalSession(input: {
  projectId: string;
  targetId: string;
  cols: number;
  rows: number;
}): Promise<{
  id: string;
  name: string;
  targetId: string;
  nodeName: string;
  shell: string;
  status: "active";
  startedAt: string;
}> {
  const response = asRecord(
    await core().terminals.create({
      projectId: input.projectId,
      targetId: input.targetId,
      cols: input.cols,
      rows: input.rows,
    }),
  );
  const session = asRecord(response.session);
  return {
    id: asString(session.id),
    name: asString(session.targetId) || asString(session.id),
    targetId: asString(session.targetId) || input.targetId,
    nodeName: asString(session.targetId) || input.targetId,
    shell: asString(session.shell) || "local",
    status: "active",
    startedAt: asString(session.startedAt) || new Date().toISOString(),
  };
}

export async function writeLiveTerminalSession(id: string, data: string): Promise<void> {
  await core().terminals.write({
    id,
    data: utf8ToBase64(data),
  });
}

export async function resizeLiveTerminalSession(id: string, cols: number, rows: number): Promise<void> {
  await core().terminals.resize({ id, cols, rows });
}

export async function closeLiveTerminalSession(id: string): Promise<void> {
  await core().terminals.close({ id });
}

export async function readLiveTerminalOutput(
  id: string,
  offset: number,
): Promise<{ data: string; nextOffset: number; closed: boolean }> {
  const response = asRecord(
    await core().terminals.read({
      id,
      offset,
    }),
  );
  const encoded = coerceBase64(response.data);
  return {
    data: base64ToUtf8(encoded),
    nextOffset: Number(response.nextOffset ?? offset),
    closed: Boolean(response.closed),
  };
}
