import { bandForConfidence, bandForProbability } from "./thresholds.mjs";

export function formatUsage(usage) {
  if (!usage) return "usage unavailable";
  return `input ${usage.input_tokens} · output ${usage.output_tokens}`;
}

export function formatNoul(name, answer) {
  const band = bandForProbability(answer.noul);
  return `${name}  ${answer.noul.toFixed(2)}  ${band}`;
}

export function formatChoice(name, answer) {
  const band = bandForConfidence(answer.confidence);
  const ranked = Object.entries(answer.probabilities)
    .sort((left, right) => right[1] - left[1])
    .slice(0, 3)
    .map(([label, probability]) => `${label} ${probability.toFixed(2)}`)
    .join(", ");
  return `${name}  ${answer.choice}  confidence ${answer.confidence.toFixed(2)}  ${band}  [${ranked}]`;
}

export function worstNoulBand(answers) {
  return Object.values(answers).reduce((worst, answer) => {
    if (answer.type !== "noul") return worst;
    const band = bandForProbability(answer.noul);
    if (band === "hold" || worst === "hold") return "hold";
    if (band === "review") return "review";
    return worst;
  }, "clear");
}

export function exitCodeForBand(band) {
  return band === "hold" ? 2 : 0;
}
