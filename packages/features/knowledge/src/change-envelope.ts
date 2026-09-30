/** Summary and rationale recorded with every audited project-model change. */
export type ChangeEnvelope = {
  summary: string;
  rationale: string;
  agentRunId?: string;
};
