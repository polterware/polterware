// Starting bands for development judgments. Calibrate them on real tasks.
export const REVIEW_AT = 0.45;
export const HOLD_AT = 0.75;

export function bandForProbability(probability) {
  if (probability >= HOLD_AT) return "hold";
  if (probability >= REVIEW_AT) return "review";
  return "clear";
}

export function bandForConfidence(confidence) {
  if (confidence < REVIEW_AT) return "hold";
  if (confidence < HOLD_AT) return "review";
  return "clear";
}
