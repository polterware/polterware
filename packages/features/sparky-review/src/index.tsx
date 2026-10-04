import type { SparkyFeature } from "@polterware/feature-sparky-shell/feature";
import { ReviewPage } from "./review-page";

const feature: SparkyFeature = {
  id: "review",
  routes: [{ path: "/review", title: "Review", component: ReviewPage }],
  sidebar: [{ id: "review", label: "Review", path: "/review", icon: "Activity" }],
};

export default feature;
export { ReviewPage };
export { createReviewStore, reviewStore } from "./store";
export { createReviewBridge, browserBridge } from "./bridge";
export {
  CORE_COMPLETED_MOOD,
  CORE_STREAK_MOOD,
  HEATMAP_THRESHOLDS,
  completionTarget,
  reviewWindow,
  streakTarget,
  weekStartFromSettings,
} from "./model";
