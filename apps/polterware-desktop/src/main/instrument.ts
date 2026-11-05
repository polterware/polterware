import { execFile } from "node:child_process";
import { appendFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import { app, type BrowserWindow } from "electron";

/**
 * The Desktop comparisons with the GPUI Desktop (`tooling/parity`, `tooling/bench`), off unless
 * an environment variable turns them on:
 *
 * - `POLTERWARE_TOUR=<dir>`: the renderer walks the screens `desktop-gpui`'s tour walks, under
 *   the same names, and each stop is captured to `<dir>/<name>.png` with `screencapture -l`
 *   (the window as composited, vibrancy included), then the app exits.
 * - `POLTERWARE_BENCH=<file>`: the renderer's milestones (`projects-ready`) are appended to the
 *   file as `{"app", "event", "at"}` lines, `at` in epoch milliseconds, for startup timing.
 */
export const instrumentEnv = {
  tourDir: process.env.POLTERWARE_TOUR?.trim() || "",
  benchFile: process.env.POLTERWARE_BENCH?.trim() || "",
};

const STOP_NAME = /^[0-9a-z][0-9a-z-]{0,63}$/;

function capture(window: BrowserWindow, path: string): Promise<void> {
  // getMediaSourceId is "window:<CGWindowID>:0" on macOS.
  const id = window.getMediaSourceId().split(":")[1];
  return new Promise((resolve, reject) => {
    execFile("screencapture", ["-x", "-o", `-l${id}`, path], (error) => (error ? reject(error) : resolve()));
  });
}

export function createInstrumentController(getWindow: () => BrowserWindow | null) {
  const { tourDir, benchFile } = instrumentEnv;
  if (tourDir) mkdirSync(tourDir, { recursive: true });
  return {
    async shot(name: string) {
      const window = getWindow();
      if (!tourDir || !STOP_NAME.test(name) || !window) return false;
      app.focus({ steal: true });
      window.focus();
      await capture(window, join(tourDir, `${name}.png`));
      return true;
    },
    mark(event: string) {
      if (!benchFile || !STOP_NAME.test(event)) return false;
      appendFileSync(benchFile, `${JSON.stringify({ app: "electron", event, at: Date.now() })}\n`);
      return true;
    },
    done() {
      if (!tourDir && !benchFile) return false;
      // exit, not quit: the quit confirmation and the core stop do not apply to a comparison.
      setTimeout(() => app.exit(0), 50);
      return true;
    },
  };
}
