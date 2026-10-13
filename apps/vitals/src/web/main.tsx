import { DesignSystemProvider, StatusChip } from "@polterware/frontend-design-system";
import "@fontsource-variable/inter/opsz.css";
import { render } from "solid-js/web";
import { App } from "../renderer/App";
import "../renderer/styles.css";
import "./mock.css";
import { connectDemoSession, installVitalsMock } from "./mock-bridge";

installVitalsMock();

const root = document.getElementById("root");
if (!root) throw new Error("root missing");

render(
  () => (
    <DesignSystemProvider storagePrefix="vitals-web">
      <div class="vitals-web">
        <div class="vitals-web-banner">
          <StatusChip tone="warn">Mock</StatusChip>
          <span>Sample hosts and metrics. Nothing leaves this browser.</span>
        </div>
        <div class="vitals-web-app">
          <App />
        </div>
      </div>
    </DesignSystemProvider>
  ),
  root,
);

setTimeout(connectDemoSession, 0);
