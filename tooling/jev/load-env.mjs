import { readFileSync } from "node:fs";
import path from "node:path";

const KEY = "TYPESAFE_API_KEY";

export function loadTypeSafeApiKey(root) {
  if (process.env[KEY]?.trim()) return;

  let text = "";
  try {
    text = readFileSync(path.join(root, ".env"), "utf8");
  } catch (error) {
    if (error && typeof error === "object" && "code" in error && error.code === "ENOENT") {
      return;
    }
    throw error;
  }

  for (const line of text.split(/\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#") || !trimmed.startsWith(`${KEY}=`)) continue;
    const value = trimmed
      .slice(KEY.length + 1)
      .trim()
      .replace(/^['"]|['"]$/g, "");
    if (value) process.env[KEY] = value;
    return;
  }
}
