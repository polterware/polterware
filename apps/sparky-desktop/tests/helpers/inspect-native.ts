import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { Sparky } from "../fixtures";
const run = promisify(execFile);
export async function inspectNative(sparky: Sparky, command: "pending" | "delivered") {
  const resultPath = join(sparky.directory, `${command}-result.json`);
  const environmentFile = join(sparky.directory, `${command}-environment.json`);
  await writeFile(environmentFile, JSON.stringify({ SPARKY_CORE_DB: sparky.database, SPARKY_NOTIFY_INSPECT: command, SPARKY_NOTIFY_INSPECT_RESULT: resultPath }), { mode: 0o600 });
  const env = Object.fromEntries(Object.entries(process.env).filter(([key, value]) => value !== undefined && !key.startsWith("SPARKY_") && key !== "ELECTRON_RUN_AS_NODE")) as Record<string, string>;
  await run(process.env.SPARKY_E2E_EXECUTABLE!, [`--user-data-dir=${join(sparky.directory, `inspect-${command}`)}`, `--sparky-e2e-env=${environmentFile}`], { env, timeout: 15000 });
  return JSON.parse(await readFile(resultPath, "utf8")) as { ok: boolean; pending?: { id: string }[]; delivered?: { id: string }[] };
}
