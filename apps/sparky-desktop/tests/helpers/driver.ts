import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { resolve } from "node:path";
import type { Sparky } from "../fixtures";
const run = promisify(execFile);
export async function driver(sparky: Sparky, mode: "stdio" | "http" | "seed", title = "Agent memory", count = 10000) {
  const { stdout } = await run(resolve("tests/build/e2edriver"), ["-mode", mode, "-db", sparky.database, "-binary", resolve("../../dist/sparky"), "-title", title, "-count", String(count)], { timeout: 600_000 });
  return JSON.parse(stdout);
}
