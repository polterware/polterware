import { Button, Card, pushToast } from "@polterware/frontend-design-system";
import {
  POLTERWARE_AI_SKILL_FILENAME,
  POLTERWARE_AI_SKILL_MARKDOWN,
} from "./polterware-ai-skill";

function downloadSkill() {
  const blob = new Blob([POLTERWARE_AI_SKILL_MARKDOWN], { type: "text/markdown;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = POLTERWARE_AI_SKILL_FILENAME;
  anchor.click();
  URL.revokeObjectURL(url);
}

/** Copy or download the Polterware AI skill that teaches MCP usage. */
export function SkillCard() {
  return (
    <Card>
      <p class="m-0 text-[12px] font-medium text-[var(--color-text-primary)]">
        Polterware AI skill
      </p>
      <p class="m-0 mt-1 text-[12px] text-[var(--color-text-muted)]">
        Markdown instructions for an agent (Cursor skill or similar) that learns how to call the
        local Polterware MCP tools against this Desktop store.
      </p>
      <div class="mt-3 flex flex-wrap gap-2">
        <Button
          size="sm"
          variant="primary"
          onClick={() => {
            void navigator.clipboard.writeText(POLTERWARE_AI_SKILL_MARKDOWN).then(
              () => pushToast("Skill copied.", "success"),
              () => pushToast("Could not copy skill.", "danger"),
            );
          }}
        >
          Copy skill
        </Button>
        <Button size="sm" variant="ghost" onClick={() => downloadSkill()}>
          Download .md
        </Button>
      </div>
      <pre class="m-0 mt-3 max-h-48 overflow-auto rounded-[var(--radius-card)] border border-[var(--color-border-subtle)] bg-[var(--color-surface-raised)] p-2 font-mono text-[10.5px] whitespace-pre-wrap text-[var(--color-text-secondary)]">
        {POLTERWARE_AI_SKILL_MARKDOWN.trim()}
      </pre>
    </Card>
  );
}
