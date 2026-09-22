import type { JSX } from "solid-js";
import { Show } from "solid-js";
import { DetailPanel, MarkdownEditor } from "@polterware/frontend-design-system";
import { createDetailPanelSelector } from "@polterware/frontend-state";
import "./knowledge-document.css";

export function KnowledgeDocumentShell(props: {
  title: string;
  bodyMarkdown: string;
  onBodyChange: (md: string) => void;
  /** Content above the editor (links, path, artifact tabs) — stays inside Body for collapsed hide. */
  preamble?: JSX.Element;
}) {
  // Format dock only when DetailPanel is maximized (full writing surface).
  const panelMode = createDetailPanelSelector((state) => state.mode);
  const showFormatToolbar = () => panelMode() === "maximized";

  return (
    <DetailPanel.Body layout="document">
      <div class="pw-knowledge-doc">
        <Show when={props.preamble}>
          <div class="pw-knowledge-doc__preamble">{props.preamble}</div>
        </Show>
        <div class="pw-knowledge-doc__body">
          <div class="pw-knowledge-doc__column">
            <h2 class="pw-knowledge-doc__title">{props.title}</h2>
            <MarkdownEditor
              value={props.bodyMarkdown}
              onChange={props.onBodyChange}
              showToolbar={showFormatToolbar()}
              placeholder="Write in Markdown…"
              aria-label={`${props.title} body`}
            />
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
      <DetailPanel.Header title={title()} />
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
