import { choice, noul } from "@typesafe-ai/sdk";

import { CURRENT_SCOPE } from "./rules.mjs";
import { exitCodeForBand, formatChoice, formatNoul, formatUsage } from "./report.mjs";
import { bandForProbability } from "./thresholds.mjs";

const CANDIDATE_LIMIT = 24;

export function candidateCriteria(candidates) {
  if (candidates.length < 2) {
    throw new Error("pick needs at least two candidates after --");
  }
  if (candidates.length > CANDIDATE_LIMIT) {
    throw new Error(`pick accepts at most ${CANDIDATE_LIMIT} candidates.`);
  }
  return Object.fromEntries(candidates.map((candidate) => [candidate, candidate]));
}

export async function runPick(client, task, candidates) {
  const criteria = candidateCriteria(candidates);
  const response = await client.systemOne({
    state: { task, scope: CURRENT_SCOPE, candidates },
    questions: {
      reading: choice(
        "Which candidate should be read first to carry out `task`? Choose none when none of them is the right starting point.",
        { ...criteria, none: "No listed candidate is the right place to start" },
      ),
      covered: noul("Does one of `candidates` cover the first reading needed for `task`?"),
    },
  });
  const coveredBand = bandForProbability(response.answers.covered.noul);
  const missed = response.answers.reading.choice === "none" || coveredBand === "clear";
  return {
    lines: [
      formatChoice("reading", response.answers.reading),
      formatNoul("covered", response.answers.covered),
      formatUsage(response.usage),
    ],
    exitCode: missed ? exitCodeForBand("hold") : 0,
  };
}
