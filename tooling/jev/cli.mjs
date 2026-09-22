#!/usr/bin/env node

import path from "node:path";
import { fileURLToPath } from "node:url";

import { createClient } from "./client.mjs";
import { addedCopy, readWorktreeDiff } from "./git-diff.mjs";
import { loadTypeSafeApiKey } from "./load-env.mjs";
import { runPick } from "./pick.mjs";
import { runCopyReview } from "./review-copy.mjs";
import { runReview } from "./review-diff.mjs";
import { runTriage } from "./triage.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const HELP = `Usage: pnpm jev <command>

  triage <task>                 Judge where a task belongs before opening files.
  pick <task> -- <candidate...> Choose the first path or heading to read.
  review [--staged]             Judge the current git diff against v1 rules.
  copy [text...]                Judge UI or doc copy. Defaults to strings added in the diff.
`;

async function main() {
  const [command, ...rest] = process.argv.slice(2);
  if (!command || command === "--help" || command === "-h") {
    process.stdout.write(HELP);
    return;
  }

  loadTypeSafeApiKey(ROOT);
  const client = createClient();
  const result = await run(client, command, rest);
  for (const line of result.lines) process.stdout.write(`${line}\n`);
  process.exitCode = result.exitCode;
}

async function run(client, command, rest) {
  if (command === "triage") {
    const task = rest.join(" ").trim();
    if (!task) throw new Error("triage needs the task text.");
    return runTriage(client, task);
  }
  if (command === "pick") return runPickCommand(client, rest);
  if (command === "review") return runReviewCommand(client, rest);
  if (command === "copy") return runCopyCommand(client, rest);
  throw new Error(`Unknown command "${command}".\n${HELP}`);
}

async function runPickCommand(client, rest) {
  const separator = rest.indexOf("--");
  if (separator === -1) throw new Error("pick needs candidates after --");
  const task = rest.slice(0, separator).join(" ").trim();
  const candidates = rest.slice(separator + 1).filter(Boolean);
  if (!task) throw new Error("pick needs the task text before --");
  return runPick(client, task, candidates);
}

async function runReviewCommand(client, rest) {
  const stagedOnly = rest.includes("--staged");
  const diff = readWorktreeDiff(ROOT, stagedOnly);
  if (!diff.text.trim()) {
    return { lines: ["No diff to review."], exitCode: 0 };
  }
  const result = await runReview(client, diff.text);
  if (diff.truncated) result.lines.unshift("Diff clipped to 24000 characters.");
  return result;
}

async function runCopyCommand(client, rest) {
  const provided = rest.join(" ").trim();
  const copy = provided || addedCopy(readWorktreeDiff(ROOT, false).text);
  if (!copy.trim()) {
    return { lines: ["No copy to review."], exitCode: 0 };
  }
  return runCopyReview(client, copy);
}

main().catch((error) => {
  const message = error instanceof Error ? error.message : "Jev command failed.";
  process.stderr.write(`${message}\n`);
  process.exitCode = 1;
});
