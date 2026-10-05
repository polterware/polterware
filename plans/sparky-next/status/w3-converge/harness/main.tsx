import { render } from "solid-js/web";
import {
  DesignSystemProvider,
  PageEmpty,
  PageHeader,
} from "@polterware/frontend-design-system";
import {
  ENVIRONMENT_FIXTURES,
  PROJECT_SWITCHER_FIXTURES,
  Shell,
} from "@polterware/feature-shell";
import shellFeature, {
  createShellBridge,
  SparkyShell,
  type ShellWireBridge,
} from "@polterware/feature-sparky-shell";
import "@fontsource-variable/inter/opsz.css";

const params = new URLSearchParams(location.search);
const app = params.get("app") === "sparky" ? "sparky" : "polterware";
document.documentElement.dataset.shellChrome = params.get("solid") === "true" ? "solid" : "vibrancy";

if (app === "sparky") {
  await import("../../../../../apps/sparky-desktop/src/renderer/styles.css");
  localStorage.setItem("sparky.shell.onboarding", "complete");
  const wire = {
    reminders: {
      status: async () => ({
        permission: "authorized",
        armed: 2,
        planned: 0,
        notifierLastSeenAt: new Date().toISOString(),
      }),
    },
    minds: { list: async () => ({ minds: [] }) },
    memories: {
      list: async () => ({ total: 3 }),
      create: async () => ({ memory: { id: "new-memory", title: "Buy milk", rev: "1" } }),
      get: async () => ({ memory: { id: "new-memory", title: "Buy milk", note: "Local note", rev: "1" } }),
    },
    tags: { list: async () => ({ tags: [] }) },
    search: { search: async () => ({ hits: [] }) },
    onChange: () => () => undefined,
    app: {
      onDeepLink: () => () => undefined,
      getDeepLink: async () => null,
    },
  } as unknown as ShellWireBridge;
  render(
    () => <SparkyShell features={[shellFeature]} bridge={createShellBridge(wire)} />,
    document.getElementById("root")!,
  );
} else {
  await import("../../../../../apps/desktop/src/renderer/styles.css");
  render(
    () => (
      <DesignSystemProvider storagePrefix="polterware">
        <Shell
          projects={PROJECT_SWITCHER_FIXTURES}
          environments={ENVIRONMENT_FIXTURES}
          canGoBack
          canGoForward
          navCounts={{ inbox: { count: 4, tone: "muted", label: "4 open Inbox items" } }}
        >
          <PageHeader title="Inbox" />
          <PageEmpty title="No selection" description="Select an item to see its details." />
        </Shell>
      </DesignSystemProvider>
    ),
    document.getElementById("root")!,
  );
}

if (params.get("commander") === "1") {
  queueMicrotask(() => {
    const button = [...document.querySelectorAll("button")].find((el) =>
      /commander|search or run a command/i.test(el.getAttribute("aria-label") ?? ""),
    );
    button?.click();
  });
}
