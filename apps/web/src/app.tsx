import { Suspense } from "solid-js";
import { Router } from "@solidjs/router";
import { FileRoutes } from "@solidjs/start/router";
import { MetaProvider } from "@solidjs/meta";
import { DesignSystemProvider } from "@polterware/frontend-design-system";
import { SiteHeader } from "./components/SiteHeader";
import { SiteFooter } from "./components/SiteFooter";
import "@fontsource-variable/inter/opsz.css";
import "./styles.css";

export default function App() {
  return <Router root={props => <MetaProvider>
    <DesignSystemProvider storagePrefix="polterware.web">
      <a class="skip-link" href="#main">Skip to content</a>
      <SiteHeader />
      <Suspense>{props.children}</Suspense>
      <SiteFooter />
    </DesignSystemProvider>
  </MetaProvider>}><FileRoutes /></Router>;
}
