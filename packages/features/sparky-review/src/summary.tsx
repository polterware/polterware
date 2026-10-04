import { Show, createSignal, createEffect, on } from "solid-js";
import { Card, EmptyIllustration, ProgressRing } from "@polterware/frontend-design-system";
import type { ReviewJSON } from "./bridge";
import {
  characterAsset,
  completedAccessibility,
  completionTarget,
  milestoneProgress,
  streakAccessibility,
  streakTarget,
} from "./model";

function CharacterArt(props: { kind: "streak" | "completed"; mood: string | undefined }) {
  const [failed, setFailed] = createSignal(false);
  const asset = () => characterAsset(props.kind, props.mood);
  createEffect(on(() => asset().src, () => setFailed(false)));
  return (
    <EmptyIllustration>
      <Show when={!failed()} fallback={<span class="sparky-review-metric__caption">Character art unavailable</span>}>
        <img src={asset().src} srcSet={asset().srcSet} alt="" width={106} height={106} onError={() => setFailed(true)} />
      </Show>
    </EmptyIllustration>
  );
}

export function SummaryCard(props: { report: ReviewJSON }) {
  const streak = () => props.report.streak ?? 0;
  const completed = () => props.report.completedTotal ?? 0;
  const streakGoal = () => streakTarget(streak());
  const completedGoal = () => completionTarget(completed());
  return (
    <Card>
      <div class="sparky-review-summary">
        <section class="sparky-review-metric" aria-label={streakAccessibility(streak(), streakGoal())}>
          <div class="sparky-review-metric__art">
            <CharacterArt kind="streak" mood={props.report.streakMood} />
            <p class="sparky-review-metric__value">{streak()}</p>
          </div>
          <p class="sparky-review-metric__title">Day streak</p>
          <ProgressRing label="Streak milestone" value={Math.round(milestoneProgress(streak(), streakGoal()) * 100)} size={36} thickness={4} />
          <p class="sparky-review-metric__caption">{streak()}/{streakGoal()} {streakGoal() === 1 ? "day" : "days"}</p>
        </section>
        <section class="sparky-review-metric" aria-label={completedAccessibility(completed(), completedGoal())}>
          <div class="sparky-review-metric__art">
            <CharacterArt kind="completed" mood={props.report.completedMood} />
            <p class="sparky-review-metric__value">{completed()}</p>
          </div>
          <p class="sparky-review-metric__title">Completed</p>
          <ProgressRing label="Completion milestone" value={Math.round(milestoneProgress(completed(), completedGoal()) * 100)} size={36} thickness={4} />
          <p class="sparky-review-metric__caption">{completed()}/{completedGoal()} completed</p>
        </section>
      </div>
    </Card>
  );
}
