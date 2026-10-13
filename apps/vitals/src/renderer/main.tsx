import { render } from "solid-js/web";
import "@fontsource-variable/inter/opsz.css";
import { DesignSystemProvider } from "@polterware/frontend-design-system";
import "./styles.css";
import { App } from "./App";
import { connectDemoSession, installVitalsMock } from "../web/mock-bridge";

const root = document.getElementById("root");
if (!root) throw new Error("root missing");

const usingMock = !window.vitals;
if (usingMock) installVitalsMock();

render(
  () => (
    <DesignSystemProvider storagePrefix="vitals">
      <App />
    </DesignSystemProvider>
  ),
  root,
);

if (usingMock) setTimeout(connectDemoSession, 0);
