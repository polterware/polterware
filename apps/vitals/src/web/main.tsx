import "@fontsource-variable/inter/opsz.css";
import { render } from "solid-js/web";
import { App } from "../renderer/App";
import { VitalsDesignSystemProvider } from "../renderer/design-system";
import "../renderer/styles.css";
import "./mock.css";
import { connectDemoSession, installVitalsMock } from "./mock-bridge";

installVitalsMock();

const root = document.getElementById("root");
if (!root) throw new Error("root missing");

render(
  () => (
    <VitalsDesignSystemProvider storagePrefix="vitals-web">
      <div class="vitals-web">
        <div class="vitals-web-app">
          <App />
        </div>
      </div>
    </VitalsDesignSystemProvider>
  ),
  root,
);

setTimeout(connectDemoSession, 0);
