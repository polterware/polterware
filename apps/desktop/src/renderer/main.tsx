/* @refresh reload */
import { render } from "solid-js/web";
import { App } from "./App";
import { CommanderApp } from "./CommanderApp";
import { attachDesktopDropGuards } from "./desktop-drop-guards.js";
import { hideBootSplash } from "./boot-splash.js";
import { applyShellChromeFromBootQuery } from "./shell-chrome-boot.js";
import "./styles.css";

attachDesktopDropGuards();
applyShellChromeFromBootQuery();

const root = document.getElementById("root");
if (!root) throw new Error("root missing");

const surface = new URLSearchParams(window.location.search).get("surface");

if (surface === "commander") {
  document.documentElement.classList.add("commander-surface");
  hideBootSplash({ immediate: true });
  render(() => <CommanderApp />, root);
} else {
  render(() => <App />, root);
}
