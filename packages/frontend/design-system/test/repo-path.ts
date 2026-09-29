import { resolve } from "node:path";

/** Absolute path of a repository-relative file, independent of the directory tests run from. */
export function repoPath(relative: string): string {
  return resolve(import.meta.dirname, "../../../..", relative);
}
