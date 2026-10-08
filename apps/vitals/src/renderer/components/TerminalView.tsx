import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import { createEffect, onCleanup, onMount } from "solid-js";

export function TerminalView(props: { sessionId: string; connected: boolean; visible: boolean }) {
  let host: HTMLDivElement | undefined;
  let terminal: Terminal | undefined;
  let fit: FitAddon | undefined;
  let opened = false;

  const fitNow = () => {
    if (!props.visible || !host || !terminal || !fit) return;
    if (host.clientWidth < 16 || host.clientHeight < 16) return;
    try {
      fit.fit();
    } catch {
      return;
    }
    if (opened) {
      void window.vitals.shellResize({ sessionId: props.sessionId, cols: terminal.cols, rows: terminal.rows });
    }
  };

  const openShell = () => {
    if (opened || !terminal) return;
    opened = true;
    void window.vitals
      .shellOpen({ sessionId: props.sessionId, cols: terminal.cols, rows: terminal.rows })
      .catch((error: unknown) => {
        opened = false;
        terminal?.writeln(`\r\n[${error instanceof Error ? error.message : "Shell failed"}]`);
      });
  };

  onMount(() => {
    if (!host) return;
    const sessionId = props.sessionId;
    terminal = new Terminal({
      cursorBlink: true,
      fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
      fontSize: 13,
      theme: { background: "#111111", foreground: "#e8e8e8" },
    });
    fit = new FitAddon();
    terminal.loadAddon(fit);
    terminal.open(host);

    terminal.onData((data) => {
      if (opened) void window.vitals.shellWrite({ sessionId, data });
    });

    const offData = window.vitals.onShellData((ev) => {
      if (ev.sessionId === sessionId) terminal?.write(ev.data);
    });
    const offExit = window.vitals.onShellExit((ev) => {
      if (ev.sessionId !== sessionId) return;
      opened = false;
      terminal?.writeln("\r\n[shell closed — reopen the Terminal tab to start a new one]");
    });

    const observer = new ResizeObserver(() => fitNow());
    observer.observe(host);

    onCleanup(() => {
      observer.disconnect();
      offData();
      offExit();
      void window.vitals.shellClose(sessionId);
      terminal?.dispose();
      terminal = undefined;
    });
  });

  createEffect(() => {
    if (!props.visible || !props.connected) return;
    requestAnimationFrame(() => {
      fitNow();
      openShell();
    });
  });

  // Reconnect drops the SSH channel: allow the next visible/connected pass to reopen it.
  createEffect(() => {
    if (!props.connected) opened = false;
  });

  return <div ref={host} class="vitals-terminal" style={{ height: "100%", width: "100%" }} />;
}
