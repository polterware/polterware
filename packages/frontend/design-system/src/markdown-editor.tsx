import {
  For,
  Show,
  createEffect,
  createSignal,
  onCleanup,
  onMount,
  type JSX,
} from "solid-js";
import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import Link from "@tiptap/extension-link";
import { marked } from "marked";
import TurndownService from "turndown";
import {
  LeftToRightListBulletIcon,
  LeftToRightListNumberIcon,
  MinusSignIcon,
  QuoteDownIcon,
  SourceCodeIcon,
  TextBoldIcon,
  TextItalicIcon,
} from "./hugeicons";
import "./markdown-editor.css";

export type MarkdownEditorProps = {
  value: string;
  onChange?: (markdown: string) => void;
  placeholder?: string;
  editable?: boolean;
  /**
   * When false, the format dock stays mounted (padding reserved) but is visually hidden.
   * Knowledge passes false while DetailPanel is not maximized.
   * Defaults to true when editable.
   */
  showToolbar?: boolean;
  class?: string;
  "aria-label"?: string;
};

const turndown = new TurndownService({
  headingStyle: "atx",
  codeBlockStyle: "fenced",
  bulletListMarker: "-",
});

marked.setOptions({ gfm: true, breaks: false });

function isStringValue(value: string | Promise<string>): value is string {
  return Object.prototype.toString.call(value) === "[object String]";
}

function markdownToHtml(markdown: string): string {
  const raw = markdown?.trim() ? markdown : "";
  const html = marked.parse(raw, { async: false });
  return isStringValue(html) && html.length > 0 ? html : "<p></p>";
}

function htmlToMarkdown(html: string): string {
  if (!html || html === "<p></p>") return "";
  return turndown.turndown(html).trim();
}

type ToolbarAction = {
  id: string;
  label: string;
  group: "format" | "structure" | "insert";
  /** Hugeicon renderer, or omit when using `text`. */
  icon?: (props: { size?: number }) => JSX.Element;
  /** Compact text label (e.g. H1 / H2 / H3). */
  text?: string;
  run: (editor: Editor) => void;
  active?: (editor: Editor) => boolean;
};

/**
 * List wrapping fails when the current block cannot nest in a list item
 * (heading, code block, etc.). Normalize to a paragraph first, then toggle.
 */
function toggleList(
  editor: Editor,
  kind: "bulletList" | "orderedList",
): void {
  const chain = editor.chain().focus();
  if (
    editor.isActive("heading") ||
    editor.isActive("codeBlock") ||
    editor.isActive("blockquote")
  ) {
    chain.clearNodes();
  }
  if (kind === "bulletList") {
    chain.toggleBulletList().run();
  } else {
    chain.toggleOrderedList().run();
  }
}

function headingAction(level: 1 | 2 | 3): ToolbarAction {
  return {
    id: `h${level}`,
    label: `Heading ${level}`,
    group: "structure",
    text: `H${level}`,
    run: (e) => e.chain().focus().toggleHeading({ level }).run(),
    active: (e) => e.isActive("heading", { level }),
  };
}

const TOOLBAR: ToolbarAction[] = [
  {
    id: "bold",
    label: "Bold",
    group: "format",
    icon: (p) => <TextBoldIcon size={p.size ?? 14} />,
    run: (e) => e.chain().focus().toggleBold().run(),
    active: (e) => e.isActive("bold"),
  },
  {
    id: "italic",
    label: "Italic",
    group: "format",
    icon: (p) => <TextItalicIcon size={p.size ?? 14} />,
    run: (e) => e.chain().focus().toggleItalic().run(),
    active: (e) => e.isActive("italic"),
  },
  headingAction(1),
  headingAction(2),
  headingAction(3),
  {
    id: "bullet",
    label: "Bullet list",
    group: "structure",
    icon: (p) => <LeftToRightListBulletIcon size={p.size ?? 14} />,
    run: (e) => toggleList(e, "bulletList"),
    active: (e) => e.isActive("bulletList"),
  },
  {
    id: "ordered",
    label: "Numbered list",
    group: "structure",
    icon: (p) => <LeftToRightListNumberIcon size={p.size ?? 14} />,
    run: (e) => toggleList(e, "orderedList"),
    active: (e) => e.isActive("orderedList"),
  },
  {
    id: "code",
    label: "Code block",
    group: "insert",
    icon: (p) => <SourceCodeIcon size={p.size ?? 14} />,
    run: (e) => e.chain().focus().toggleCodeBlock().run(),
    active: (e) => e.isActive("codeBlock"),
  },
  {
    id: "quote",
    label: "Quote",
    group: "insert",
    icon: (p) => <QuoteDownIcon size={p.size ?? 14} />,
    run: (e) => e.chain().focus().toggleBlockquote().run(),
    active: (e) => e.isActive("blockquote"),
  },
  {
    id: "hr",
    label: "Divider",
    group: "insert",
    icon: (p) => <MinusSignIcon size={p.size ?? 14} />,
    run: (e) => e.chain().focus().setHorizontalRule().run(),
  },
];

const GROUPS: Array<ToolbarAction["group"]> = ["format", "structure", "insert"];

/**
 * Markdown-backed rich text editor for Knowledge documents.
 * TipTap core (no React). Storage SSOT is Markdown; HTML is the live view.
 * Formatting dock is a fixed floating bar while editable.
 */
export function MarkdownEditor(props: MarkdownEditorProps): JSX.Element {
  let host: HTMLDivElement | undefined;
  let lastEmitted = props.value ?? "";
  const [editor, setEditor] = createSignal<Editor | undefined>();
  /** Bumps so toolbar active states recompute after transactions. */
  const [tick, setTick] = createSignal(0);

  const isEditable = () => props.editable !== false;
  const showToolbar = () => isEditable() && props.showToolbar !== false;

  onMount(() => {
    if (!host) return;
    const instance = new Editor({
      element: host,
      extensions: [
        StarterKit.configure({
          heading: { levels: [1, 2, 3] },
        }),
        Placeholder.configure({
          placeholder: props.placeholder ?? "Start writing…",
        }),
        Link.configure({
          openOnClick: false,
          HTMLAttributes: { class: "pw-md-editor__link" },
        }),
      ],
      content: markdownToHtml(props.value ?? ""),
      editable: isEditable(),
      onUpdate: ({ editor: ed }) => {
        const md = htmlToMarkdown(ed.getHTML());
        setTick((n) => n + 1);
        // Ignore no-op updates (e.g. setEditable emitting update, or identical round-trips).
        if (md === lastEmitted) return;
        lastEmitted = md;
        props.onChange?.(md);
      },
      onSelectionUpdate: () => setTick((n) => n + 1),
      editorProps: {
        attributes: {
          class: "pw-md-editor__prose",
          "aria-label": props["aria-label"] ?? "Document body",
        },
      },
    });
    // TipTap HTML round-trip may differ from the Markdown prop; adopt that baseline
    // without notifying the parent (avoids autosave / change-envelope on open).
    lastEmitted = htmlToMarkdown(instance.getHTML());
    setEditor(instance);
    setTick(1);
  });

  createEffect(() => {
    const next = props.value ?? "";
    const ed = editor();
    if (!ed) return;
    if (next === lastEmitted) return;
    const current = htmlToMarkdown(ed.getHTML());
    if (current === next) {
      lastEmitted = next;
      return;
    }
    ed.commands.setContent(markdownToHtml(next), false);
    lastEmitted = htmlToMarkdown(ed.getHTML());
  });

  createEffect(() => {
    // TipTap setEditable defaults to emitUpdate: true, which would fire onChange on mount.
    editor()?.setEditable(isEditable(), false);
  });

  onCleanup(() => {
    const ed = editor();
    ed?.destroy();
    setEditor(undefined);
  });

  function runAction(action: ToolbarAction) {
    const ed = editor();
    if (!ed || !isEditable()) return;
    action.run(ed);
    setTick((n) => n + 1);
  }

  function isActive(action: ToolbarAction): boolean {
    void tick();
    const ed = editor();
    if (!ed || !action.active) return false;
    return action.active(ed);
  }

  return (
    <div
      class={`pw-md-editor ${props.class ?? ""}`}
      data-editable={isEditable() ? "true" : "false"}
      data-toolbar={isEditable() ? "true" : "false"}
      data-floating="true"
    >
      <div ref={(el) => { host = el; }} class="pw-md-editor__surface" />
      <Show when={isEditable()}>
        <div
          class="pw-md-editor__dock"
          data-visible={showToolbar() ? "true" : "false"}
          aria-hidden={!showToolbar()}
        >
          <div
            class="pw-md-editor__toolbar menu-panel"
            role="toolbar"
            aria-label="Formatting"
          >
            <For each={GROUPS}>
              {(group) => (
                <div class="pw-md-editor__group" data-group={group}>
                  <For each={TOOLBAR.filter((action) => action.group === group)}>
                    {(action) => {
                      const Icon = action.icon;
                      return (
                        <button
                          type="button"
                          class="pw-md-editor__tool"
                          data-kind={action.text ? "text" : "icon"}
                          data-active={isActive(action) ? "true" : "false"}
                          aria-label={action.label}
                          aria-pressed={isActive(action)}
                          title={action.label}
                          disabled={!editor()}
                          onMouseDown={(event) => {
                            // Keep editor selection when clicking the dock.
                            event.preventDefault();
                          }}
                          onClick={() => runAction(action)}
                        >
                          <Show when={action.text} fallback={Icon ? <Icon size={14} /> : null}>
                            <span class="pw-md-editor__tool-text">{action.text}</span>
                          </Show>
                        </button>
                      );
                    }}
                  </For>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>
    </div>
  );
}
