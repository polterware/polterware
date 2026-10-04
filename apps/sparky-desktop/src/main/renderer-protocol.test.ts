import { mkdtemp, mkdir, writeFile, symlink, link, rm, realpath } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vitest";
import { rendererFileResponse } from "./renderer-protocol";

describe("renderer file protocol", () => {
  it("serves bundled assets and denies private files, symlinks and hard links", async () => {
    const temp = await realpath(await mkdtemp(join(tmpdir(), "sparky-renderer-security-")));
    try {
      const root = join(temp, "renderer");
      await mkdir(join(root, "assets"), { recursive: true });
      await writeFile(join(root, "index.html"), "<html>Sparky</html>");
      await writeFile(join(root, "assets", "app.js"), "console.log('app')");
      const secret = join(temp, "core.json");
      await writeFile(secret, "private");
      const response = (path: string) => rendererFileResponse(pathToFileURL(path).href, root);
      expect((await response(join(root, "index.html"))).status).toBe(200);
      expect((await response(join(root, "assets", "app.js"))).headers.get("Content-Type")).toBe("text/javascript");
      expect((await response(secret)).status).toBe(404);
      await symlink(secret, join(root, "assets", "secret.js"));
      expect((await response(join(root, "assets", "secret.js"))).status).toBe(404);
      await symlink(temp, join(root, "escape"));
      await writeFile(join(temp, "secret.js"), "private");
      expect((await response(join(root, "escape", "secret.js"))).status).toBe(404);
      await link(secret, join(root, "assets", "linked.js"));
      expect((await response(join(root, "assets", "linked.js"))).status).toBe(404);
      expect((await rendererFileResponse("file://evil/etc/passwd", root)).status).toBe(404);
    } finally { await rm(temp, { recursive: true, force: true }); }
  });
});
