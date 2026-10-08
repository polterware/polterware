import { app } from "electron";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";

function themePath(): string {
  return join(app.getPath("userData"), "theme.json");
}

export function readTheme(): "dark" | "light" {
  try {
    const raw = readFileSync(themePath(), "utf8");
    const parsed = JSON.parse(raw) as { theme?: string };
    return parsed.theme === "light" ? "light" : "dark";
  } catch {
    return "dark";
  }
}

export function writeTheme(theme: "dark" | "light"): void {
  mkdirSync(app.getPath("userData"), { recursive: true });
  writeFileSync(themePath(), JSON.stringify({ theme }), "utf8");
}
