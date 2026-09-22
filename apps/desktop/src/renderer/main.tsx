/* @refresh reload */
import { render } from "solid-js/web";
import { App } from "./App";
import { CommanderApp } from "./CommanderApp";
import "./styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("root missing");

const surface = new URLSearchParams(window.location.search).get("surface");

if (surface === "commander") {
  document.documentElement.classList.add("commander-surface");
  render(() => <CommanderApp />, root);
} else {
  render(() => <App />, root);
}
