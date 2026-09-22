import { Button, pushToast } from "@polterware/frontend-design-system";
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

/** One-line Polterware AI skill export (copy / download). */
export function SkillCard() {
  return (
    <section class="rounded-[var(--radius-card)] border border-[var(--color-border-subtle)] bg-[var(--color-surface)] px-3 py-2.5">
      <div class="flex items-center justify-between gap-3">
        <p class="m-0 text-[12px] font-medium text-[var(--color-text-primary)]">
          Polterware AI skill
        </p>
        <div class="flex flex-wrap gap-1.5">
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
      </div>
    </section>
  );
}
