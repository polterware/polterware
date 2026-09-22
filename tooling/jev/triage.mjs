import { choice, noul } from "@typesafe-ai/sdk";

import { AREAS, CURRENT_SCOPE } from "./rules.mjs";
import { exitCodeForBand, formatChoice, formatNoul, formatUsage, worstNoulBand } from "./report.mjs";

export function triageQuestions() {
  return {
    area: choice("Which repository area should be opened first for `task`, given `scope`?", AREAS),
    outsideCurrentSpec: noul(
      "Does `task` ask for behavior that `scope` places outside the current Desktop, such as a Web App, Cloud, a project-model agent, or hosted workspace?",
    ),
    newSurface: noul(
      "Does `task` add a product surface beyond Architecture, Resources, Knowledge, Operations, Terminal, Integrations, and Settings?",
    ),
  };
}

export async function runTriage(client, task) {
  const response = await client.systemOne({
    state: { task, scope: CURRENT_SCOPE },
    questions: triageQuestions(),
  });
  const lines = [
    formatChoice("area", response.answers.area),
    formatNoul("outsideCurrentSpec", response.answers.outsideCurrentSpec),
    formatNoul("newSurface", response.answers.newSurface),
    formatUsage(response.usage),
  ];
  return {
    lines,
    exitCode: exitCodeForBand(worstNoulBand(response.answers)),
  };
}
