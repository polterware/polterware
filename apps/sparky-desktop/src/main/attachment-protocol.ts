import { realpath, stat, readFile } from "node:fs/promises";
import { isAbsolute, relative, resolve, extname } from "node:path";
import { ULID } from "./navigation-policy.js";

const mime: Record<string, string> = { ".png": "image/png", ".jpg": "image/jpeg", ".jpeg": "image/jpeg", ".webp": "image/webp",
  ".gif": "image/gif", ".mp3": "audio/mpeg", ".wav": "audio/wav", ".m4a": "audio/mp4", ".webm": "audio/webm" };
export async function resolveAttachmentUrl(raw: string, root: string): Promise<string> {
  const url = new URL(raw);
  if (url.protocol !== "sparky-file:" || url.hostname !== "attachment" || url.port || url.username || url.password || url.search || url.hash) throw new Error("Invalid attachment URL");
  const parts = url.pathname.slice(1).split("/").map(decodeURIComponent);
  if (parts.length !== 2 || !ULID.test(parts[0]!) || !/^[0-7][0-9A-HJKMNP-TV-Z]{25}\.[a-zA-Z0-9]{1,10}$/.test(parts[1]!)) throw new Error("Invalid attachment identifiers");
  const base = await realpath(root), path = await realpath(resolve(base, ...parts));
  const rel = relative(base, path);
  if (!rel || rel.startsWith("..") || isAbsolute(rel) || !(await stat(path)).isFile()) throw new Error("Attachment outside storage");
  return path;
}
export async function attachmentResponse(raw: string, root: string): Promise<Response> {
  try {
    const path = await resolveAttachmentUrl(raw, root), type = mime[extname(path).toLowerCase()];
    if (!type) return new Response(null, { status: 415 });
    return new Response(new Uint8Array(await readFile(path)), { headers: { "Content-Type": type,
      "X-Content-Type-Options": "nosniff", "Content-Security-Policy": "default-src 'none'; sandbox" } });
  } catch { return new Response(null, { status: 404 }); }
}
