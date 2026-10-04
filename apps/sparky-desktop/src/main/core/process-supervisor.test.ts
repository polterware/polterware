import { mkdtempSync, mkdirSync, writeFileSync, rmSync, chmodSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { buildId, isNewerBuild, readCoreState } from "./process-supervisor.js";
describe("supervisor runstate", () => {
  it("validates database identity, token, permissions and loopback endpoint", () => {
    const temp = mkdtempSync(join(tmpdir(), "sparky-state-"));
    try {
      mkdirSync(join(temp, "run")); const db = join(temp, "sparky.db"), state = join(temp, "run/core.json"); writeFileSync(db, "");
      const base = { pid: process.pid, dbPath: db, token: "test", endpoint: "http://127.0.0.1:12345" };
      const put = (value: unknown) => { writeFileSync(state, JSON.stringify(value)); chmodSync(state, 0o600); };
      put(base); expect(readCoreState(db)).toMatchObject({ token: "test", pid: process.pid });
      chmodSync(state, 0o644); expect(readCoreState(db)).toBeUndefined();
      for (const patch of [{ token: "" }, { pid: -1 }, { endpoint: "http://example.com:12345" }, { dbPath: "/other.db" }]) {
        put({ ...base, ...patch }); expect(readCoreState(db)).toBeUndefined();
      }
      expect(buildId(db)).toMatch(/^\d+-0$/);
      expect(isNewerBuild("200-5", "100-10")).toBe(true);
      expect(isNewerBuild("100-10", "200-5")).toBe(false);
    } finally { rmSync(temp, { recursive: true, force: true }); }
  });
});
