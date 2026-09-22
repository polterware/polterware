/**
 * Derive a project display name from a local directory path (last segment).
 */
export function projectNameFromDirectory(path: string): string {
  const trimmed = path.trim().replace(/[/\\]+$/, "");
  if (!trimmed) return "";
  const parts = trimmed.split(/[/\\]/).filter(Boolean);
  return parts[parts.length - 1] ?? "";
}

/**
 * Whether the current name should be replaced by a new auto-suggested basename.
 * Keeps manual edits: only fill when empty or still equal to the previous auto name.
 */
export function shouldApplyAutoProjectName(
  currentName: string,
  previousAutoName: string,
): boolean {
  const trimmed = currentName.trim();
  if (trimmed.length === 0) return true;
  return trimmed === previousAutoName.trim();
}
