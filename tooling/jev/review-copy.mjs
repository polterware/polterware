import { choice, noul } from "@typesafe-ai/sdk";

import { CURRENT_SCOPE } from "./rules.mjs";
import { exitCodeForBand, formatChoice, formatNoul, formatUsage, worstNoulBand } from "./report.mjs";

export async function runCopyReview(client, copy) {
  const response = await client.systemOne({
    state: { copy, scope: CURRENT_SCOPE },
    questions: {
      kind: choice("What does `copy` describe, given `scope`?", {
        "current-behavior": "Behavior the local Desktop implements today",
        "future-direction": "Planned direction that is not current behavior",
        internal: "A development note, identifier, or log line",
      }),
      readsAsShipped: noul(
        "Would a reader treat `copy` as a capability the product already ships when `scope` says it does not?",
      ),
    },
  });
  return {
    lines: [
      formatChoice("kind", response.answers.kind),
      formatNoul("readsAsShipped", response.answers.readsAsShipped),
      formatUsage(response.usage),
    ],
    exitCode: exitCodeForBand(worstNoulBand(response.answers)),
  };
}
