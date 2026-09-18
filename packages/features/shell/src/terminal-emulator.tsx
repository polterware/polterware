import { onCleanup, onMount } from "solid-js";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";

function isTerminalGoneError(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error ?? "");
  const lower = message.toLowerCase();
  return (
    lower.includes("terminal session not found") ||
    lower.includes("not_found") ||
    lower.includes("not found") ||
    lower.includes("code: 5") ||
    lower.includes("code: 13")
  );
}

export function TerminalEmulator(props: {
  sessionId: string;
  onWrite: (data: string) => void;
  onResize?: (cols: number, rows: number) => void;
  onRead: (offset: number) => Promise<{ data: string; nextOffset: number; closed: boolean }>;
}) {
  let host: HTMLDivElement | undefined;
  let terminal: Terminal | undefined;
  let fitAddon: FitAddon | undefined;
  let offset = 0;
  let timer: number | undefined;
  let closed = false;
  let sessionReady = false;

  onMount(() => {
    if (!host) return;
    const sessionId = props.sessionId;
    if (!sessionId) return;

    terminal = new Terminal({
      cursorBlink: true,
      fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
      fontSize: 12,
      theme: {
        background: "#111111",
        foreground: "#e8e8e8",
      },
    });
    fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(host);

    const pushResize = () => {
      if (closed || !sessionReady || !terminal) return;
      const cols = terminal.cols;
      const rows = terminal.rows;
      if (cols < 2 || rows < 2) return;
      void Promise.resolve(props.onResize?.(cols, rows)).catch((error: unknown) => {
        if (isTerminalGoneError(error)) stop("[session closed]");
      });
    };

    const fitToHost = () => {
      if (closed || !terminal || !fitAddon || !host) return;
      const width = host.clientWidth;
      const height = host.clientHeight;
      if (width < 16 || height < 16) return;
      try {
        fitAddon.fit();
      } catch {
        return;
      }
      pushResize();
    };

    terminal.onData((data) => {
      if (closed || !sessionReady) return;
      void Promise.resolve(props.onWrite(data)).catch((error: unknown) => {
        if (isTerminalGoneError(error)) stop("[session closed]");
      });
    });

    const onWindowResize = () => fitToHost();
    window.addEventListener("resize", onWindowResize);

    const resizeObserver = typeof ResizeObserver !== "undefined"
      ? new ResizeObserver(() => fitToHost())
      : undefined;
    resizeObserver?.observe(host);

    // Drawer animation / flex layout often mounts at 0×0; fit after paint.
    requestAnimationFrame(() => {
      requestAnimationFrame(() => fitToHost());
    });

    const stop = (message?: string) => {
      if (closed) return;
      closed = true;
      sessionReady = false;
      if (timer !== undefined) {
        window.clearTimeout(timer);
        timer = undefined;
      }
      if (message && terminal) {
        terminal.writeln(`\r\n${message}`);
      }
    };

    const poll = async () => {
      if (closed || !terminal) return;
      try {
        const chunk = await props.onRead(offset);
        if (closed || !terminal) return;
        if (!sessionReady) {
          sessionReady = true;
          fitToHost();
        }
        if (chunk.data) {
          terminal.write(chunk.data);
          offset = chunk.nextOffset;
        }
        if (chunk.closed) {
          stop("[session closed]");
          return;
        }
      } catch (error: unknown) {
        if (closed) return;
        if (isTerminalGoneError(error)) {
          stop("[session closed]");
          return;
        }
      }
      if (closed || !terminal) return;
      timer = window.setTimeout(() => {
        void poll();
      }, 50);
    };
    void poll();

    onCleanup(() => {
      stop();
      resizeObserver?.disconnect();
      window.removeEventListener("resize", onWindowResize);
      terminal?.dispose();
      terminal = undefined;
    });
  });

  return (
    <div
      ref={host}
      class="h-full min-h-0 w-full overflow-hidden bg-[#111]"
      style={{ height: "100%", width: "100%" }}
      data-session={props.sessionId}
    />
  );
}
