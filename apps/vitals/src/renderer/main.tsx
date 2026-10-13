import { render } from "solid-js/web";
import "@fontsource-variable/inter/opsz.css";
import "./styles.css";
import { App } from "./App";
import { VitalsDesignSystemProvider } from "./design-system";
import { connectDemoSession, installVitalsMock } from "../web/mock-bridge";

const root = document.getElementById("root");
if (!root) throw new Error("root missing");

const usingMock = !window.vitals;
if (usingMock) installVitalsMock();

render(
  () => (
    <VitalsDesignSystemProvider storagePrefix="vitals">
      <App />
    </VitalsDesignSystemProvider>
  ),
  root,
);

if (usingMock) setTimeout(connectDemoSession, 0);
