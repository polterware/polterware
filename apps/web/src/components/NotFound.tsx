import { getRequestEvent, isServer } from "solid-js/web";
import { LinkButton, ChevronLeft } from "@polterware/frontend-design-system";
import { Seo } from "./Seo";
export function NotFound(props: { status?: number } = {}) {
  if (isServer && props.status) {
    const event = getRequestEvent();
    if (event) event.response.status = props.status;
  }
  return <main id="main" tabindex="-1" class="site-container not-found">
    <Seo title="Page not found" description="This page could not be found." path="/404" noindex />
    <p class="eyebrow">404</p><h1>Page not found</h1><p>This page could not be found.</p>
    <LinkButton href="/"><ChevronLeft size={16} />Back to home</LinkButton>
  </main>;
}
