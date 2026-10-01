import type { JSX } from "solid-js";
import { Show } from "solid-js";
import { DetailPanel, MarkdownEditor } from "@polterware/frontend-design-system";
import { createDetailPanelSelector } from "@polterware/frontend-state";
import "./knowledge-document.css";

export function KnowledgeDocumentShell(props: {
  title: string;
  icon?: JSX.Element;
  bodyMarkdown: string;
  onBodyChange: (md: string) => void;
  /** Artifact switcher rendered above the title (Spec features). */
  tabs?: JSX.Element;
  /** Inline `DetailPanel.Properties` rendered under the title. */
  properties?: JSX.Element;
  /** `DetailPanel.LinkedEntities` rendered under the properties. */
  links?: JSX.Element;
  /** Commands associated with document metadata. */
  metadataActions?: JSX.Element;
  /** Destructive or document-wide commands after the editor. */
  footerActions?: JSX.Element;
}) {
  // Format dock only when the detail sidebar is expanded (full writing surface).
  const expanded = createDetailPanelSelector((state) => state.expanded);
  const showFormatToolbar = () => expanded();

  return (
    <DetailPanel.Body layout="document">
      <div class="pw-knowledge-doc">
        <div class="pw-knowledge-doc__body">
          <div class="pw-knowledge-doc__column">
            <Show when={props.tabs}>
              <div class="pw-knowledge-doc__tabs">{props.tabs}</div>
            </Show>
            <header class="pw-knowledge-doc__header">
              <div class="flex items-center gap-2">
                <Show when={props.icon}>
                  <span class="shrink-0 text-[var(--color-text-secondary)]" aria-hidden>{props.icon}</span>
                </Show>
                <h2 class="pw-knowledge-doc__title">{props.title}</h2>
              </div>
              <Show when={props.properties || props.links}>
                <div class="pw-knowledge-doc__meta">
                  {props.properties}
                  {props.links}
                </div>
              </Show>
              <Show when={props.metadataActions}>
                <div class="pw-detail-panel__actions">{props.metadataActions}</div>
              </Show>
            </header>
            <MarkdownEditor
              value={props.bodyMarkdown}
              onChange={props.onBodyChange}
              showToolbar={showFormatToolbar()}
              placeholder="Write in Markdown…"
              aria-label={`${props.title} body`}
            />
            <Show when={props.footerActions}>
              <div class="pw-detail-panel__actions mt-4">{props.footerActions}</div>
            </Show>
          </div>
        </div>
      </div>
    </DetailPanel.Body>
  );
}

export function KnowledgeDocumentNotFound(props: {
  kindLabel: string;
  title?: string;
}) {
  const title = () => props.title ?? `${props.kindLabel} not found`;
  return (
    <DetailPanel.Content>
      <DetailPanel.Header />
      <DetailPanel.Body layout="document">
        <div class="pw-knowledge-doc">
          <div class="pw-knowledge-doc__body">
            <div class="pw-knowledge-doc__column">
              <h2 class="pw-knowledge-doc__title">{title()}</h2>
              <p class="m-0 text-[12.5px] text-[var(--color-text-muted)]">
                This {props.kindLabel.toLowerCase()} is not available in the current project knowledge
                set. It may have been archived or the link is stale.
              </p>
            </div>
          </div>
        </div>
      </DetailPanel.Body>
    </DetailPanel.Content>
  );
}
