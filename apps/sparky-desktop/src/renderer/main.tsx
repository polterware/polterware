import { render } from "solid-js/web";
import "@fontsource-variable/inter/opsz.css";
import "./styles.css";
import { App } from "./App";

const root = document.getElementById("root");
if (!root) throw new Error("root missing");

render(() => <App />, root);
