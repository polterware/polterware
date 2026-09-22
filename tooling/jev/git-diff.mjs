import { execFileSync } from "node:child_process";

const DIFF_CHAR_LIMIT = 24_000;

export function readWorktreeDiff(root, stagedOnly) {
  const args = ["diff", "--no-ext-diff", "--unified=3"];
  if (stagedOnly) args.push("--cached");
  const text = execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
  });
  return clip(text);
}

export function addedCopy(diff) {
  return diff
    .split(/\n/)
    .filter((line) => line.startsWith("+") && !line.startsWith("+++"))
    .map((line) => line.slice(1))
    .filter((line) => /["'`][^"'`]{8,}["'`]/.test(line))
    .join("\n");
}

function clip(text) {
  if (text.length <= DIFF_CHAR_LIMIT) return { text, truncated: false };
  return { text: text.slice(0, DIFF_CHAR_LIMIT), truncated: true };
}
