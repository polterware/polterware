import { mkdtemp, mkdir, writeFile, symlink, rm, realpath } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { attachmentResponse, resolveAttachmentUrl } from "./attachment-protocol.js";
const id = "01K6N3QWJ8P9A7B6C5D4E3F2G1";
describe("attachment protocol", () => {
  it("serves allowed previews, rejects active content and symlink escapes", async () => {
    const temp = await mkdtemp(join(tmpdir(), "sparky-files-"));
    try {
      const root = join(temp, "attachments"); await mkdir(join(root, id), { recursive: true });
      const path = join(root, id, `${id}.png`); await writeFile(path, "image");
      const url = `sparky-file://attachment/${id}/${id}.png`;
      expect(await resolveAttachmentUrl(url, root)).toBe(await realpath(path));
      expect((await attachmentResponse(url, root)).headers.get("Content-Type")).toBe("image/png");
      await writeFile(join(root, id, `${id}.html`), "<script>bad()</script>");
      expect((await attachmentResponse(url.replace(".png", ".html"), root)).status).toBe(415);
      await writeFile(join(temp, "outside.png"), "private"); await rm(path); await symlink(join(temp, "outside.png"), path);
      expect((await attachmentResponse(url, root)).status).toBe(404);
      for (const raw of ["file:///etc/passwd", "sparky-file://attachment/../../etc/passwd", `sparky-file://attachment/${id}/%2Fetc%2Fpasswd`, url + "?x=1"]) {
        await expect(resolveAttachmentUrl(raw, root)).rejects.toThrow();
      }
    } finally { await rm(temp, { recursive: true, force: true }); }
  });
});
