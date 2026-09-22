import { noul } from "@typesafe-ai/sdk";

import { CURRENT_SCOPE } from "./rules.mjs";
import { exitCodeForBand, formatNoul, formatUsage, worstNoulBand } from "./report.mjs";

export function reviewQuestions() {
  return {
    rendererNode: noul(
      "Does `diff` make the renderer spawn a shell or call a Node API, outside window.polterware, against `scope`?",
    ),
    providerCredential: noul(
      "Does `diff` store a provider credential in SQLite, or add an integration outside the nine-provider catalog in `scope`?",
    ),
    futureAsShipped: noul(
      "Does `diff` describe a Web App, Cloud, or an in-product agent as behavior that already ships, against `scope`?",
    ),
    handRolledControl: noul(
      "Does `diff` hand-roll a button, input, dialog, menu, table, or entity panel instead of using the shared controls named in `scope`?",
    ),
    knowledgeScope: noul(
      "Does `diff` make Knowledge expose something other than Docs and Runbooks?",
    ),
  };
}

export async function runReview(client, diff) {
  const response = await client.systemOne({
    state: { diff, scope: CURRENT_SCOPE },
    questions: reviewQuestions(),
  });
  const lines = [
    formatNoul("rendererNode", response.answers.rendererNode),
    formatNoul("providerCredential", response.answers.providerCredential),
    formatNoul("futureAsShipped", response.answers.futureAsShipped),
    formatNoul("handRolledControl", response.answers.handRolledControl),
    formatNoul("knowledgeScope", response.answers.knowledgeScope),
    formatUsage(response.usage),
  ];
  return {
    lines,
    exitCode: exitCodeForBand(worstNoulBand(response.answers)),
  };
}
