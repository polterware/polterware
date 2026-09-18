import { onCleanup, onMount } from "solid-js";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";

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

  onMount(() => {
    if (!host) return;
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
    fitAddon.fit();
    props.onResize?.(terminal.cols, terminal.rows);

    terminal.onData((data) => {
      props.onWrite(data);
    });

    const onWindowResize = () => {
      fitAddon?.fit();
      if (terminal) props.onResize?.(terminal.cols, terminal.rows);
    };
    window.addEventListener("resize", onWindowResize);

    const poll = async () => {
      if (closed || !terminal) return;
      try {
        const chunk = await props.onRead(offset);
        if (chunk.data) {
          terminal.write(chunk.data);
          offset = chunk.nextOffset;
        }
        if (chunk.closed) {
          closed = true;
          terminal.writeln("\r\n[session closed]");
          return;
        }
      } catch {
        // Keep polling; transient IPC errors should not kill the session UI.
      }
      timer = window.setTimeout(() => {
        void poll();
      }, 50);
    };
    void poll();

    onCleanup(() => {
      closed = true;
      if (timer) window.clearTimeout(timer);
      window.removeEventListener("resize", onWindowResize);
      terminal?.dispose();
    });
  });

  return <div ref={host} class="h-full min-h-0 w-full overflow-hidden bg-[#111]" data-session={props.sessionId} />;
}
