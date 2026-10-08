import { render } from "solid-js/web";
import "@fontsource-variable/inter/opsz.css";
import { DesignSystemProvider } from "@polterware/frontend-design-system";
import "./styles.css";
import { App } from "./App";

const root = document.getElementById("root");
if (!root) throw new Error("root missing");

render(
  () => (
    <DesignSystemProvider storagePrefix="vitals">
      <App />
    </DesignSystemProvider>
  ),
  root,
);
